use std::thread::sleep;
use std::time::Duration;

use rppal::spi::*;

const SPI_INTER_TRANSACTION_GAP: Duration = Duration::from_micros(100);

/* SPI CON EL PIC */
pub async fn spi_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8; 5]>,
    tx: tokio::sync::broadcast::Sender<[u8; 2]>,
    mega_hertz: bool,
) {
    let clock_speed = if mega_hertz { 1000000 } else { 100000 };
    let spi =
        Spi::new(Bus::Spi0, SlaveSelect::Ss0, clock_speed, Mode::Mode1).expect("Falló abrir spi");
    let mut buffer = [0; 2];

    loop {
        let msg = rx.recv().await.unwrap();

        spi.transfer(&mut buffer, &msg[1..3]).unwrap();
        sleep(SPI_INTER_TRANSACTION_GAP);
        if verbose {
            println!("Spi sent: {:02X}{:02X}", msg[1], msg[2]);
            println!("Spi got: {:02X}{:02X}", buffer[0], buffer[1]);
        }
        if msg[0] > 1 {
            spi.transfer(&mut buffer, &msg[3..5]).unwrap();
            sleep(SPI_INTER_TRANSACTION_GAP);
            if verbose {
                println!("Spi sent: {:02X}{:02X}", msg[3], msg[4]);
                println!("Spi got: {:02X}{:02X}", buffer[0], buffer[1]);
            }
        }
        if msg[0] > 0 {
            spi.transfer(&mut buffer, &[0; 2]).unwrap();
            sleep(SPI_INTER_TRANSACTION_GAP);
            if verbose {
                println!("Spi sent: 0");
                println!("Spi got: {:02X}{:02X}", buffer[0], buffer[1]);
            }
        }

        tx.send(buffer).unwrap();
    }
}

pub async fn spi_read(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 5]>,
) -> [u8; 2] {
    spi_core(1, msg, rx, tx).await
}

pub async fn spi_write(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 5]>,
) -> [u8; 2] {
    spi_core(2, msg, rx, tx).await
}

pub async fn spi_debug(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 5]>,
) -> [u8; 2] {
    spi_core(0, msg << 16, rx, tx).await
}

pub async fn spi_stress_test(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 5]>,
    verbose: bool,
) -> [u8; 2] {
    let pack_count = msg & 0x0000FFFF;
    if verbose {
        println!("Stress testing with {} packets", pack_count);
    }
    for _ in 0..pack_count {
        spi_core(1, 0, rx, tx).await;
    }
    if verbose {
        println!("Stress testing finished");
    }
    spi_core(1, 0, rx, tx).await
}

async fn spi_core(
    len: u8,
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 5]>,
) -> [u8; 2] {
    let msg = parity_set(msg);
    let mut arr = [len; 5];
    arr[1..].clone_from_slice(&msg.to_be_bytes());

    tx.send(arr).await.unwrap();

    rx.recv().await.unwrap()
}

fn parity_set(dato: u32) -> u32 {
    let msgh = dato & 0xFFFF0000;
    let msgl = dato & 0x0000FFFF;
    let mut ret = dato;
    if msgh.count_ones() % 2 != 0 {
        ret ^= 0x80000000
    }
    if msgl.count_ones() % 2 != 0 {
        ret ^= 0x00008000
    }
    ret
}
