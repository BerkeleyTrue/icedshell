use std::{env::var_os, ffi::OsString, time::Duration};

use iced::futures::{Stream, stream};
pub use niri_ipc::Event as NiriEvent;
use niri_ipc::{Reply, Request, socket};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};

#[derive(Debug, thiserror::Error, Clone)]
pub enum NiriStreamError {
    #[error("No socket found for Niri")]
    NiriNoSocket,

    #[error("Failed to connect to socket")]
    NiriConnectionError(String),

    #[error("Niri refused connection")]
    NiriConnectionClosed,

    #[error("Niri refused event stream")]
    NiriStreamRefused(String),

    #[error("Serde failed to parse")]
    SerdeErr(String),
}

impl From<std::io::Error> for NiriStreamError {
    fn from(err: std::io::Error) -> Self {
        Self::NiriConnectionError(err.to_string())
    }
}

impl From<serde_json::Error> for NiriStreamError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeErr(err.to_string())
    }
}

enum NiriStream {
    Disconnected { attempts: u32 },
    Connected(BufReader<UnixStream>),
}

impl NiriStream {
    fn path() -> Result<OsString, NiriStreamError> {
        var_os(socket::SOCKET_PATH_ENV).ok_or(NiriStreamError::NiriNoSocket)
    }

    pub async fn connect() -> Result<BufReader<UnixStream>, NiriStreamError> {
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

        serde_json::from_str::<NiriEvent>(&line).map_err(NiriStreamError::from)
    }
}

pub fn listen() -> impl Stream<Item = Result<NiriEvent, NiriStreamError>> {
    let eventstream = NiriStream::Disconnected { attempts: 0 };
    stream::unfold(eventstream, |es| async move {
        match es {
            NiriStream::Disconnected { attempts } => {
                if attempts >= 4 {
                    return None;
                }

                if attempts > 0 {
                    let duration = Duration::from_secs(2_u64.pow(attempts));
                    tokio::time::sleep(duration).await;
                }

                let mut reader = match NiriStream::connect().await {
                    Ok(reader) => reader,
                    Err(err) => {
                        return Some((
                            Err(err),
                            NiriStream::Disconnected {
                                attempts: attempts + 1,
                            },
                        ));
                    }
                };

                let event = match NiriStream::read(&mut reader).await {
                    Ok(event) => event,
                    Err(err) => {
                        return Some((
                            Err(err),
                            NiriStream::Disconnected {
                                attempts: attempts + 1,
                            },
                        ));
                    }
                };

                Some((Ok(event), NiriStream::Connected(reader)))
            }
            NiriStream::Connected(mut reader) => {
                let event = match NiriStream::read(&mut reader).await {
                    Ok(event) => event,
                    Err(err) => return Some((Err(err), NiriStream::Disconnected { attempts: 1 })),
                };

                Some((Ok(event), NiriStream::Connected(reader)))
            }
        }
    })
}
