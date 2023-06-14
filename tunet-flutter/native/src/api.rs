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
                    model.queue(Action::Timer);
                    model.queue(Action::State(Some(NetState::Auth4)));
                }
                while let Some(action) = rx.recv().await {
                    log::info!("[tunet-flutter/native] received action: {:?}", action);
                    model.lock().unwrap().handle(action);
                }
            });
        });
    }

    pub fn queue_flux(&self) {
        let _guard = self.handle.lock().unwrap().as_ref().unwrap().enter();
        self.model.lock().unwrap().queue(Action::Flux);
    }

    pub fn flux(&self) -> NetFlux {
        self.model.lock().unwrap().flux.clone()
    }

    pub fn state(&self) -> NetStateWrap {
        NetStateWrap(self.model.lock().unwrap().state)
    }
}
