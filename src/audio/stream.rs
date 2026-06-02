use std::ops::{Div, Mul};
use std::thread;
use std::time::Duration;
use std::{ffi::CString, io::BufReader, os::unix::net::UnixStream};

use anyhow::{Error, Result, anyhow};
use iced::futures::stream;
use pulseaudio::protocol::{self, CommandReply, SubscriptionMask};
use tokio::fs;
use tokio::sync::mpsc;
use tokio_stream::Stream;
use tracing::info;

fn get_sock() -> Result<BufReader<UnixStream>> {
    let socket_path =
        pulseaudio::socket_path_from_env().ok_or(anyhow!("pulseAudio not available"))?;
    let sock = BufReader::new(UnixStream::connect(socket_path)?);
    Ok(sock)
}

async fn get_auth() -> Result<protocol::AuthParams> {
    // PulseAudio usually puts an authentication "cookie" in ~/.config/pulse/cookie.
    // PulseAudio is a per user service, cookie here auths users
    let path = pulseaudio::cookie_path_from_env().ok_or(anyhow!("No cookie found"))?;
    let cookie = fs::read(path).await?;

    Ok(protocol::AuthParams {
        version: protocol::MAX_VERSION,
        supports_shm: false,
        supports_memfd: false,
        cookie,
    })
}

fn request<T: CommandReply>(
    sock: &mut BufReader<UnixStream>,
    seq: u32,
    command: &protocol::Command,
    ver: u16,
) -> Result<(u32, T)> {
    protocol::write_command_message(sock.get_mut(), seq, command, ver)?;
    let res = protocol::read_reply_message::<T>(sock, ver)?;
    Ok(res)
}

#[derive(Debug, Clone)]
pub enum AudioEvents {
    Connected,
    Vol(u32),
    None,
}

struct PulseSock {
    sock: BufReader<UnixStream>,
    ver: u16,
}

impl PulseSock {
    fn command<T: CommandReply>(
        &mut self,
        seq: u32,
        command: &protocol::Command,
    ) -> Result<(u32, T)> {
        request(&mut self.sock, seq, command, self.ver)
    }
    fn get_server_info(&mut self, seq: u32) -> Result<(u32, protocol::ServerInfo)> {
        let res = request::<protocol::ServerInfo>(
            &mut self.sock,
            seq,
            &protocol::Command::GetServerInfo,
            self.ver,
        )?;
        Ok(res)
    }
    fn get_sink_info(&mut self, seq: u32, name: CString) -> Result<(u32, protocol::SinkInfo)> {
        let res = request::<protocol::SinkInfo>(
            &mut self.sock,
            seq,
            &protocol::Command::GetSinkInfo(protocol::GetSinkInfo {
                index: None,
                name: Some(name),
            }),
            self.ver,
        )?;
        Ok(res)
    }
}

enum StreamState {
    Disconnected { attempts: u32 },
    Connected { pulse: PulseSock },
}

macro_rules! try_stream {
    ($expr:expr, $state:expr$(,)?) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Some((Err(Error::from(err)), $state)),
        }
    };
}

macro_rules! ok_stream {
    ($expr:expr, $state:expr$(,)?) => {
        match $expr {
            Some(val) => val,
            None => return Some((Ok(AudioEvents::None), $state)),
        }
    };
}

pub fn listen() -> impl Stream<Item = Result<AudioEvents>> {
    let stream_state = StreamState::Disconnected { attempts: 0 };
    stream::unfold(stream_state, |state| async {
        match state {
            StreamState::Disconnected { attempts } => {
                if attempts >= 4 {
                    return None;
                }

                if attempts > 0 {
                    let duration = Duration::from_secs(2_u64.pow(attempts));
                    tokio::time::sleep(duration).await;
                }

                let sock = try_stream!(
                    get_sock(),
                    StreamState::Disconnected {
                        attempts: attempts + 1,
                    }
                );

                let auth = try_stream!(
                    get_auth().await,
                    StreamState::Disconnected {
                        attempts: attempts + 1,
                    }
                );

                let mut pulse = PulseSock {
                    sock,
                    ver: protocol::MAX_VERSION,
                };

                let (_, auth_reply) = try_stream!(
                    pulse.command::<protocol::AuthReply>(0, &protocol::Command::Auth(auth)),
                    StreamState::Disconnected {
                        attempts: attempts + 1,
                    }
                );

                pulse.ver = auth_reply.version;

                // Setup client
                let mut props = protocol::Props::new();
                let name = try_stream!(
                    CString::new("icedshell"),
                    StreamState::Disconnected {
                        attempts: attempts + 1,
                    }
                );

                props.set(protocol::Prop::ApplicationName, name);

                // Do we need client id?
                let _ = try_stream!(
                    pulse.command::<protocol::SetClientNameReply>(
                        1,
                        &protocol::Command::SetClientName(props)
                    ),
                    StreamState::Disconnected {
                        attempts: attempts + 1,
                    }
                );

                Some((Ok(AudioEvents::Connected), StreamState::Connected { pulse }))
            }
            StreamState::Connected { mut pulse } => {
                let (_, server_info) =
                    try_stream!(pulse.get_server_info(0), StreamState::Connected { pulse },);

                tokio::time::sleep(Duration::from_secs(1)).await;

                let sink_name = ok_stream!(
                    server_info.default_sink_name,
                    StreamState::Connected { pulse },
                );

                let (_, sink) = try_stream!(
                    pulse.get_sink_info(1, sink_name),
                    StreamState::Connected { pulse },
                );

                let vol = ok_stream!(
                    sink.cvolume.channels().iter().next(),
                    StreamState::Connected { pulse },
                );

                let vol = vol.as_u32() as f32;
                let vol = vol.div(protocol::Volume::NORM.as_u32() as f32).mul(100.);

                Some((
                    Ok(AudioEvents::Vol(vol.round_ties_even() as u32)),
                    StreamState::Connected { pulse },
                ))
            }
        }
    })
}
