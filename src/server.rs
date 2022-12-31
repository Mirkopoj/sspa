use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt,AsyncReadExt};

use crate::spi::{spi_read, spi_write, dac_read, dac_write};

pub async fn run(
    verbose: bool,
    quiet: bool,
    port: &str,
    spi_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    spi_tx: tokio::sync::mpsc::Sender<[u8;5]>,
    dac_rx: tokio::sync::broadcast::Receiver<[u8;2]>,
    dac_tx: tokio::sync::mpsc::Sender<[u8;3]>
) {
    if verbose { println!("Server starting"); }

    let listener = TcpListener::bind("0.0.0.0:".to_string()+port).await.unwrap();

    if !quiet { println!("Server listening {}", listener.local_addr().unwrap()); }

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let spi_rx_clone =  spi_rx.resubscribe();
        let spi_tx_clone =  spi_tx.clone();
        let dac_rx_clone =  dac_rx.resubscribe();
        let dac_tx_clone =  dac_tx.clone();

        tokio::spawn(async move {
            handle_connection(
                socket,
                verbose,
                quiet,
        		spi_rx_clone,
        		spi_tx_clone,
        		dac_rx_clone,
        		dac_tx_clone
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
    dac_tx: tokio::sync::mpsc::Sender<[u8;3]>
) {
    loop {
        let mut buffer = [0; 4];

        let n_bytes = socket.read(&mut buffer).await.unwrap();

        if n_bytes != 4 { 
            if verbose { println!("Number of bytes is not 4"); }
            continue; 
        }

        let mensaje = <u32>::from_be_bytes(buffer);
        if !quiet { println!("Received: {:X}", mensaje); }

        let respuesta = match mensaje & 0x7F000000 {
            0x32000000 => { Some(spi_read(mensaje, &mut spi_rx, &spi_tx).await) },
            0x25000000 => { Some(spi_write(mensaje, &mut spi_rx, &spi_tx).await) },
            0x3A000000 => { Some(dac_read(mensaje, &mut dac_rx, &dac_tx).await) },
            0x2A000000 => { Some(dac_write(mensaje, &mut dac_rx, &dac_tx).await) },
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
