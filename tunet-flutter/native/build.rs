use lib_flutter_rust_bridge_codegen::{
    codegen::{generate, Config, MetaConfig},
    utils::logs::configure_opinionated_logging,
};

const RUST_INPUT: &str = "src/api.rs";

fn main() {
    configure_opinionated_logging("../logs/", true).unwrap();

    println!("cargo:rerun-if-changed={RUST_INPUT}");
    println!("cargo:rerun-if-changed=.flutter_rust_bridge.yml");

    let config = Config::from_files_auto().unwrap();
    let meta_config = MetaConfig::default();

    generate(config, meta_config).unwrap();
}
