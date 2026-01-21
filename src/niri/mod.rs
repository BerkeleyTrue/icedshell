use std::{env::var_os, ffi::OsString};

use iced::futures::{Stream, stream};
use niri_ipc::{Event as NiriEvent, Reply, Request, socket};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

pub mod types;

#[derive(Debug, thiserror::Error)]
pub enum NiriStreamError {
    #[error("No socket found for Niri")]
    NiriNoSocket,

    #[error("Failed to connect to socket")]
    NiriConnectionError(#[from] std::io::Error),

    #[error("Niri refused connection")]
    NiriConnectionClosed,

    #[error("Niri refused event stream")]
    NiriStreamRefused(String),

    #[error("Serde failed to parse")]
    SerdeErr(#[from] serde_json::Error),
}

enum NiriStream {
    Disconnected,
    Connected(BufReader<UnixStream>),
}

impl NiriStream {
    fn path() -> Result<OsString, NiriStreamError> {
        var_os(socket::SOCKET_PATH_ENV).ok_or(NiriStreamError::NiriNoSocket)
    }

    pub async fn new() -> Result<BufReader<UnixStream>, NiriStreamError> {
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
        reply.map_err(NiriStreamError::NiriStreamRefused)?;

        Ok(reader)
    }

    async fn read(reader: &mut BufReader<UnixStream>) -> Result<NiriEvent, NiriStreamError> {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(NiriStreamError::NiriConnectionClosed);
        }

        serde_json::from_str::<NiriEvent>(&line).map_err(NiriStreamError::SerdeErr)
    }
}

pub fn listen() -> impl Stream<Item = Result<NiriEvent, NiriStreamError>> {
    let eventstream = NiriStream::Disconnected;
    stream::unfold(eventstream, |es| async {
        match es {
            NiriStream::Disconnected => {
                let mut reader = match NiriStream::new().await {
                    Ok(reader) => reader,
                    Err(err) => return Some((Err(err), NiriStream::Disconnected)),
                };

                let event = match NiriStream::read(&mut reader).await {
                    Ok(event) => event,
                    Err(err) => return Some((Err(err), NiriStream::Disconnected)),
                };

                Some((Ok(event), NiriStream::Connected(reader)))
            }
            NiriStream::Connected(mut reader) => {
                let event = match NiriStream::read(&mut reader).await {
                    Ok(event) => event,
                    Err(err) => return Some((Err(err), NiriStream::Disconnected)),
                };

                Some((Ok(event), NiriStream::Connected(reader)))
            }
        }
    })
}
