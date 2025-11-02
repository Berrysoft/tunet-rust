use crate::frb_generated::StreamSink;
use flutter_rust_bridge::{frb, setup_default_user_utils};

use futures_util::StreamExt;
pub use netstatus::NetStatus;
use std::sync::Arc;
pub use std::sync::Mutex;
pub use tunet_helper::{
    Balance, Duration as NewDuration, Flux, NaiveDateTime, NaiveDuration as Duration, NetFlux,
    NetState,
};
pub use tunet_model::{Action, Model, UpdateMsg};
use winio_elm::{Child, Component, ComponentSender};

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
    sink: Option<StreamSink<UpdateMsgWrap>>,
}

enum ModelWrapperMessage {
    Noop,
    Msg(UpdateMsg),
    Post(Action),
}

impl Component for ModelWrapper {
    type Init<'a> = ();
    type Message = ModelWrapperMessage;
    type Event = ();

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let model = Child::<Model>::init(());
        Self { model, sink: None }
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

    async fn update_children(&mut self) -> bool {
        self.model.update().await
    }

    async fn update(&mut self, message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        match message {
            ModelWrapperMessage::Noop => false,
            ModelWrapperMessage::Msg(msg) => {
                if let Some(sink) = &self.sink {
                    let msg = {
                        match msg {
                            UpdateMsg::Credential => {
                                UpdateMsgWrap::Credential(self.model.username.clone())
                            }
                            UpdateMsg::State => UpdateMsgWrap::State(self.model.state),
                            UpdateMsg::Status => {
                                UpdateMsgWrap::Status(self.model.status.to_string())
                            }
                            UpdateMsg::Log => UpdateMsgWrap::Log(self.model.log.to_string()),
                            UpdateMsg::Flux => UpdateMsgWrap::Flux(self.model.flux.clone()),
                            UpdateMsg::LogBusy => UpdateMsgWrap::LogBusy(self.model.log_busy()),
                        }
                    };
                    sink.add(msg).ok();
                }
                false
            }
            ModelWrapperMessage::Post(a) => {
                self.model.post(a);
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}

    fn render_children(&mut self) {
        self.model.render();
    }
}

pub struct Runtime {
    model: Arc<Mutex<Child<ModelWrapper>>>,
    sender: ComponentSender<ModelWrapper>,
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

        let model = Child::<ModelWrapper>::init(());
        let sender = model.sender().clone();
        Self {
            model: Arc::new(Mutex::new(model)),
            sender,
        }
    }

    #[frb(sync)]
    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>, config: RuntimeStartConfig) {
        {
            if (!config.username.is_empty()) && (!config.password.is_empty()) {
                self.queue(Action::Credential(config.username, config.password));
            }
            self.queue(Action::Status(Some(config.status)));
            self.queue(Action::Timer);
        }
        let model = self.model.clone();
        std::thread::spawn(move || {
            let runtime = compio::runtime::RuntimeBuilder::new().build().unwrap();
            runtime.block_on(async {
                let mut model = model.lock().unwrap();
                model.sink = Some(sink);
                let stream = model.run();
                let mut stream = std::pin::pin!(stream);
                stream.next().await;
            });
        });
    }

    fn queue(&self, a: Action) {
        self.sender.post(ModelWrapperMessage::Post(a));
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
