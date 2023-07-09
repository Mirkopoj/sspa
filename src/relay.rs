use rppal::gpio::{Gpio, OutputPin};

pub async fn relay_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8; 4]>,
    tx: tokio::sync::broadcast::Sender<[u8; 2]>,
    pin: u8,
) {
    let gpio = Gpio::new().unwrap();
    let mut relay_pin = gpio.get(pin).unwrap().into_output(); //reset 12, program 0
    relay_pin.set_low();

    loop {
        let msg = rx.recv().await.unwrap();
        let mut arr = [0; 2];
        arr.clone_from_slice(&msg[2..]);
        let valor_nuevo = <u16>::from_be_bytes(arr);

        relay_state(valor_nuevo, &mut relay_pin, verbose);

        let respuesta = arr;

        tx.send(respuesta).unwrap();
    }
}

pub async fn relay(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 4]>,
) -> [u8; 2] {
    tx.send(msg.to_be_bytes()).await.unwrap();
    rx.recv().await.unwrap()
}

fn relay_state(valor: u16, pin: &mut OutputPin, verbose: bool) {
    if valor == 0 {
        if verbose {
            println!("Relay off");
        }
        pin.set_high();
        return;
    }
    if verbose {
        println!("Relay on");
    }
    pin.set_low();
}
