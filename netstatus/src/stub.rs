use crate::*;

pub fn current() -> NetStatus {
    NetStatus::Unknown
}

pub fn watch() -> impl Stream<Item = ()> {
    tokio_stream::pending()
}
