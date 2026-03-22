use std::path::PathBuf;

use derive_more::{Deref, DerefMut, From};
use iced::futures::{
    StreamExt, TryStreamExt,
    stream::{self, BoxStream},
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};
use tokio_stream::wrappers::{LinesStream, UnixListenerStream};
use tracing::info;

use crate::osd::OsdCommand;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Request {
    Launcher,
    Osd(OsdCommand),
    PowerMenu,
}

#[derive(Deref, DerefMut, From)]
pub struct IcedSocket(pub BoxStream<'static, anyhow::Result<Request>>);

impl Request {
    fn to_string_line(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self)? + "\n")
    }

    fn from_string_line(line: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str::<Self>(line)?)
    }
}

pub fn get_path() -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(std::env::var("XDG_RUNTIME_DIR")?).join("icedshell.sock"))
}

pub fn connect_and_launch() -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let path = get_path()?;
        let mut stream = UnixStream::connect(path).await?;
        let req = Request::Launcher.to_string_line()?;

        stream.writable().await?;
        stream.write_all(req.as_bytes()).await?;
        stream.shutdown().await?;
        Ok(())
    })
}

pub fn connect_and_osd(args: OsdCommand) -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let path = get_path()?;
        let mut stream = UnixStream::connect(path).await?;
        let req = Request::Osd(args).to_string_line()?;

        stream.writable().await?;
        stream.write_all(req.as_bytes()).await?;
        stream.shutdown().await?;
        Ok(())
    })
}

pub fn connect_and_powermenu() -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let path = get_path()?;
        let mut stream = UnixStream::connect(path).await?;
        let req = Request::PowerMenu.to_string_line()?;

        stream.writable().await?;
        stream.write_all(req.as_bytes()).await?;
        stream.shutdown().await?;
        Ok(())
    })
}

pub fn listen() -> IcedSocket {
    get_path()
        .and_then(|path| {
            UnixListener::bind(&path).or_else(|err| match err.kind() {
                std::io::ErrorKind::AddrInUse => {
                    info!("clearing stale socket");
                    std::fs::remove_file(&path)?;
                    UnixListener::bind(&path).map_err(anyhow::Error::from)
                }
                _ => Err(anyhow::Error::from(err)),
            })
        })
        .map(|listener| {
            UnixListenerStream::new(listener)
                .flat_map(|client_stream_res| match client_stream_res {
                    Ok(stream) => LinesStream::new(BufReader::new(stream).lines())
                        .map(|res| res.map_err(anyhow::Error::from))
                        .boxed(),
                    Err(err) => stream::once(async move { Err(anyhow::Error::from(err)) }).boxed(),
                })
                .and_then(|line| async move { Request::from_string_line(&line) })
                .boxed()
        })
        .unwrap_or_else(|err| stream::once(async move { Err(err) }).boxed())
        .into()
}
