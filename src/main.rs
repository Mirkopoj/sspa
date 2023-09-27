use std::env;
use std::process::Command;
use tokio::sync::{broadcast, mpsc};

extern crate unicode_segmentation;

const VERSION: &str = "v0.8.0";

const CARGOPATH: &str = "/opt/sspa";

mod tnr;
use tnr::tnr_handler;

mod spi;
use spi::spi_handler;

mod dac;
use dac::dac_handler;

mod server;
use server::run;

mod relay;
use relay::relay_handler;

mod tnr_monitor;
use tnr_monitor::monitor_handler;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut verbose = false;
    let mut quiet = false;
    let mut quit = false;
    let mut port = "8000";
    let mut little_endian = false;
    let mut hat = false;
    let mut mega_hertz = false;

    let mut arg = args.iter().peekable();
    arg.next();
    while let Some(option) = arg.next() {
        match option.as_str() {
            "-u" | "--update" => {
                actualizar();
                quit = true;
                break;
            }
            "-h" | "--help" => {
                imprimir_ayuda();
                quit = true;
                break;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "-q" | "--quiet" => {
                quiet = true;
            }
            "-l" | "--little-endian" => {
                little_endian = true;
            }
            "-H" | "--hat" => {
                hat = true;
            }
            "-M" | "--mega-hertz" => {
                mega_hertz = true;
            }
            "-p" | "--port" => {
                port = match arg.next_if(|&x| x.parse::<u16>().is_ok()) {
                    Some(n) => n,
                    None => port,
                }
            }
            "-V" | "--version" => {
                println!("{}", VERSION);
                quit = true;
                break;
            }
            _ => {
                println!("Invalid Argument: {}", option);
                quit = true;
                break;
            }
        }
    }

    if quiet {
        verbose = false;
    }

    if !quit {
        let (spi_tx, rx_spi) = mpsc::channel(16);
        let (tx_spi, spi_rx) = broadcast::channel(16);

        let (dac_tx, rx_dac) = mpsc::channel(16);
        let (tx_dac, dac_rx) = broadcast::channel(16);

        let (tnr_tx, rx_tnr) = mpsc::channel(16);
        let (tx_tnr, tnr_rx) = broadcast::channel(16);

        let (reset_relay_tx, rx_reset_relay) = mpsc::channel(16);
        let (tx_reset_relay, reset_relay_rx) = broadcast::channel(16);

        let (program_relay_tx, rx_program_relay) = mpsc::channel(16);
        let (tx_program_relay, program_relay_rx) = broadcast::channel(16);

        let (monitor_tx, rx_monitor) = mpsc::channel(16);
        let (tx_monitor, monitor_rx) = broadcast::channel(16);

        tokio::spawn(async move {
            spi_handler(verbose, rx_spi, tx_spi, mega_hertz).await;
        });

        tokio::spawn(async move {
            dac_handler(hat, verbose, rx_dac, tx_dac).await;
        });

        tokio::spawn(async move {
            tnr_handler(verbose, rx_tnr, tx_tnr).await;
        });

        tokio::spawn(async move {
            relay_handler(verbose, rx_reset_relay, tx_reset_relay, 12).await;
        });

        tokio::spawn(async move {
            relay_handler(verbose, rx_program_relay, tx_program_relay, 0).await;
        });

        tokio::spawn(async move {
            monitor_handler(verbose, rx_monitor, tx_monitor).await;
        });

        run(
            verbose,
            quiet,
            port,
            spi_rx,
            spi_tx,
            dac_rx,
            dac_tx,
            tnr_rx,
            tnr_tx,
            reset_relay_rx,
            reset_relay_tx,
            program_relay_rx,
            program_relay_tx,
            monitor_rx,
            monitor_tx,
            little_endian,
            hat,
        )
        .await;
    }
}

fn imprimir_ayuda() {
    println!("Automatic board tester");
    println!();
    println!("USAGE:");
    println!("\tsspa");
    println!("\tsspa [OPTION]...");
    println!();
    println!("OPTIONS:");
    println!("\t-h --help\t\tPrints this page and exit");
    println!("\t-u --update\t\tUpdates binaries and exit");
    println!("\t-v --verbose\t\tExplain what is being done");
    println!("\t-q --quiet\t\tDo no log to stdout, will overwrite --verbose");
    println!("\t-l --little-endian\tChange net byte order from BigEndian to LittleEndian");
    println!("\t-H --hat\t\tChange dac functionality to use software pwm as analog out");
    println!("\t-M --mega-hertz\t\tChange spi clock frequency, from 100kHz to 1MHz");
    println!("\t-p --port\t\tEspecify a port for the TCP server to listen at, 8000 by default");
    println!("\t-V --version\t\tPrints version information and exit");
    println!();
    println!("NOTE: you can uninstall the program at any time running:");
    println!("\tsspa_uninstall.sh");
    println!();
}

fn actualizar() {
    let mut child = Command::new("sudo")
        .arg("git")
        .arg("pull")
        .current_dir(CARGOPATH)
        .spawn()
        .expect("failed to execute git pull");

    child.wait().expect("Failed to wait on git pull");

    let mut child = Command::new("cargo")
        .arg("update")
        .current_dir(CARGOPATH)
        .spawn()
        .expect("failed to execute cargo update");

    child.wait().expect("Failed to wait on cargo update");

    let mut child = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(CARGOPATH)
        .spawn()
        .expect("failed to execute cargo build");

    child.wait().expect("Failed to wait on cargo build");

    let mut child = Command::new("sudo")
        .arg("rm")
        .arg("/bin/sspa")
        .spawn()
        .expect("failed to rm sspa");

    child.wait().expect("Failed to wait on rm sspa");

    let mut child = Command::new("sudo")
        .arg("cp")
        .arg(CARGOPATH.to_string() + "/target/release/sspa")
        .arg("/bin/sspa")
        .spawn()
        .expect("failed to add sspa to path");

    child.wait().expect("Failed to wait on cp sspa");
}
