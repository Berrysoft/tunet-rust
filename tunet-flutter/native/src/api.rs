use anyhow::Result;
use flutter_rust_bridge::{frb, RustOpaque, StreamSink};

pub use std::sync::{Arc, Mutex};
pub use tokio::sync::mpsc;
pub use tunet_helper::NetState;
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

pub struct Runtime {
    pub rx: RustOpaque<Mutex<Option<mpsc::Receiver<Action>>>>,
    pub model: RustOpaque<Mutex<Model>>,
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
        })
    }

    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>) {
        let model = self.model.clone();
        let mut rx = self.rx.lock().unwrap().take().unwrap();
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
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
        self.model.lock().unwrap().queue(Action::Flux);
    }
}
