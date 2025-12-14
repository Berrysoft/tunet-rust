use crate::frb_generated::StreamSink;
use flutter_rust_bridge::{frb, setup_default_user_utils};

use anyhow::Result;
use futures_util::StreamExt;
pub use netstatus::NetStatus;
use std::convert::Infallible;
pub use std::sync::Mutex;
pub use tunet_helper::{
    Balance, Duration as NewDuration, Flux, NaiveDateTime, NaiveDuration as Duration, NetFlux,
    NetState,
};
pub use tunet_model::{Action, Model, UpdateMsg};
use winio_elm::{Child, Component, ComponentSender, Root, RunEvent};

pub enum UpdateMsgWrap {
    Credential(String),
    State(NetState),
    Status(String),
    Log(String),
    Flux(NetFlux),
    LogBusy(bool),
}

#[frb(mirror(NetState))]
pub enum _NetState {
    Unknown,
    Auth4,
    Auth6,
}

#[frb(mirror(NetStatus))]
pub enum _NetStatus {
    Unknown,
    Wwan,
    Wlan(String),
    Lan,
}

#[frb(mirror(NetFlux))]
pub struct _NetFlux {
    pub username: String,
    pub flux: Flux,
    pub online_time: NewDuration,
    pub balance: Balance,
}

#[frb(mirror(Flux))]
pub struct _Flux(pub u64);

#[frb(mirror(NewDuration))]
pub struct _NewDuration(pub Duration);

#[frb(mirror(Balance))]
pub struct _Balance(pub f64);

#[frb(mirror(NetDateTime))]
pub struct _NetDateTime(pub NaiveDateTime);

pub struct RuntimeStartConfig {
    pub status: NetStatus,
    pub username: String,
    pub password: String,
}

struct ModelWrapper {
    model: Child<Model>,
    sink: StreamSink<UpdateMsgWrap>,
}

enum ModelWrapperMessage {
    Noop,
    Msg(UpdateMsg),
    Post(Action),
}

impl Component for ModelWrapper {
    type Error = anyhow::Error;
    type Init<'a> = StreamSink<UpdateMsgWrap>;
    type Message = ModelWrapperMessage;
    type Event = Infallible;

    async fn init(sink: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let model = Child::<Model>::init(()).await?;
        Ok(Self { model, sink })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        self.model
            .start(
                sender,
                |msg| Some(ModelWrapperMessage::Msg(msg)),
                || ModelWrapperMessage::Noop,
            )
            .await
    }

    async fn update_children(&mut self) -> Result<bool> {
        self.model.update().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ModelWrapperMessage::Noop => {}
            ModelWrapperMessage::Msg(msg) => {
                let msg = match msg {
                    UpdateMsg::Credential => UpdateMsgWrap::Credential(self.model.username.clone()),
                    UpdateMsg::State => UpdateMsgWrap::State(self.model.state),
                    UpdateMsg::Status => UpdateMsgWrap::Status(self.model.status.to_string()),
                    UpdateMsg::Log => UpdateMsgWrap::Log(self.model.log.to_string()),
                    UpdateMsg::Flux => UpdateMsgWrap::Flux(self.model.flux.clone()),
                    UpdateMsg::LogBusy => UpdateMsgWrap::LogBusy(self.model.log_busy()),
                };
                self.sink.add(msg).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
            ModelWrapperMessage::Post(a) => {
                self.model.post(a);
            }
        }
        Ok(false)
    }

    fn render_children(&mut self) -> Result<()> {
        self.model.render()
    }
}

pub struct Runtime {
    sender: Mutex<Option<ComponentSender<ModelWrapper>>>,
}

impl Runtime {
    #[frb(sync)]
    pub fn new() -> Runtime {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Trace)
                .with_filter(
                    android_logger::FilterBuilder::new()
                        .parse("warn,tunet=trace,native=trace")
                        .build(),
                ),
        );
        setup_default_user_utils();

        Self {
            sender: Mutex::new(None),
        }
    }

    pub async fn start(&self, sink: StreamSink<UpdateMsgWrap>, config: RuntimeStartConfig) {
        {
            if (!config.username.is_empty()) && (!config.password.is_empty()) {
                self.queue(Action::Credential(config.username, config.password));
            }
            self.queue(Action::Status(Some(config.status)));
            self.queue(Action::Timer);
        }
        let (tx, rx) = futures_channel::oneshot::channel();
        std::thread::spawn(move || {
            let runtime = compio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let mut model = Root::<ModelWrapper>::init(sink).await.unwrap();
                tx.send(model.sender().clone()).ok();
                let stream = model.run();
                let mut stream = std::pin::pin!(stream);
                while let Some(event) = stream.next().await {
                    match event {
                        RunEvent::Event(e) => match e {},
                        RunEvent::UpdateErr(e) => {
                            log::error!("Update error: {:?}", e);
                        }
                        RunEvent::RenderErr(e) => {
                            log::error!("Render error: {:?}", e);
                        }
                        _ => {
                            log::warn!("Unknown event: {:?}", event);
                        }
                    }
                }
            });
            unreachable!("model ended unexpectedly");
        });
        let sender = rx.await.unwrap();
        self.sender.lock().unwrap().replace(sender);
    }

    fn queue(&self, a: Action) {
        let sender = self.sender.lock().unwrap();
        if let Some(sender) = sender.as_ref() {
            sender.post(ModelWrapperMessage::Post(a));
        }
    }

    #[frb(sync)]
    pub fn queue_credential(&self, u: String, p: String) {
        self.queue(Action::Credential(u, p));
    }

    #[frb(sync)]
    pub fn queue_login(&self) {
        self.queue(Action::Login);
    }

    #[frb(sync)]
    pub fn queue_logout(&self) {
        self.queue(Action::Logout);
    }

    #[frb(sync)]
    pub fn queue_flux(&self) {
        self.queue(Action::Flux);
    }

    #[frb(sync)]
    pub fn queue_state(&self, s: Option<NetState>) {
        self.queue(Action::State(s));
    }

    #[frb(sync)]
    pub fn queue_status(&self, s: NetStatus) {
        self.queue(Action::Status(Some(s)));
    }
}
