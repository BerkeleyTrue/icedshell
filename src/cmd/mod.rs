use iced::{
    Color, Element, Subscription, Task,
    advanced::graphics::futures::MaybeSend,
    time::{every, seconds},
    widget::text,
};

use crate::feature::CompWithProps;

enum CmdState {
    Output(String),
    Error,
}

async fn run_cmd(cmd: String, args: Vec<String>) -> CmdState {
    tokio::process::Command::new(&cmd)
        .args(&args)
        .output()
        .await
        .ok()
        .map(|o| {
            if o.status.success() {
                CmdState::Output(
                    String::from_utf8(o.stdout)
                        .ok()
                        .map(|o| o.trim().to_string())
                        .unwrap_or_default(),
                )
            } else {
                CmdState::Error
            }
        })
        .unwrap_or(CmdState::Output("".to_string()))
}

pub struct Init {
    pub cmd: String,
    pub args: Vec<String>,
    pub interval: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Output(String),
    Error,
}

pub struct CmdComp {
    cmd: String,
    args: Vec<String>,
    interval: u64,
    output: String,
    is_error: bool,
}

impl CmdComp {
    pub fn output(&self) -> &str {
        &self.output
    }
    pub fn is_error(&self) -> bool {
        self.is_error
    }
}

impl CompWithProps for CmdComp {
    type Message = Message;
    type Init = Init;
    type Props<'a> = Color;

    fn new<O: MaybeSend + 'static>(
        input: Self::Init,
        f: impl Fn(Self::Message) -> O + MaybeSend + 'static,
    ) -> (Self, Task<O>) {
        let Init {
            cmd,
            args,
            interval,
        } = input;
        let comp = Self {
            cmd: cmd.clone(),
            args: args.clone(),
            interval,
            output: String::new(),
            is_error: false,
        };

        let task = Task::perform(run_cmd(cmd, args), move |out| {
            f(match out {
                CmdState::Output(out) => Message::Output(out),
                CmdState::Error => Message::Error,
            })
        });

        (comp, task)
    }

    fn subscription(&self) -> Subscription<Message> {
        every(seconds(self.interval)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error => {
                self.output = "".to_string();
                self.is_error = true;
                Task::none()
            }
            Message::Tick => {
                let cmd = self.cmd.clone();
                let args = self.args.clone();
                Task::perform(run_cmd(cmd, args), |cmd_state| match cmd_state {
                    CmdState::Output(out) => Message::Output(out),
                    CmdState::Error => Message::Error,
                })
            }
            Message::Output(out) => {
                self.output = out;
                self.is_error = false;
                Task::none()
            }
        }
    }

    fn view(&self, color: Color) -> Element<'_, Message> {
        text(self.output.clone()).color(color).into()
    }
}
