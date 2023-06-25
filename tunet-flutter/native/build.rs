use lib_flutter_rust_bridge_codegen::{
    config_parse, frb_codegen, get_symbols_if_no_duplicates, init_logger, RawOpts,
};

const RUST_INPUT: &str = "src/api.rs";

fn main() {
    init_logger("../logs/", true).unwrap();

    println!("cargo:rerun-if-changed={RUST_INPUT}");

    let raw_opts = RawOpts::try_parse_args_or_yaml().unwrap();

    let all_configs = config_parse(raw_opts);

    let all_symbols = get_symbols_if_no_duplicates(&all_configs).unwrap();
    assert_eq!(all_configs.len(), 1);
    frb_codegen(&all_configs[0], &all_symbols).unwrap();
}
