use anyhow::Result;
use flutter_rust_bridge::{frb, RustOpaque, StreamSink};

pub use std::sync::{Arc, Mutex};
pub use tokio::{runtime::Handle, sync::mpsc};
pub use tunet_helper::{
    Balance, Duration as NewDuration, Flux, NaiveDuration as Duration, NetFlux, NetState,
};
pub use tunet_model::{Action, Model, UpdateMsg};

#[frb(mirror(UpdateMsg))]
pub enum _UpdateMsg {
    Credential,
    State,
    Status,
    Log,
    Flux,
    Online,
    Details,
    LogBusy,
    OnlineBusy,
    DetailBusy,
}

pub struct UpdateMsgWrap(pub UpdateMsg);

#[frb(mirror(NetState))]
pub enum _NetState {
    Unknown,
    Net,
    Auth4,
    Auth6,
}

pub struct NetStateWrap(pub NetState);

#[frb(mirror(NetFlux))]
pub struct _NetFlux {
    pub username: String,
    pub flux: Flux,
    pub online_time: NewDuration,
    pub balance: Balance,
}

#[frb(mirror(Flux))]
pub struct _Flux(pub u64);

pub fn flux_to_string(f: u64) -> String {
    Flux(f).to_string()
}

#[frb(mirror(NewDuration))]
pub struct _NewDuration(pub Duration);

#[frb(mirror(Balance))]
pub struct _Balance(pub f64);

pub struct Runtime {
    pub rx: RustOpaque<Mutex<Option<mpsc::Receiver<Action>>>>,
    pub model: RustOpaque<Mutex<Model>>,
    pub handle: RustOpaque<Mutex<Option<Handle>>>,
}

impl Runtime {
    pub fn new() -> Result<Runtime> {
        #[cfg(target_os = "android")]
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
        );
        #[cfg(target_os = "ios")]
        oslog::OsLogger::new("com.berrysoft.tunet.flutter")
            .level_filter(log::LevelFilter::Trace)
            .init()?;

        let (tx, rx) = mpsc::channel(32);
        let model = Model::new(tx)?;
        Ok(Self {
            rx: RustOpaque::new(Mutex::new(Some(rx))),
            model: RustOpaque::new(Mutex::new(model)),
            handle: RustOpaque::new(Mutex::new(None)),
        })
    }

    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>) {
        let model = self.model.clone();
        let mut rx = self.rx.lock().unwrap().take().unwrap();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let handle = runtime.handle().clone();
        *self.handle.lock().unwrap() = Some(handle);
        std::thread::spawn(move || {
            runtime.block_on(async {
                {
                    let mut model = model.lock().unwrap();
                    model.update = Some(Box::new(move |msg| {
                        sink.add(UpdateMsgWrap(msg));
                    }));
                    model.queue(Action::Status);
                    model.queue(Action::Timer);
                }
                while let Some(action) = rx.recv().await {
                    log::info!("[tunet-flutter/native] received action: {:?}", action);
                    model.lock().unwrap().handle(action);
                }
            });
        });
    }

    fn queue(&self, a: Action) {
        let _guard = self.handle.lock().unwrap().as_ref().unwrap().enter();
        self.model.lock().unwrap().queue(a);
    }

    pub fn queue_login(&self) {
        self.queue(Action::Login);
    }

    pub fn queue_logout(&self) {
        self.queue(Action::Logout);
    }

    pub fn queue_flux(&self) {
        self.queue(Action::Flux);
    }

    pub fn queue_state(&self, s: Option<NetStateWrap>) {
        self.queue(Action::State(s.map(|s| s.0)))
    }

    pub fn log_busy(&self) -> bool {
        self.model.lock().unwrap().log_busy()
    }

    pub fn flux(&self) -> NetFlux {
        self.model.lock().unwrap().flux.clone()
    }

    pub fn state(&self) -> NetStateWrap {
        NetStateWrap(self.model.lock().unwrap().state)
    }

    pub fn status(&self) -> String {
        self.model.lock().unwrap().status.to_string()
    }
}
