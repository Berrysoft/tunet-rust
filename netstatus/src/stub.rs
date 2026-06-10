use crate::*;

pub fn current() -> NetStatus {
    NetStatus::Unknown
}

pub fn watch() -> impl Stream<Item = ()> {
    futures_util::stream::pending()
}
