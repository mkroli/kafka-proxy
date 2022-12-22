use crate::cli::{FileServer, StdInServer};
use crate::server::stream::{StringStream, StringStreamResult};
use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;

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
