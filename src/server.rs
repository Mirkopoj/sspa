use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt,AsyncReadExt};
use std::process::Command;

use crate::spi::{spi_read, spi_write, dac_read, dac_write};
use crate::tnr::tnr;

pub async fn run(
    verbose: bool,
    quiet: bool,
    port: &str,
    spi_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    spi_tx: tokio::sync::mpsc::Sender<[u8;5]>,
    dac_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    dac_tx: tokio::sync::mpsc::Sender<[u8;3]>,
    tnr_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    tnr_tx: tokio::sync::mpsc::Sender<[u8;4]>
) {
    if verbose { println!("Server starting"); }

    let listener = TcpListener::bind("0.0.0.0:".to_string()+port).await.unwrap();

    if !quiet { println!("Server listening {}", listener.local_addr().unwrap()); }
    
    if verbose { println!("Server starting"); }
    iniciar_gpiod();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let spi_rx_clone =  spi_rx.resubscribe();
        let spi_tx_clone =  spi_tx.clone();
        let dac_rx_clone =  dac_rx.resubscribe();
        let dac_tx_clone =  dac_tx.clone();
        let tnr_rx_clone =  tnr_rx.resubscribe();
        let tnr_tx_clone =  tnr_tx.clone();

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
        		tnr_tx_clone
            ).await;
        });
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    verbose: bool,
    quiet: bool,
    mut spi_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    spi_tx: tokio::sync::mpsc::Sender<[u8;5]>,
    mut dac_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    dac_tx: tokio::sync::mpsc::Sender<[u8;3]>,
    mut tnr_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    tnr_tx: tokio::sync::mpsc::Sender<[u8;4]>
) {
    loop {
        let mut buffer = [0; 4];

        let n_bytes = socket.read(&mut buffer).await.unwrap();

        if n_bytes != 4 { 
            //if verbose { println!("Number of bytes is not 4"); }
            continue; 
        }

        let mensaje = <u32>::from_be_bytes(buffer);
        if !quiet { println!("Received: {:X}", mensaje); }

        let respuesta = match mensaje & 0x7F000000 {
            0x32000000 => { Some(spi_read(mensaje, &mut spi_rx, &spi_tx).await) },
            0x25000000 => { Some(spi_write(mensaje, &mut spi_rx, &spi_tx).await) },
            0x3A000000 => { Some(dac_read(mensaje, &mut dac_rx, &dac_tx).await) },
            0x2A000000 => { Some(dac_write(mensaje, &mut dac_rx, &dac_tx).await) },
            0x33000000 | 0x23000000 | 0xA3000000 => { Some(tnr(mensaje, &mut tnr_rx, &tnr_tx).await) },
            _ => { 
                if verbose { println!("Invalid Command"); }
                None 
            }
        };

        if let Some(valor) = respuesta {
            let _ = socket.write_all(&valor).await;
            if !quiet { println!("Sent: {:X}", <u16>::from_be_bytes(valor)); }
        }
    }
}

fn iniciar_gpiod(){
    let mut child = Command::new("sudo")
            .arg("gpiod")
            .spawn()
            .expect("failed to launch gpiod");

    child.wait().expect("Failed to wait on gpiod");
}
