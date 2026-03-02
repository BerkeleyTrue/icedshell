use std::path::PathBuf;

use derive_more::{Deref, DerefMut, From};
use iced::futures::{
    StreamExt,
    stream::{BoxStream, empty},
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};
use tokio_stream::wrappers::{LinesStream, UnixListenerStream};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Request {
    Launcher,
}

#[derive(Deref, DerefMut, From)]
pub struct IcedSocket(BoxStream<'static, Request>);

impl Request {
    fn to_string_line(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&Request::Launcher)? + "\n")
    }

    fn from_string_line(line: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str::<Self>(line)?)
    }
}

pub fn get_path() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(std::env::var("XDG_RUNTIME_DIR")?).join("icedshell.sock"))
}

pub async fn connect_and_launch() -> anyhow::Result<()> {
    let path = get_path()?;
    let mut stream = UnixStream::connect(path).await?;
    let req = Request::Launcher.to_string_line()?;

    stream.writable().await?;
    stream.write_all(req.as_bytes()).await?;
    stream.shutdown().await?;

    Ok(())
}

// TODO: map errors to stream items
pub fn listen() -> anyhow::Result<IcedSocket> {
    let path = get_path()?;
    let listener = UnixListener::bind(path)?;

    let stream =
        UnixListenerStream::new(listener).flat_map(|client_stream_res| match client_stream_res {
            Ok(stream) => LinesStream::new(BufReader::new(stream).lines()).boxed(),
            Err(err) => {
                info!("Error opening client stream: {err:?}");
                empty().boxed()
            }
        });

    let stream = tokio_stream::StreamExt::filter_map(stream, |res| {
        res.map_err(anyhow::Error::from)
            .and_then(|req| Request::from_string_line(&req))
            .inspect_err(|err| {
                info!("Error parsing message: {err:?}");
            })
            .ok()
    });

    Ok(stream.boxed().into())
}
