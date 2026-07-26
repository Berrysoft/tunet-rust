use winio::prelude::*;

use crate::MainModel;

pub fn start() -> anyhow::Result<()> {
    App::builder()
        .name("io.github.berrysoft.tunet")
        .build()?
        .block_on(MainModel::run_until_event(None))
}
