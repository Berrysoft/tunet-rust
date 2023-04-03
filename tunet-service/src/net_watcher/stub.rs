use tokio_stream::{wrappers::WatchStream, Stream};
use tunet_helper::{anyhow, Result};

pub fn watch() -> Result<impl Stream<Item = ()>> {
    Err::<WatchStream<()>, _>(anyhow!("不支持的平台"))
}
