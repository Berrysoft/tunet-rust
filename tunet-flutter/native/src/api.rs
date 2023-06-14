use anyhow::Result;
use flutter_rust_bridge::{frb, RustOpaque, StreamSink};

pub use std::sync::{Arc, Mutex};
pub use tokio::sync::mpsc;
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
        let (tx, rx) = mpsc::channel(32);
        let model = Model::new(tx)?;
        Ok(Self {
            rx: RustOpaque::new(Mutex::new(Some(rx))),
            model: RustOpaque::new(Mutex::new(model)),
        })
    }

    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>) {
        let model = self.model.clone();
        {
            let mut model = model.lock().unwrap();
            model.update = Some(Box::new(move |msg| {
                sink.add(UpdateMsgWrap(msg));
            }));
            model.queue(Action::Status);
            model.queue(Action::Timer);
        }
        let mut rx = self.rx.lock().unwrap().take().unwrap();
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    while let Some(action) = rx.recv().await {
                        model.lock().unwrap().handle(action);
                    }
                });
        });
    }
}
