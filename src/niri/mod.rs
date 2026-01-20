use std::env::var_os;

use niri_ipc::{Reply, Request, Response, socket};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::UnixStream};

mod types;

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

struct EventStream(UnixStream);

impl EventStream {
    pub async fn connect() -> Result<Self, EventStreamError> {
        let path = var_os(socket::SOCKET_PATH_ENV).ok_or(EventStreamError::NiriNoSocket)?;
        let stream = UnixStream::connect(path).await?;
        Ok(Self(stream))
    }

    pub async fn get_reader(&mut self, req: Request) -> Result<Response, EventStreamError> {
        let req_buff = serde_json::to_string(&req)? + "\n";

        self.0.writable().await?;
        self.0.write_all(req_buff.as_bytes()).await?;
        self.0.shutdown().await?;

        let mut reader = BufReader::new(&mut self.0);

        let mut line = String::new();
        let _ = reader.read_line(&mut line);

        let reply: Reply = serde_json::from_str(&line)?;
        reply.map_err(EventStreamError::NiriStreamRefused)
    }
}
