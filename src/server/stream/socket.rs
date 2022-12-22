use crate::cli::{TcpSocketServer, UnixSocketServer};
use crate::server::stream::{StringStream, StringStreamResult};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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
