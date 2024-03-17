use evalexpr::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::net::UdpSocket;
use std::str;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use tungstenite::connect;

#[derive(Serialize, Deserialize, Debug)]
struct Cords {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Shape {
    k: String,
    v: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TrackingParam<'a> {
    id: &'a str,
    weight: Option<f64>,
    value: f64, // -1000000 | 1000000
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CalcFn {
    name: String,
    func: String,
    min: f64,
    max: f64,
    default_value: f64,
}

//VTS WS TYPES

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VTSApiResponce<T> {
    api_name: String,
    api_version: String,
    timestamp: u64,
    message_type: String,
    #[serde(rename(deserialize = "requestID"))]
    request_id: String,
    data: T,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VTSApiRequest<'a, T> {
    api_name: &'a str,
    api_version: &'a str,
    #[serde(rename(deserialize = "requestID"))]
    request_id: &'a str,
    message_type: &'a str,
    data: T,
}

//RESPONCES

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DiscoveryResponce {
    active: bool,
    port: u16,
    #[serde(rename(deserialize = "instanceID"))]
    instance_id: String,
    window_title: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthResponce {
    authentication_token: String,
}

//REQUESTS

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthTokenRequest<'a> {
    plugin_name: &'a str,
    plugin_developer: &'a str,
    plugin_icon: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthRequest<'a> {
    plugin_name: &'a str,
    plugin_developer: &'a str,
    authentication_token: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InjectParamsRequest<'a> {
    face_found: bool,
    mode: &'a str,
    parameter_values: Vec<TrackingParam<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ParameterCreationRequest {
    parameter_name: String,
    explanation: String,
    min: f64,
    max: f64,
    default_value: f64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Not enough arguments");
        return;
    }
    let ip = &args[1];
    let file_path = &args[2];

    let ws_port: u16 = find_ws_port();
    let (precalc_fns, new_params) = precalc_cfg(file_path);
    let mut auth_token = fs::read_to_string("token").ok();

    // Phone
    let phone_socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => panic!("unable bind phone socket: {}", e),
    };
    let port = phone_socket.local_addr().unwrap().port();
    let request_traking: String = json!({
        "messageType":"iOSTrackingDataRequest",
        "sentBy": "RustyBridge",
        "sendForSeconds": 10,
        "ports": [port]
    })
    .to_string();
    let socket_clone = phone_socket.try_clone().expect("unable clone phone socket");

    let (mut websocket, _response) = connect(format!("ws://localhost:{}", ws_port)).unwrap();

    thread::spawn(move || {
        let mut buf = [0; 4096];
        let mut context = HashMapContext::new();

        if auth_token.is_none() {
            let auth_data = AuthTokenRequest {
                plugin_name: "RustyBridge",
                plugin_developer: "ovROG",
                plugin_icon: None,
            };

            let auth_token_req = VTSApiRequest {
                data: auth_data,
                api_name: "VTubeStudioPublicAPI",
                api_version: "1.0",
                request_id: "iiii",
                message_type: "AuthenticationTokenRequest",
            };

            let auth_msg = serde_json::to_string(&auth_token_req).unwrap();
            websocket.send(auth_msg.into()).unwrap();

            loop {
                let msg = websocket.read().unwrap();
                if msg.is_text() {
                    let token: VTSApiResponce<AuthResponce> =
                        serde_json::from_str(msg.to_text().unwrap()).unwrap();
                    fs::write("token", &token.data.authentication_token).unwrap();
                    auth_token = Some(token.data.authentication_token);
                    break;
                }
            }
        }

        let auth_token = AuthRequest {
            plugin_name: "RustyBridge",
            plugin_developer: "ovROG",
            authentication_token: &auth_token.ok_or("").unwrap(), //TODO: Remove Some()
        };

        let auth_req = VTSApiRequest {
            data: auth_token,
            api_name: "VTubeStudioPublicAPI",
            api_version: "1.0",
            request_id: "iiii",
            message_type: "AuthenticationRequest",
        };

        let token_msg = serde_json::to_string(&auth_req).unwrap();
        websocket.send(token_msg.into()).unwrap();

        loop {
            let msg = websocket.read().unwrap();
            if msg.is_text() {
                break;
            }
        }

        //New params
        for p in new_params {
            let new_param_req = VTSApiRequest {
                data: p,
                api_name: "VTubeStudioPublicAPI",
                api_version: "1.0",
                request_id: "iiii",
                message_type: "ParameterCreationRequest",
            };
            let param_msg = serde_json::to_string(&new_param_req).unwrap();
            websocket.send(param_msg.into()).unwrap();

            loop {
                let msg = websocket.read().unwrap();
                if msg.is_text() {
                    break;
                }
            }
        }

        let interval = Duration::from_millis(16);
        let mut next_time = Instant::now() + interval;

        loop {
            match socket_clone.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    let data: TrackingResponce = serde_json::from_slice(&buf[..amt]).unwrap();
                    for v in data.blend_shapes {
                        context.set_value(v.k.into(), v.v.into()).unwrap();
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

                    let mut params: Vec<TrackingParam> = Vec::new();
                    for c in &precalc_fns {
                        params.push(TrackingParam {
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

                    let params_data = InjectParamsRequest {
                        face_found: data.face_found,
                        mode: "set",
                        parameter_values: params,
                    };

                    let params_request = VTSApiRequest {
                        data: params_data,
                        api_name: "VTubeStudioPublicAPI",
                        api_version: "1.0",
                        request_id: "iiii",
                        message_type: "InjectParameterDataRequest",
                    };

                    let params_msg = serde_json::to_string(&params_request).unwrap();
                    websocket.send(params_msg.into()).unwrap();

                    loop {
                        let msg = websocket.read().unwrap();
                        if msg.is_ping() || msg.is_pong() {
                            break;
                        }
                    }
                    thread::sleep(next_time - Instant::now()); //rate limit
                    next_time += interval;
                }
                Err(e) => {
                    println!("couldn't recieve a datagram: {}", e);
                }
            }
        }
    });

    loop {
        phone_socket
            .send_to(request_traking.as_bytes(), format!("{ip}:21412"))
            .unwrap();
        thread::sleep(Duration::from_secs(1));
    }
}

// Other fns

fn precalc_cfg(file_path: &String) -> (Vec<(String, Node)>, Vec<ParameterCreationRequest>) {
    println!("file {:?}", file_path);

    let def_params = [
        String::from("FacePositionX"),
        String::from("FacePositionX"),
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

    let mut new_params: Vec<ParameterCreationRequest> = Vec::new();

    let config = fs::read_to_string(file_path).unwrap();

    let calc_fns: Vec<CalcFn> = serde_json::from_str(&config[..]).unwrap();

    let precalc_fns: Vec<_> = calc_fns
        .into_iter()
        .map(|func| {
            (func.name.clone(), {
                if !def_params.contains(&func.name) {
                    new_params.push(ParameterCreationRequest {
                        parameter_name: func.name,
                        explanation: "Custom rusty-bridge param".to_string(),
                        min: func.min,
                        max: func.max,
                        default_value: func.default_value,
                    })
                }
                build_operator_tree(&func.func[..]).unwrap()
            })
        })
        .collect();

    return (precalc_fns, new_params);
}

fn find_ws_port() -> u16 {
    let mut buf = [0; 4096];
    let discovery_socket = match UdpSocket::bind("0.0.0.0:47779") {
        Ok(s) => s,
        Err(e) => panic!("unable bind discovery socket: {}", e),
    };
    match discovery_socket.recv_from(&mut buf) {
        Ok((amt, _src)) => {
            let data: VTSApiResponce<DiscoveryResponce> =
                serde_json::from_slice(&buf[..amt]).unwrap();
            return data.data.port;
        }
        Err(e) => {
            panic!("unable recieve discovery msg: {}", e);
        }
    }
}
