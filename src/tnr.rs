use rppal::gpio::{Gpio, OutputPin};
use tokio::fs;
use tokio::process::{ChildStdin, Command};
use tokio::io::AsyncWriteExt;
use std::process::Stdio;
use num::Integer;

pub async fn tnr_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;4]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    let mut registros = [1;6];
    registros[0] = 100;
    registros[1] = 10;
    
    let gpio = Gpio::new().unwrap();
    let mut power_enable_pin = gpio.get(4).unwrap().into_output();
    let mut tnr = None;

    loop {
        let msg = rx.recv().await.unwrap();
        let mut arr = [0;2];
        arr.clone_from_slice(&msg[2..]);
        let valor_nuevo = <u16>::from_be_bytes(arr);

        if msg[0] == 0xA3 {
            tnr = actualizar(verbose, registros, tnr).await;
            tx.send(arr).unwrap();
            continue;
        }

        let addr = msg[1] as usize;

        if addr > 5 {
            if verbose { println!("Direccion invalida"); }
            tx.send(arr).unwrap();
            continue;
        }

        if msg[0] == 0x23 {
            if verbose { println!("Se guardó {} en {}", valor_nuevo, addr); }
            registros[addr] = valor_nuevo;
            
            if addr == 5 { power_enable(valor_nuevo, &mut power_enable_pin, verbose); }
        }

        let respuesta = registros[addr].to_be_bytes();

        tx.send(respuesta).unwrap();
    }
}

pub async fn tnr(
    msg: u32,
    rx: &mut tokio::sync::broadcast::Receiver<[u8;2]>,
    tx: &tokio::sync::mpsc::Sender<[u8;4]>
) -> [u8;2] {
    tx.send(msg.to_be_bytes()).await.unwrap();
    rx.recv().await.unwrap()
}

async fn actualizar(verbose: bool, reg: [u16;6], tnr: Señal) -> Señal {
    if verbose { println!("generando señal {:?}", reg); }
    generar_señal(reg[1], reg[0], reg[2], reg[3], tnr, reg[4]).await
}

fn power_enable(valor: u16, pin: &mut OutputPin, verbose: bool){
    if valor == 0 {
        if verbose { println!("PowerEnable set low"); }
        pin.set_low();
        return;
    }
    if verbose { println!("PowerEnable set high"); }
    pin.set_high();
}

async fn generar_señal (
    ancho_del_pulso: u16,
    periodo: u16,
    margen_inicial: u16,
    margen_final: u16,
    señal_anterior: Señal,
    cantidad_de_pulsos: u16
) -> Señal {
    let señal_terminada = terminar_señal(señal_anterior);
    let duracion_del_bit = generar_archivo_señal(
        ancho_del_pulso,
        periodo,
        margen_inicial,
        margen_final
    ).await;
    señal_terminada.await;
    ejecutar_señal(duracion_del_bit, cantidad_de_pulsos).await
}

async fn terminar_señal(señal_anterior: Señal) {
    match señal_anterior {
        Some(mut señal) => {
            let msg: [u8;1] = [0;1];
            señal
                .write(&msg)
                .await
                .expect("No se puedo terminar la señal");
            drop(señal);
        }
        None => { }
    }
}

async fn generar_archivo_señal(
    ancho_del_pulso: u16,
    periodo: u16,
    margen_inicial: u16,
    margen_final: u16,
) -> u16 {    
    let bit_time = maximo_comun_divisor(
        ancho_del_pulso,
        periodo,
        margen_inicial,
        margen_final
    ).await;
    let unos_tnr = (ancho_del_pulso / bit_time) as usize;
    let ceros_tnr = ((periodo-ancho_del_pulso) / bit_time) as usize;
    let ceros_encabezado_rf = (margen_inicial / bit_time) as usize;
    let unos_rf = (((ancho_del_pulso - margen_inicial) - margen_final) / bit_time) as usize;
    let ceros_cola_rf = (((periodo - ancho_del_pulso) + margen_final )/ bit_time) as usize;

    let tnr = 
        String::from("27 ")+
        &String::from_utf8(vec![b'1'; unos_tnr]) .unwrap()+
        &String::from_utf8(vec![b'0'; ceros_tnr]) .unwrap();

    let rf = 
        String::from("17 ")+
        &String::from_utf8(vec![b'0'; ceros_encabezado_rf]) .unwrap()+
        &String::from_utf8(vec![b'1'; unos_rf]) .unwrap()+
        &String::from_utf8(vec![b'0'; ceros_cola_rf]) .unwrap();

    fs::write("wave_file", tnr + "\n" + &rf + "\n").await.expect("Failed to write to file");

    bit_time
    
}

async fn maximo_comun_divisor(a: u16, b: u16, c: u16, d: u16) -> u16 {
    a.gcd(&b.gcd(&c.gcd(&d)))
}

async fn ejecutar_señal(bit_time: u16, count: u16) -> Señal {
    let bit_time = bit_time.to_string();
    let count = count.to_string();

    let mut señal = Command::new("taskset")
        .arg("-c")
        .arg("3")
        .arg("python3")
        .arg("/opt/sspa/gen_tnr.py")
        .arg("wave_file")
        .arg(bit_time)
        .arg(count)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Falló el lanzar la señal");

    señal.stdin.take()
}

type Señal = Option<ChildStdin>;
