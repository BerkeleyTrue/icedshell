use std::{env::var_os, ffi::OsString};

use iced::futures::{Stream, stream};
use niri_ipc::{Event as NiriEvent, Reply, Request, socket};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

pub mod types;

#[derive(Debug, thiserror::Error)]
pub enum EventStreamError {
    #[error("No socket found for Niri")]
    NiriNoSocket,

    #[error("Failed to connect to socket")]
    NiriConnectionError(#[from] std::io::Error),

    #[error("Niri refused connection")]
    NiriConnectionRefused,

    #[error("Niri refused event stream")]
    NiriStreamRefused(String),

    #[error("Serde failed to parse")]
    SerdeErr(#[from] serde_json::Error),
}

pub struct EventStream {
    reader: BufReader<UnixStream>,
}

impl<'a> EventStream {
    fn path() -> Result<OsString, EventStreamError> {
        var_os(socket::SOCKET_PATH_ENV).ok_or(EventStreamError::NiriNoSocket)
    }

    pub async fn new() -> Result<Self, EventStreamError> {
        let path = Self::path()?;
        let req_buff = serde_json::to_string(&Request::EventStream)? + "\n";

        let mut stream = UnixStream::connect(path).await?;

        stream.writable().await?;
        stream.write_all(req_buff.as_bytes()).await?;
        stream.shutdown().await?;

        let mut reader = BufReader::new(stream);

        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let reply: Reply = serde_json::from_str(&line)?;
        reply.map_err(EventStreamError::NiriStreamRefused)?;

        Ok(Self { reader })
    }

    pub fn listen(self) -> impl Stream<Item = NiriEvent> {
        stream::unfold(self.reader, |mut reader| async {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await.ok()?;
            if bytes_read == 0 {
                return None;
            }
            match serde_json::from_str::<NiriEvent>(&line) {
                Ok(e) => Some((e, reader)),
                Err(_err) => None,
            }
        })
    }
}
