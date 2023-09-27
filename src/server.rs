use std::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::dac::{dac_read, dac_write};
use crate::relay::relay;
use crate::spi::{spi_debug, spi_read, spi_write, spi_stress_test};
use crate::tnr::tnr;
use crate::tnr_monitor::tnr_monitor;

pub async fn run(
    verbose: bool,
    quiet: bool,
    port: &str,
    spi_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    spi_tx: tokio::sync::mpsc::Sender<[u8; 5]>,
    dac_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    dac_tx: tokio::sync::mpsc::Sender<[u8; 3]>,
    tnr_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    tnr_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    reset_relay_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    reset_relay_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    program_relay_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    program_relay_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    monitor_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    monitor_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    little_endian: bool,
    hat: bool,
) {
    if verbose {
        println!("Server starting");
    }

    let listener = TcpListener::bind("0.0.0.0:".to_string() + port)
        .await
        .unwrap();

    if !quiet {
        println!("Server listening {}", listener.local_addr().unwrap());
    }

    if verbose {
        println!("Server started");
    }
    iniciar_gpiod();

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        if verbose {
            println!("Conection from: {:?}", addr);
        }
        let spi_rx_clone = spi_rx.resubscribe();
        let spi_tx_clone = spi_tx.clone();
        let dac_rx_clone = dac_rx.resubscribe();
        let dac_tx_clone = dac_tx.clone();
        let tnr_rx_clone = tnr_rx.resubscribe();
        let tnr_tx_clone = tnr_tx.clone();
        let reset_relay_rx_clone = reset_relay_rx.resubscribe();
        let reset_relay_tx_clone = reset_relay_tx.clone();
        let program_relay_rx_clone = program_relay_rx.resubscribe();
        let program_relay_tx_clone = program_relay_tx.clone();
        let monitor_rx_clone = monitor_rx.resubscribe();
        let monitor_tx_clone = monitor_tx.clone();

        tokio::spawn(async move {
            handle_connection(
                socket,
                verbose,
                quiet,
                spi_rx_clone,
                spi_tx_clone,
                dac_rx_clone,
                dac_tx_clone,
                tnr_rx_clone,
                tnr_tx_clone,
                reset_relay_rx_clone,
                reset_relay_tx_clone,
                program_relay_rx_clone,
                program_relay_tx_clone,
                monitor_rx_clone,
                monitor_tx_clone,
                little_endian,
                hat,
            )
            .await;
        });
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    verbose: bool,
    quiet: bool,
    mut spi_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    spi_tx: tokio::sync::mpsc::Sender<[u8; 5]>,
    mut dac_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    dac_tx: tokio::sync::mpsc::Sender<[u8; 3]>,
    mut tnr_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    tnr_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    mut reset_relay_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    reset_relay_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    mut program_relay_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    program_relay_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    mut monitor_rx: tokio::sync::broadcast::Receiver<[u8; 2]>,
    monitor_tx: tokio::sync::mpsc::Sender<[u8; 4]>,
    little_endian: bool,
    hat: bool,
) {
    loop {
        let mut buffer = [0; 4];

        let n_bytes = socket.read(&mut buffer).await.unwrap();

        if n_bytes != 4 {
            if n_bytes == 0 {
                break;
            }
            if verbose {
                println!("Number of bytes is not 4, received {} bytes", n_bytes);
            }
            continue;
        }

        let mensaje = if little_endian {
            <u32>::from_le_bytes(buffer)
        } else {
            <u32>::from_be_bytes(buffer)
        };
        if !quiet {
            println!("Received: {:X}", mensaje);
        }

        let respuesta = match mensaje & 0x7F000000 {
            0x3C000000 => Some(spi_read(mensaje, &mut spi_rx, &spi_tx).await),
            0x25000000 => Some(spi_write(mensaje, &mut spi_rx, &spi_tx).await),
            0x5B000000 => Some(spi_debug(mensaje, &mut spi_rx, &spi_tx).await),
            0x3A000000 => Some(dac_read(mensaje, &mut dac_rx, &dac_tx, hat).await),
            0x2A000000 => Some(dac_write(mensaje, &mut dac_rx, &dac_tx, hat).await),
            0x33000000 | 0x23000000 | 0xA3000000 => Some(tnr(mensaje, &mut tnr_rx, &tnr_tx).await),
            0x2D000000 => Some(relay(mensaje, &mut reset_relay_rx, &reset_relay_tx).await),
            0x3D000000 => Some(relay(mensaje, &mut program_relay_rx, &program_relay_tx).await),
            0x4D000000 => Some(tnr_monitor(mensaje, &mut monitor_rx, &monitor_tx).await),
            0x5E000000 => Some(spi_stress_test(mensaje, &mut spi_rx, &spi_tx, verbose).await),
            _ => {
                if verbose {
                    println!("Invalid Command");
                }
                None
            }
        };

        if let Some(valor) = respuesta {
            let valor = invertir(valor, little_endian);
            let _ = socket.write_all(&valor).await;
            if !quiet {
                println!(
                    "Sent: {:X}",
                    if little_endian {
                        <u16>::from_le_bytes(valor)
                    } else {
                        <u16>::from_be_bytes(valor)
                    }
                );
            }
        }
    }
}

fn iniciar_gpiod() {
    let child = Command::new("pidof")
        .arg("pigpiod")
        .output()
        .expect("failed to launch gpiod");

    if !child.stdout.is_empty() {
        return;
    }

    let mut child = Command::new("sudo")
        .arg("pigpiod")
        .spawn()
        .expect("failed to launch gpiod");

    child.wait().expect("Failed to wait on gpiod");
}

fn invertir(valor: [u8; 2], little_endian: bool) -> [u8; 2] {
    let mut ret = valor;
    if little_endian {
        ret[0] = valor[1];
        ret[1] = valor[0];
    }
    ret
}
