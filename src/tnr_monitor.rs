use std::time::Duration;

use rppal::gpio::{Gpio, Trigger};

pub async fn monitor_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;4]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>,
){
    let gpio = Gpio::new().unwrap();
    let mut monitor_pin = gpio.get(1).unwrap().into_input();

    loop {
        let msg = rx.recv().await.unwrap();
        monitor_pin.set_interrupt(Trigger::RisingEdge).unwrap();
        let mut arr = [0;2];
        arr.clone_from_slice(&msg[2..]);
        let timeout_period = <u16>::from_be_bytes(arr) as u64;


        let respuesta = match monitor_pin
            .poll_interrupt(
                true,
                Some(Duration::from_millis(timeout_period)))
            .unwrap() {
                Some(_) => { 
                    if verbose { println!("TnR found"); }
                    [0,1] 
                },
                None => {
                    if verbose { println!("TnR not found"); }
                    [0,0] 
                },
        };

        tx.send(respuesta).unwrap();

        monitor_pin.clear_interrupt();
    }
}

pub async fn tnr_monitor(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;4]>
) -> [u8;2] {
    tx.send(msg.to_be_bytes()).await.unwrap();
    rx.recv().await.unwrap()
}
