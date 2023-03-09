use crate::CARGOPATH;

const BLINK: &str = "blink.hex";

pub fn run(_verbose: bool, _quiet: bool, _log: bool, _log_path: Option<&String>){
    programar(BLINK);
}

fn programar(path_to_hex: &str){
    let _hex = CARGOPATH.to_string() + path_to_hex;
}

