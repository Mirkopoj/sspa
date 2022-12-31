use rppal::spi::*;
use std::thread::sleep;
use std::time::Duration;

/* SPI CON EL PIC */
pub async fn spi_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;5]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1000000, Mode::Mode0)
        .expect("Falló abrir spi");
    let mut buffer = [0;2];

    loop {
        let msg = rx.recv().await.unwrap();
        
        spi.transfer(&mut buffer, &msg[1..3]).unwrap();
        if verbose { 
            println!("Spi sent: {:02X}{:02X}", msg[1], msg[2]); 
            println!("Spi got: {:02X}{:02X}", buffer[0], buffer[1]); 
        }
        if msg[0] >= 2 {
            spi.transfer(&mut buffer, &msg[3..5]).unwrap();
            if verbose { 
                println!("Spi sent: {:02X}{:02X}", msg[3], msg[4]); 
                println!("Spi got: {:02X}{:02X}", buffer[0], buffer[1]); 
            }
        }
        sleep(Duration::from_millis(50));
        spi.transfer(&mut buffer, &[0;2]).unwrap();
        if verbose { 
            println!("Spi sent: 0"); 
            println!("Spi got: {:02X}{:02X}", buffer[0], buffer[1]); 
        }

        tx.send(buffer).unwrap();
    }
}

pub async fn spi_read(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;5]>
) -> [u8;2] {
    spi_core(1, msg, rx, tx).await
}

pub async fn spi_write(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;5]>
) -> [u8;2] {
    spi_core(2, msg, rx, tx).await
}

async fn spi_core(
    len: u8,
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;5]>
) -> [u8;2] {
    let msg = parity_set(msg);
    let mut arr = [len;5];
    arr[1..].clone_from_slice(&msg.to_be_bytes());

    tx.send(arr).await.unwrap();
    
    rx.recv().await.unwrap()
}

fn parity_set(dato: u32) -> u32 {
    let msgh = dato & 0xFFFF0000;
    let msgl = dato & 0x0000FFFF;
    let bith = (msgh.count_ones()%2) << 31;
    let bitl = (msgl.count_ones()%2) << 15;
    dato | bith | bitl
}

/* SPI CON EL DAC */
pub async fn dac_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;3]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss1, 1000000, Mode::Mode0)
        .expect("Falló abrir dac");
    let mut buffer = [0;3];
    let mut respuesta= [0;2];

    loop {
        let mut msg = rx.recv().await.unwrap();

        if msg[0]&0xF0 > 0x70 {
            if verbose { println!("Address out of range"); }
            tx.send([0xF0,0xF0]).unwrap(); 
        }
        
        spi.transfer(&mut buffer, &msg).unwrap();
        if verbose { 
            println!("Spi sent: {:02X}{:02X}{:02X}", msg[0], msg[1], msg[2]); 
            println!("Spi got: {:02X}{:02X}{:02X}", buffer[0], buffer[1], buffer[2]); 
        }
        sleep(Duration::from_millis(50));
        if msg[0]&0x0C == 0 {
            msg[0] |= 0xC;
            spi.transfer(&mut buffer, &msg).unwrap();
            if verbose { 
                println!("Spi sent: {:02X}{:02X}{:02X}", msg[0], msg[1], msg[2]); 
                println!("Spi got: {:02X}{:02X}{:02X}", buffer[0], buffer[1], buffer[2]); 
            }
        }

        respuesta.clone_from_slice(&buffer[1..]);
        //000000dddddddddd <- Respuesta valida
        //100000dddddddddd <- Respuesta invalida
        respuesta[0] |= !buffer[0].reverse_bits();
        tx.send(respuesta).unwrap();
    }
}

pub async fn dac_read(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;3]>
) -> [u8;2] {
    dac_core(0xC, msg, rx, tx).await
}

pub async fn dac_write(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;3]>
) -> [u8;2] {
    dac_core(0x0, msg, rx, tx).await
}

async fn dac_core(
    wr: u8,
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;3]>
) -> [u8;2] {
    let mut arr = [0;3];
    let msg = msg & 0x000F03FF;
    arr.clone_from_slice(&msg.to_be_bytes()[1..]);
    arr[1] <<= 4;
    arr[1] |= wr;

    tx.send(arr).await.unwrap();
    
    rx.recv().await.unwrap()
}
