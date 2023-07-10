use std::time::Duration;

use rppal::gpio::{Gpio, Trigger, InputPin};
use std::sync::{Arc, Mutex};

pub async fn monitor_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8; 4]>,
    tx: tokio::sync::broadcast::Sender<[u8; 2]>,
) {
    let gpio = Gpio::new().unwrap();
    let monitor_pin = Arc::new(Mutex::new(gpio.get(1).unwrap().into_input()));
    let (tx_count, rx_count) = tokio::sync::broadcast::channel(16);
    let mut count_join_handle = None;

    loop {
        let msg = rx.recv().await.unwrap();
        let respuesta;
        match msg[1] {
            0 => {
                if verbose {
                    println!("Monitoring change");
                }
                let mut monitor_pin = monitor_pin.lock().unwrap();
                monitor_pin.set_interrupt(Trigger::RisingEdge).unwrap();
                let mut arr = [0; 2];
                arr.clone_from_slice(&msg[2..]);
                let timeout_period = <u16>::from_be_bytes(arr) as u64;

                let pin_ret;

                loop {
                    match monitor_pin.poll_interrupt(true, Some(Duration::from_millis(timeout_period)))
                    {
                        Ok(l) => {
                            pin_ret = l;
                            break;
                        }
                        Err(_) => {}
                    };
                }

                respuesta = match pin_ret {
                    Some(_) => {
                        if verbose {
                            println!("TnR found");
                        }
                        [0, 1]
                    }
                    None => {
                        if verbose {
                            println!("TnR not found");
                        }
                        [0, 0]
                    }
                };
                monitor_pin.clear_interrupt().unwrap();
            },
            1 => {
                let mut arr = [0; 2];
                arr.clone_from_slice(&msg[2..]);
                let timeout_period = <u16>::from_be_bytes(arr) as u64;
                count_join_handle = Some(
                    tokio::task::spawn(
                        counter(
                            rx_count.resubscribe(),
                            verbose,
                            monitor_pin.clone(),
                            timeout_period
                            )));
                respuesta = [0, 0];
            }
            2 => {
                tx_count.send(0).unwrap();
                respuesta = match count_join_handle {
                    Some(handle) => handle.await.unwrap().to_be_bytes(),
                    None => [0, 0],
                };
                count_join_handle = None;
            }
            _ => {
                let monitor_pin = monitor_pin.lock().unwrap();
                if verbose {
                    println!("Monitoring level");
                }
                if monitor_pin.is_high() {
                    if verbose {
                        println!("TnR found");
                    }
                    respuesta = [0, 1];
                } else {
                    if verbose {
                        println!("TnR not found");
                    }
                    respuesta = [0, 0];
                }
            },
        }

        tx.send(respuesta).unwrap();

    }
}

pub async fn tnr_monitor(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8; 2]>,
    tx: &tokio::sync::mpsc::Sender<[u8; 4]>,
) -> [u8; 2] {
    tx.send(msg.to_be_bytes()).await.unwrap();
    rx.recv().await.unwrap()
}

async fn counter(
    mut rx: tokio::sync::broadcast::Receiver<u8>,
    verbose: bool,
    monitor_pin: Arc<Mutex<InputPin>>,
    timeout_period: u64
) -> u16 {
    let mut count = 0;
    if verbose {
        println!("Monitoring change count");
    }
    let mut monitor_pin = monitor_pin.lock().unwrap();
    monitor_pin.set_interrupt(Trigger::RisingEdge).unwrap();

    loop{
        match rx.try_recv() {
            Ok(_) => { 
                if verbose {
                    println!("Counted {}", count);
                }
                monitor_pin.clear_interrupt().unwrap();
                return count;
            },
            Err(_) => { },
        }

        match monitor_pin.poll_interrupt(true, Some(Duration::from_millis(timeout_period)))
        {
            Ok(_) => {
                count +=1;
            }
            Err(_) => {
                if verbose {
                    println!("Reached Timeout Counted {}", count);
                }
                monitor_pin.clear_interrupt().unwrap();
                return count;
            }
        };

    }
}
