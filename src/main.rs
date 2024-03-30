use core::time;
use std::{
    fs,
    net::UdpSocket,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use evalexpr::*;
use vts::{requests, Vts};

mod vts;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Cords {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct Shape {
    k: String,
    v: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CalcFn {
    name: String,
    func: String,
    min: f64,
    max: f64,
    default_value: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Config {
    func_file: String,
    iphone_ip: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct TrackingResponce {
    timestamp: u64,
    hotkey: i16,
    face_found: bool,
    rotation: Cords,
    position: Cords,
    eye_left: Cords,
    blend_shapes: Vec<Shape>,
}

fn main() {
    let config_string = fs::read_to_string("config.json").unwrap();
    let config: Config = serde_json::from_str(&config_string[..]).unwrap();
    let (precalc_funcs, new_params) = precalc_cfg(&config.func_file);

    let mut vts_app = Vts::new();

    let (sender, receiver): (Sender<TrackingResponce>, Receiver<TrackingResponce>) =
        mpsc::channel();

    vts_app.connect();
    while vts_app.auth().err().is_some() {
        thread::sleep(time::Duration::from_secs(1));
        println!("Unable to auth with VTS retrying...");
    }
    vts_app.add_params(new_params);

    thread::spawn(move || {
        let mut context = HashMapContext::new();

        let interval = time::Duration::from_millis(10);
        let mut next_time = std::time::Instant::now() + interval;

        loop {
            let data = match receiver.try_iter().last() {
                Some(d) => d,
                None => continue,
            };

            for v in &data.blend_shapes {
                context.set_value(v.k.clone(), v.v.into()).unwrap();
            }

            context
                .set_value("HeadPosX".into(), data.position.x.into())
                .unwrap();
            context
                .set_value("HeadPosY".into(), data.position.y.into())
                .unwrap();
            context
                .set_value("HeadPosZ".into(), data.position.z.into())
                .unwrap();

            context
                .set_value("HeadRotX".into(), data.rotation.x.into())
                .unwrap();
            context
                .set_value("HeadRotY".into(), data.rotation.y.into())
                .unwrap();
            context
                .set_value("HeadRotZ".into(), data.rotation.z.into())
                .unwrap();

            let mut params: Vec<requests::TrackingParam> = Vec::new();
            if data.face_found {
                for c in &precalc_funcs {
                    params.push(requests::TrackingParam {
                        id: c.0.as_str(),
                        value: c
                            .1
                            .eval_with_context(&context)
                            .unwrap()
                            .as_float()
                            .unwrap()
                            .clamp(-1000000.0, 1000000.0),
                        weight: Some(1.0),
                    });
                }
            }

            let params_data = requests::InjectParams {
                face_found: data.face_found,
                mode: "set",
                parameter_values: params,
            };

            let _ = vts_app.send(params_data, "InjectParameterDataRequest");
            let _ = vts_app.read_next();

            //rate limit
            thread::sleep(next_time - std::time::Instant::now());
            next_time += interval;
        }
    });

    let phone_socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let port = phone_socket.local_addr().unwrap().port();

    let mut buf = [0; 4096];

    let request_traking: String = serde_json::json!({
        "messageType":"iOSTrackingDataRequest",
        "sentBy": "RustyBridge",
        "sendForSeconds": 10,
        "ports": [port]
    })
    .to_string();

    let mut next_time = std::time::Instant::now();
    loop {
        if next_time <= std::time::Instant::now() {
            next_time = std::time::Instant::now() + time::Duration::from_secs(1);
            phone_socket
                .send_to(
                    request_traking.as_bytes(),
                    format!("{:}:21412", config.iphone_ip),
                )
                .unwrap();
        }

        match phone_socket.recv_from(&mut buf) {
            Ok((amt, _src)) => {
                let data: TrackingResponce = serde_json::from_slice(&buf[..amt]).unwrap();
                sender.send(data).unwrap();
            }
            Err(e) => {
                println!("couldn't recieve IPhone data: {}", e);
            }
        }
    }
}

fn precalc_cfg(
    file_path: &String,
) -> (
    Vec<(String, evalexpr::Node)>,
    Vec<requests::ParameterCreation>,
) {
    println!("file {:?}", file_path);

    let def_params = [
        String::from("FacePositionX"),
        String::from("FacePositionY"),
        String::from("FacePositionZ"),
        String::from("FaceAngleX"),
        String::from("FaceAngleY"),
        String::from("FaceAngleZ"),
        String::from("MouthSmile"),
        String::from("MouthOpen"),
        String::from("Brows"),
        String::from("ToungeOut"),
        String::from("EyeOpenLeft"),
        String::from("EyeOpenRight"),
        String::from("EyeLeftX"),
        String::from("EyeLeftY"),
        String::from("EyeRightX"),
        String::from("EyeRightY"),
        String::from("CheekPuff"),
        String::from("FaceAngry"),
        String::from("BrowLeftY"),
        String::from("BrowRightY"),
        String::from("MouthX"),
    ];

    let mut new_params: Vec<requests::ParameterCreation> = Vec::new();
    let config = fs::read_to_string(file_path).unwrap();
    let calc_fns: Vec<CalcFn> = serde_json::from_str(&config[..]).unwrap();

    let precalc_fns: Vec<_> = calc_fns
        .into_iter()
        .map(|func| {
            (func.name.clone(), {
                if !def_params.contains(&func.name) {
                    new_params.push(requests::ParameterCreation {
                        parameter_name: func.name,
                        explanation: "Custom rusty-bridge param".to_string(),
                        min: func.min,
                        max: func.max,
                        default_value: func.default_value,
                    })
                }
                evalexpr::build_operator_tree(&func.func[..]).unwrap()
            })
        })
        .collect();

    (precalc_fns, new_params)
}
