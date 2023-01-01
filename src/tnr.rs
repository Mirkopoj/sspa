pub async fn tnr_handler(
    verbose: bool,
    mut rx: tokio::sync::mpsc::Receiver<[u8;4]>,
    tx: tokio::sync::broadcast::Sender<[u8;2]>
){
    let mut registros = [0;6];

    loop {
        let msg = rx.recv().await.unwrap();
        let mut arr = [0;2];
        let valor_nuevo = <u16>::from_be_bytes(arr);

        if msg[0] == 0xA3 {
            actualizar(valor_nuevo, verbose);
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
            arr.clone_from_slice(&msg[2..]);
            registros[addr] = valor_nuevo;
            
            if addr == 5 { power_enable(valor_nuevo)}
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

#[allow(unused)]
fn actualizar(valor: u16, verbose: bool){}

#[allow(unused)]
fn power_enable(valor: u16){}
