use std::{env::var_os, ffi::OsString};

use iced::futures::{Stream, StreamExt, stream};
use niri_ipc::{Event as NiriEvent, Reply, Request, socket};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

pub mod types;

#[derive(Debug, thiserror::Error)]
enum EventStreamError {
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

enum EventStream {
    Disconnected,
    Connected(BufReader<UnixStream>),
}

impl EventStream {
    fn path() -> Result<OsString, EventStreamError> {
        var_os(socket::SOCKET_PATH_ENV).ok_or(EventStreamError::NiriNoSocket)
    }

    pub async fn new() -> Result<BufReader<UnixStream>, EventStreamError> {
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

        Ok(reader)
    }
}

pub fn listen() -> impl Stream<Item = NiriEvent> {
    let eventstream = EventStream::Disconnected;
    stream::unfold(eventstream, |es| async {
        match es {
            EventStream::Disconnected => {
                let reader = EventStream::new().await.ok()?;
                Some((None, EventStream::Connected(reader)))
            },
            EventStream::Connected(mut reader) => {
                let mut line = String::new();
                let bytes_read = reader.read_line(&mut line).await.ok()?;
                if bytes_read == 0 {
                    return Some((None, EventStream::Disconnected));
                }

                match serde_json::from_str::<NiriEvent>(&line) {
                    Ok(e) => Some((Some(e), EventStream::Connected(reader))),
                    Err(_err) => None,
                }
            }
        }
    })
    .filter_map(async |maybe_event| maybe_event)
}
