use rppal::spi::*;
use rppal::gpio::Gpio;
use std::thread::sleep;
use std::time::Duration;

/* SPI CON EL PIC */
pub async fn spi_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;5]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 100000, Mode::Mode1)
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
    hat: bool,
    verbose: bool,
    rx: tokio::sync::mpsc::Receiver<[u8;3]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    if hat {
        pwm_dac_handler(verbose, rx, tx).await;
    } else {
        spi_dac_handler(verbose, rx, tx).await;
    }
}

async fn spi_dac_handler(
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

async fn pwm_dac_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;3]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    let gpio = Gpio::new().expect("Falló obtener gpios");
    let (tx0, rx0) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(21).expect("Falló gettear el gpio 21").into_output();
    tokio::spawn(async move {
        pwm(
            rx0,
            pin
        ).await;
    });
    let (tx1, rx1) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(20).expect("Falló gettear el gpio 20").into_output();
    tokio::spawn(async move {
        pwm(
            rx1,
            pin
        ).await;
    });
    let (tx2, rx2) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(26).expect("Falló gettear el gpio 26").into_output();
    tokio::spawn(async move {
        pwm(
            rx2,
            pin
        ).await;
    });
    let (tx3, rx3) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(16).expect("Falló gettear el gpio 16").into_output();
    tokio::spawn(async move {
        pwm(
            rx3,
            pin
        ).await;
    });
    let (tx4, rx4) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(19).expect("Falló gettear el gpio 19").into_output();
    tokio::spawn(async move {
        pwm(
            rx4,
            pin
        ).await;
    });
    let (tx5, rx5) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(13).expect("Falló gettear el gpio 13").into_output();
    tokio::spawn(async move {
        pwm(
            rx5,
            pin
        ).await;
    });
    let (tx6, rx6) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(6).expect("Falló gettear el gpio 6").into_output();
    tokio::spawn(async move {
        pwm(
            rx6,
            pin
        ).await;
    });
    let (tx7, rx7) = tokio::sync::mpsc::channel(16);
    let pin = gpio.get(5).expect("Falló gettear el gpio 5").into_output();
    tokio::spawn(async move {
        pwm(
            rx7,
            pin
        ).await;
    });
    let mut respuesta= [0;2];

    loop {
        let msg = rx.recv().await.unwrap();
        
        if verbose { 
            println!("Dac got: {:02X}{:02X}{:02X}", msg[0], msg[1], msg[2]); 
        }

        respuesta.clone_from_slice(&msg[1..]);

        match msg[0]&0xF{
            0 => {
                tx0.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            1 => {
                tx1.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            2 => {
                tx2.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            3 => {
                tx3.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            4 => {
                tx4.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            5 => {
                tx5.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            6 => {
                tx6.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            7 => {
                tx7.send(u16::from_be_bytes(respuesta)).await.unwrap();
            }
            _ => {
                if verbose { println!("Address out of range"); }
                tx.send([0xF0,0xF0]).unwrap(); 
            }
        }

        respuesta.clone_from_slice(&msg[1..]);
        tx.send(respuesta).unwrap();
    }
}

async fn pwm(
    mut rx: tokio::sync::mpsc::Receiver<u16>,
    mut pin: rppal::gpio::OutputPin
){
    loop {
        let duty = rx.recv().await.unwrap() as f64;
        pin.set_pwm_frequency(10000.0, duty/1024.0).unwrap();
        //pin.set_pwm(Duration::from_micros(1024), Duration::from_micros(duty as u64)).unwrap();
    }
}

pub async fn dac_read(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;3]>,
    hat: bool
) -> [u8;2] {
    if hat {
        analog_core(msg, rx, tx).await
    } else {
        dac_core(0xC, msg, rx, tx).await
    }
}

pub async fn dac_write(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;3]>,
    hat: bool
) -> [u8;2] {
    if hat {
        analog_core(msg, rx, tx).await
    } else {
        dac_core(0x0, msg, rx, tx).await
    }
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

async fn analog_core(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;3]>
) -> [u8;2] {
    let mut arr = [0;3];
    let msg = msg & 0x000F03FF;
    arr.clone_from_slice(&msg.to_be_bytes()[1..]);

    tx.send(arr).await.unwrap();
    
    rx.recv().await.unwrap()
}
