use super::*;
use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;

fn can_connect(ipaddr: &str) -> bool {
    let (pinger, results) = match Pinger::new(None, None) {
        Ok((pinger, results)) => (pinger, results),
        Err(e) => panic!("Error creating pinger: {}", e),
    };
    pinger.add_ipaddr(ipaddr);
    pinger.ping_once();
    match results.recv() {
        Ok(result) => match result {
            Idle { addr: _ } => false,
            Receive { addr: _, rtt: _ } => true,
        },
        Err(_) => false,
    }
}

pub fn suggest() -> NetState {
    if can_connect("101.6.4.100") {
        NetState::Auth4
    } else if can_connect("166.111.204.120") {
        NetState::Net
    } else if can_connect("2402:f000:1:414:101:6:4:100") {
        NetState::Auth6
    } else {
        NetState::Unknown
    }
}
