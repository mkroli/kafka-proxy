/*
 * Copyright 2022 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::cli::{FileServer, StdInServer};
use crate::server::stream::{BytesStream, MessageStream};
use anyhow::Result;
use async_trait::async_trait;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::wrappers::LinesStream;
use tokio_stream::StreamExt;

macro_rules! buf_reader_message_stream {
    ($tp:ty, $self:ident => $reader:expr) => {
        #[async_trait]
        impl MessageStream for $tp {
            async fn stream(&$self) -> Result<BytesStream> {
                let reader = BufReader::new($reader);
                let stream = LinesStream::new(reader.lines());
                let stream = stream.map(|s| {
                    s
                        .map(|s| s.into())
                        .map_err(|e| e.into())
                });
                Ok(Box::new(stream))
            }
        }
    };
}

buf_reader_message_stream!(StdInServer, self => tokio::io::stdin());

buf_reader_message_stream!(FileServer, self => File::open(&self.file).await?);
