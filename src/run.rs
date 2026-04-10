use crate::window;

use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use tungstenite::Message;
use tungstenite::client::IntoClientRequest;

const HOST: &str = "ws://127.0.0.1:5810/nt/ledview";

 pub fn run() {
    let rx = ws();
    window::start(rx);
}

pub fn ws() -> Receiver<Vec<[f64; 3]>> {
    let mut req = HOST.into_client_request().unwrap();
    req.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        "v4.1.networktables.first.wpi.edu".parse().unwrap(),
    );

    let (mut ws, _) = tungstenite::connect(req).unwrap();

    let msg = serde_json::json!([{
        "method": "subscribe",
        "params": {
            "topics": ["/leds"],
            "subuid": 10,
            "options": {},
        },
    }]);
    ws.send(Message::Text(msg.to_string().into())).unwrap();

    let ws = Arc::new(Mutex::new(ws));

    let (tx, rx) = mpsc::channel();

    let wsc = ws.clone();
    thread::spawn(move || {
        loop {
            wsc.lock().unwrap().send(Message::Ping(vec![].into())).unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let wsc = ws.clone();
    thread::spawn(move || {
        loop {
            let msg = wsc.lock().unwrap().read().unwrap();
            let parsed = rmp_serde::from_slice::<(u32, i64, u32, Vec<f64>)>(&msg.into_data());

            if let Ok((_, _, _, data)) = parsed {
                tx.send(data.as_chunks::<3>().0.to_owned()).ok();
            }

            thread::sleep(Duration::from_millis(1000 / 50));
        }
    });

    rx
}
