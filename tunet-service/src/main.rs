mod service;

use tunet_helper::Result;

const SERVICE_NAME: &str = "tunet-service";

fn main() -> Result<()> {
    service::start()
}
