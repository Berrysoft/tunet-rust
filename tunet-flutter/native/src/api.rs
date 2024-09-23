use crate::frb_generated::StreamSink;
use anyhow::Result;
use chrono::Datelike;
use flutter_rust_bridge::{frb, setup_default_user_utils};

pub use netstatus::NetStatus;
pub use std::{net::Ipv4Addr, sync::Mutex};
use std::{net::Ipv6Addr, sync::Arc};
pub use tunet_helper::{
    usereg::{NetDateTime, NetDetail},
    Balance, Duration as NewDuration, Flux, NaiveDateTime, NaiveDuration as Duration, NetFlux,
    NetState,
};
pub use tunet_model::{Action, DetailDaily, Model, UpdateMsg};

pub enum UpdateMsgWrap {
    Credential(String),
    State(NetState),
    Status(String),
    Log(String),
    Flux(NetFlux),
    Online(Vec<NetUserWrap>),
    Details(Vec<NetDetail>, DetailDailyWrap),
    LogBusy(bool),
    OnlineBusy(bool),
    DetailBusy(bool),
}

#[frb(mirror(NetState))]
pub enum _NetState {
    Unknown,
    Net,
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

#[frb(mirror(NetDetail))]
pub struct _NetDetail {
    pub login_time: NetDateTime,
    pub logout_time: NetDateTime,
    pub flux: Flux,
}

pub struct DetailDailyPoint {
    pub day: u32,
    pub flux: Flux,
}

pub struct DetailDailyWrap {
    pub details: Vec<DetailDailyPoint>,
    pub now_month: u32,
    pub now_day: u32,
    pub max_flux: Flux,
}

pub struct NetUserWrap {
    pub address: Ipv4AddrWrap,
    pub address_v6: Ipv6AddrWrap,
    pub login_time: NetDateTime,
    pub mac_address: String,
    pub flux: Flux,
    pub is_local: bool,
}

pub struct Ipv4AddrWrap {
    pub octets: [u8; 4],
}

impl From<Ipv4Addr> for Ipv4AddrWrap {
    fn from(value: Ipv4Addr) -> Self {
        Self {
            octets: value.octets(),
        }
    }
}

pub struct Ipv6AddrWrap {
    pub octets: [u8; 16],
}

impl From<Ipv6Addr> for Ipv6AddrWrap {
    fn from(value: Ipv6Addr) -> Self {
        Self {
            octets: value.octets(),
        }
    }
}

pub struct RuntimeStartConfig {
    pub status: NetStatus,
    pub username: String,
    pub password: String,
}

pub struct Runtime {
    arx: Arc<Mutex<Option<flume::Receiver<Action>>>>,
    urx: Arc<Mutex<Option<flume::Receiver<UpdateMsg>>>>,
    model: Arc<Mutex<Model>>,
}

impl Runtime {
    #[frb(sync)]
    pub fn new() -> Result<Runtime> {
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

        let (atx, arx) = flume::unbounded();
        let (utx, urx) = flume::unbounded();
        let model = Model::new(atx, utx)?;
        Ok(Self {
            arx: Arc::new(Mutex::new(Some(arx))),
            urx: Arc::new(Mutex::new(Some(urx))),
            model: Arc::new(Mutex::new(model)),
        })
    }

    #[frb(sync)]
    pub fn start(&self, sink: StreamSink<UpdateMsgWrap>, config: RuntimeStartConfig) {
        let model = self.model.clone();
        let arx = self.arx.lock().unwrap().take().unwrap();
        let urx = self.urx.lock().unwrap().take().unwrap();
        std::thread::spawn(move || {
            let runtime = compio::runtime::RuntimeBuilder::new().build().unwrap();
            runtime.block_on(async {
                {
                    let model = model.clone();
                    compio::runtime::spawn(async move {
                        while let Ok(msg) = urx.recv_async().await {
                            let msg = {
                                let model = model.lock().unwrap();
                                match msg {
                                    UpdateMsg::Credential => {
                                        UpdateMsgWrap::Credential(model.username.clone())
                                    }
                                    UpdateMsg::State => UpdateMsgWrap::State(model.state),
                                    UpdateMsg::Status => {
                                        UpdateMsgWrap::Status(model.status.to_string())
                                    }
                                    UpdateMsg::Log => UpdateMsgWrap::Log(model.log.to_string()),
                                    UpdateMsg::Flux => UpdateMsgWrap::Flux(model.flux.clone()),
                                    UpdateMsg::Online => UpdateMsgWrap::Online(
                                        model
                                            .users
                                            .iter()
                                            .map(|u| NetUserWrap {
                                                address: u.address.into(),
                                                address_v6: u.address_v6.into(),
                                                login_time: u.login_time,
                                                mac_address: u
                                                    .mac_address
                                                    .map(|addr| addr.to_string())
                                                    .unwrap_or_default(),
                                                flux: u.flux,
                                                is_local: model
                                                    .mac_addrs
                                                    .iter()
                                                    .any(|it| Some(it) == u.mac_address.as_ref()),
                                            })
                                            .collect(),
                                    ),
                                    UpdateMsg::Details => {
                                        UpdateMsgWrap::Details(model.details.clone(), {
                                            let data = DetailDaily::new(&model.details);
                                            DetailDailyWrap {
                                                details: data
                                                    .details
                                                    .into_iter()
                                                    .map(|(date, flux)| DetailDailyPoint {
                                                        day: date.day(),
                                                        flux,
                                                    })
                                                    .collect(),
                                                now_month: data.now.month(),
                                                now_day: data.now.day(),
                                                max_flux: data.max_flux,
                                            }
                                        })
                                    }
                                    UpdateMsg::LogBusy => UpdateMsgWrap::LogBusy(model.log_busy()),
                                    UpdateMsg::OnlineBusy => {
                                        UpdateMsgWrap::OnlineBusy(model.online_busy())
                                    }
                                    UpdateMsg::DetailBusy => {
                                        UpdateMsgWrap::DetailBusy(model.detail_busy())
                                    }
                                }
                            };
                            sink.add(msg).unwrap();
                        }
                    })
                    .detach();
                }
                {
                    let model = model.lock().unwrap();
                    if (!config.username.is_empty()) && (!config.password.is_empty()) {
                        model.queue(Action::Credential(config.username, config.password));
                    }
                    model.queue(Action::Status(Some(config.status)));
                    model.queue(Action::Timer);
                }
                while let Ok(action) = arx.recv_async().await {
                    log::debug!("received action: {:?}", action);
                    model.lock().unwrap().handle(action);
                }
            });
        });
    }

    fn queue(&self, a: Action) {
        self.model.lock().unwrap().queue(a);
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

    #[frb(sync)]
    pub fn queue_details(&self) {
        self.queue(Action::Details);
    }

    #[frb(sync)]
    pub fn queue_onlines(&self) {
        self.queue(Action::Online);
    }

    #[frb(sync)]
    pub fn queue_connect(&self, ip: Ipv4AddrWrap) {
        self.queue(Action::Connect(Ipv4Addr::from(ip.octets)));
    }

    #[frb(sync)]
    pub fn queue_drop(&self, ips: Vec<Ipv4AddrWrap>) {
        for ip in ips {
            self.queue(Action::Drop(Ipv4Addr::from(ip.octets)));
        }
    }
}
