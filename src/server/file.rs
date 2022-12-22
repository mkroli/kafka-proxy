use crate::cli::{FileServer, StdInServer, TcpSocketServer, UnixSocketServer};
use crate::kafka_producer::KafkaProducer;
use crate::server::Server;
use anyhow::Result;
use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader, BufStream};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::{LinesStream, ReceiverStream};
use tokio_stream::{Stream, StreamExt};

type StringStreamResult = Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>>;

#[async_trait]
trait StringStream {
    async fn stream(&self) -> StringStreamResult;
}

macro_rules! buf_reader_string_stream {
    ($tp:ty, $self:ident => $reader:expr) => {
        #[async_trait]
        impl StringStream for $tp {
            async fn stream(&$self) -> StringStreamResult {
                let reader = BufReader::new($reader);
                let stream = LinesStream::new(reader.lines());
                let stream = stream.map(|s| s.map_err(|e| e.into()));
                Ok(Box::new(stream))
            }
        }
    };
}

buf_reader_string_stream!(StdInServer, self => tokio::io::stdin());

buf_reader_string_stream!(FileServer, self => File::open(&self.file).await?);

macro_rules! socket_string_stream {
    ($tp:ty, $self:ident => $listener:expr) => {
        #[async_trait]
        impl StringStream for $tp {
            async fn stream(&$self) -> StringStreamResult {
                let listener = $listener;
                let (snd, rcv) = mpsc::channel(1);
                tokio::spawn(async move {
                    while let Ok((stream, _)) = listener.accept().await {
                        let snd = snd.clone();
                        tokio::spawn(async move {
                            let mut lines = BufStream::new(stream).lines();
                            while let Ok(Some(l)) = lines.next_line().await {
                                match snd.send(Ok(l)).await {
                                    Ok(()) => (),
                                    Err(_) => break,
                                }
                            }
                        });
                    }
                });
                Ok(Box::new(ReceiverStream::new(rcv)))
            }
        }
    };
}

socket_string_stream!(UnixSocketServer, self => UnixListener::bind(&self.file)?);

socket_string_stream!(TcpSocketServer, self => TcpListener::bind(self.address).await?);

#[async_trait]
impl<T> Server for T
where
    T: StringStream + Sync,
{
    async fn run(
        &self,
        kafka_producer: KafkaProducer,
        mut shutdown_trigger_receiver: Receiver<()>,
        _shutdown_sender: Sender<()>,
    ) -> Result<()> {
        let mut lines = tokio::select! {
            _ = shutdown_trigger_receiver.recv() => return Ok(()),
            lines = self.stream() => lines?,
        };
        loop {
            tokio::select! {
                _ = shutdown_trigger_receiver.recv() => break,
                line = lines.next() => match line {
                    None => break,
                    Some(Err(e)) => log::error!("{}", e),
                    Some(Ok(line)) => kafka_producer.send(&vec!(), line.as_bytes()).await?,
                },
            }
        }
        Ok(())
    }
}
