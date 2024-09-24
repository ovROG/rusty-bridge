use std::{net::UdpSocket, sync::mpsc::Sender, time};

use log::warn;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Cords {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Shape {
    pub k: String,
    pub v: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TrackingResponce {
    pub timestamp: u64,
    pub hotkey: i16,
    pub face_found: bool,
    pub rotation: Cords,
    pub position: Cords,
    pub eye_left: Cords,
    pub blend_shapes: Vec<Shape>,
}

pub struct VtsPhone;

impl VtsPhone {
    pub fn run(ip: String, sender: Sender<TrackingResponce>) {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let _ = socket.set_read_timeout(Some(time::Duration::new(2, 0)));
        let port = socket.local_addr().unwrap().port();

        let mut buf = [0; 4096];

        let request_traking: String = serde_json::json!({
            "messageType":"iOSTrackingDataRequest",
            "sentBy": "RustyBridge",
            "sendForSeconds": 10,
            "ports": [port]
        })
        .to_string();

        let mut next_time = time::Instant::now();

        loop {
            if next_time <= time::Instant::now() {
                next_time = time::Instant::now() + time::Duration::from_secs(1);

                match socket.send_to(request_traking.as_bytes(), format!("{:}:21412", ip)) {
                    Ok(_) => {
                        // nice
                    }
                    Err(error) => {
                        warn!("Unable to request tracking data: {}", error) // Maybe reconnect
                    }
                }
            }

            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => match serde_json::from_slice::<TrackingResponce>(&buf[..amt]) {
                    Ok(data) => sender.send(data).unwrap(),
                    Err(error) => {
                        warn!("Unnable to deserialize: {}", error)
                    }
                },
                Err(error) => {
                    warn!("Unnable to receive: {}", error) // Maybe reconnect
                }
            }
        }
    }
}
