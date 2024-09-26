use std::{
    collections::VecDeque,
    fs,
    net::{TcpStream, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc,
    },
};

use evalexpr::{ContextWithMutableVariables, HashMapContext, Node};
use log::{error, info, warn};
use serde_json::Value;
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

use crate::vtsphone::TrackingResponce;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VTSApiRequest<'a, T> {
    api_name: &'a str,
    api_version: &'a str,
    #[serde(rename(deserialize = "requestID"))]
    request_id: &'a str,
    message_type: &'a str,
    data: Option<T>,
}

pub mod responces {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Discovery {
        pub active: bool,
        pub port: u16,
        #[serde(rename(deserialize = "instanceID"))]
        pub instance_id: String,
        pub window_title: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct APIStateResponse {
        pub active: bool,
        pub v_tube_studio_version: String,
        pub current_session_authenticated: bool,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthenticationToken {
        pub authentication_token: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthenticationResponse {
        pub authenticated: bool,
        pub reason: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct APIError {
        #[serde(rename(deserialize = "errorID"))]
        pub error_id: u16,
        pub message: String,
    }
}

pub mod requests {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthToken<'a> {
        pub plugin_name: &'a str,
        pub plugin_developer: &'a str,
        pub plugin_icon: Option<&'a str>,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Auth<'a> {
        pub plugin_name: &'a str,
        pub plugin_developer: &'a str,
        pub authentication_token: &'a str,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ParameterCreation {
        pub parameter_name: String,
        pub explanation: String,
        pub min: f64,
        pub max: f64,
        pub default_value: f64,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct TrackingParam<'a> {
        pub id: &'a str,
        pub weight: Option<f64>,
        pub value: f64, // -1000000 | 1000000
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct InjectParams<'a> {
        pub face_found: bool,
        pub mode: &'a str,
        pub parameter_values: Vec<TrackingParam<'a>>,
    }
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

pub struct VtsPc;

impl VtsPc {
    pub fn run(
        receiver: Receiver<TrackingResponce>,
        transformation_cfg_path: String,
        active: Arc<AtomicBool>,
    ) {
        while active.load(Ordering::Relaxed) {
            let flag = Arc::clone(&active);

            let websocket = VtsPc::connect();
            VtsPc::msg_loop(websocket, &receiver, &transformation_cfg_path, flag);
        }
    }

    fn connect() -> WebSocket<MaybeTlsStream<TcpStream>> {
        let mut port = "8001".to_string();
        loop {
            match tungstenite::connect(format!("ws://localhost:{}", port)) {
                Ok((websocket, _responce)) => {
                    info!("Connected to local port:{}", port);
                    return websocket;
                }
                Err(error) => {
                    warn!("{}", error);
                    match VtsPc::discover_port() {
                        Ok(prt) => {
                            port = prt;
                        }
                        Err(e) => {
                            warn!("{}", e);
                            continue;
                        }
                    }
                }
            }
        }
    }

    fn discover_port() -> Result<String, String> {
        let mut buf = [0; 4096];

        let discovery_socket = match UdpSocket::bind("0.0.0.0:47779") {
            Ok(s) => s,
            Err(e) => return Err(e.to_string()),
        };

        match discovery_socket.set_read_timeout(Some(core::time::Duration::from_secs(3))) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };

        let (amt, _src) = match discovery_socket.recv_from(&mut buf) {
            Ok(m) => m,
            Err(e) => return Err(e.to_string()),
        };

        let data: VTSApiResponce<responces::Discovery> = match serde_json::from_slice(&buf[..amt]) {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };

        Ok(data.data.port.to_string())
    }

    fn msg_loop(
        mut websocket: WebSocket<MaybeTlsStream<TcpStream>>,
        receiver: &Receiver<TrackingResponce>,
        transformation_cfg_path: &String,
        active: Arc<AtomicBool>,
    ) {
        let mut msg_buffer: VecDeque<Message> = VecDeque::new();
        let mut token: Option<String> = fs::read_to_string("token").ok();

        msg_buffer.push_back(VtsPc::req_status_msg());

        let (precalc_funcs, mut new_params) = VtsPc::precalc_cfg(transformation_cfg_path);

        msg_buffer.append(&mut new_params);

        // let interval = time::Duration::from_millis(30);
        // let mut next_time = std::time::Instant::now() + interval;

        let mut dont_send = false;

        while active.load(Ordering::Relaxed) {
            if !dont_send {
                if let Some(msg) = msg_buffer.front() {
                    match websocket.send(msg.clone()) {
                        Ok(_) => {}
                        Err(error) => {
                            warn!("Unable to send msg: {}", error);
                            break; // Reconnect
                        }
                    }
                } else {
                    let tracking_data = VtsPc::tracking_msg(&precalc_funcs, receiver);
                    if tracking_data.is_some() {
                        match websocket.send(tracking_data.unwrap()) {
                            Ok(_) => {}
                            Err(error) => {
                                warn!("Unable to send tracking msg: {}", error);
                                break; // Reconnect
                            }
                        }
                    } else {
                        continue;
                    }
                }
            }

            match websocket.read() {
                Ok(msg) => {
                    if msg.is_text() {
                        let msg_value =
                            serde_json::from_str::<Value>(msg.to_text().unwrap()).unwrap();

                        match msg_value["messageType"].as_str() {
                            Some(msg_type) => match msg_type {
                                "APIError" => {
                                    let err_data = serde_json::from_value::<
                                        VTSApiResponce<responces::APIError>,
                                    >(msg_value)
                                    .unwrap();
                                    // warn!("API error: {:?}", err_data.data);
                                    match err_data.data.error_id {
                                        8 => {
                                            // msg_buffer.push_back(VtsPc::auth(&token));
                                        }
                                        51 => {
                                            // POPUP ON SCREEN

                                            // MAYBE
                                            // DELAY
                                            // msg_buffer.push_back(VtsPc::auth(&token));
                                        }
                                        352 => {
                                            // custom parameter exist
                                            msg_buffer.pop_front();
                                        }
                                        354 => {
                                            // custom parameter is default
                                            msg_buffer.pop_front();
                                        }
                                        450 => {
                                            //No param data was sended
                                        }
                                        _ => error!("Unknown API error: {:?}", err_data.data),
                                    }
                                }
                                "APIStateResponse" => {
                                    let state_data =
                                        serde_json::from_value::<
                                            VTSApiResponce<responces::APIStateResponse>,
                                        >(msg_value)
                                        .unwrap();
                                    msg_buffer.pop_front();
                                    if !state_data.data.current_session_authenticated {
                                        msg_buffer.push_front(VtsPc::auth(&token));
                                    }
                                }
                                "AuthenticationTokenResponse" => {
                                    let token_data =
                                        serde_json::from_value::<
                                            VTSApiResponce<responces::AuthenticationToken>,
                                        >(msg_value)
                                        .unwrap();

                                    let _ =
                                        fs::write("token", &token_data.data.authentication_token)
                                            .map_err(|e| error!("Unable to save token: {:?}", e));
                                    token = Some(token_data.data.authentication_token);
                                    info!("Recived Token from VtubeStudio");
                                    msg_buffer.pop_front();
                                    msg_buffer.push_front(VtsPc::auth(&token));
                                }
                                "AuthenticationResponse" => {
                                    let auth_data = serde_json::from_value::<
                                        VTSApiResponce<responces::AuthenticationResponse>,
                                    >(msg_value)
                                    .unwrap();
                                    msg_buffer.pop_front();
                                    if !auth_data.data.authenticated {
                                        token = None;
                                        let _ = fs::remove_file("token")
                                            .map_err(|e| error!("Unable to delete token: {:?}", e));
                                        info!("Invalid Token, Requesting new...");
                                        msg_buffer.push_back(VtsPc::auth(&token));
                                    }
                                }
                                "InjectParameterDataResponse" => {
                                    // println!("{:?}", msg);
                                }
                                "ParameterCreationResponse" => {
                                    // println!("{:?}", msg);
                                    msg_buffer.pop_front();
                                }
                                _ => warn!("Unknown message: {}", msg_value["messageType"]),
                            },
                            None => warn!("No type in responce: {}", msg.to_text().unwrap()),
                        }
                        dont_send = false;
                    } else if msg.is_ping() || msg.is_pong() {
                        dont_send = true;
                        continue;
                    } else {
                        warn!("Non text response: {:?}", msg);
                        continue;
                    }
                }
                Err(error) => {
                    warn!("Unable to read msg: {}", error);
                    break; // Reconnect
                }
            }

            //rate limit
            // thread::sleep(next_time - std::time::Instant::now());
            // next_time += interval;
        }
    }

    fn tracking_msg(
        precalc_funcs: &Vec<(String, Node)>,
        receiver: &Receiver<TrackingResponce>,
    ) -> Option<Message> {
        let mut context = HashMapContext::new();

        let mut binding = receiver.try_iter();
        let it = binding.by_ref();

        let raw_data = match it.last() {
            Some(data) => data,
            None => {
                return None;
            }
        };

        for v in &raw_data.blend_shapes {
            context.set_value(v.k.clone(), v.v.into()).unwrap();
        }

        context
            .set_value("HeadPosX".into(), raw_data.position.x.into())
            .unwrap();
        context
            .set_value("HeadPosY".into(), raw_data.position.y.into())
            .unwrap();
        context
            .set_value("HeadPosZ".into(), raw_data.position.z.into())
            .unwrap();

        context
            .set_value("HeadRotX".into(), raw_data.rotation.x.into())
            .unwrap();
        context
            .set_value("HeadRotY".into(), raw_data.rotation.y.into())
            .unwrap();
        context
            .set_value("HeadRotZ".into(), raw_data.rotation.z.into())
            .unwrap();

        let mut params: Vec<requests::TrackingParam> = Vec::new();

        if raw_data.face_found {
            for c in precalc_funcs {
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

        if params.is_empty() {
            return None;
        }

        let params_data = requests::InjectParams {
            face_found: raw_data.face_found,
            mode: "set",
            parameter_values: params,
        };

        let message_type = "InjectParameterDataRequest";

        let request = VTSApiRequest {
            data: Some(params_data),
            api_name: "VTubeStudioPublicAPI",
            api_version: "1.0",
            request_id: "iiii",
            message_type,
        };

        let request_string = serde_json::to_string(&request).unwrap();

        Some(Message::text(request_string))
    }

    fn req_status_msg() -> Message {
        let status_req = VTSApiRequest::<i32> {
            data: None,
            api_name: "VTubeStudioPublicAPI",
            api_version: "1.0",
            request_id: "iiii",
            message_type: "APIStateRequest",
        };

        let status_req_msg = serde_json::to_string(&status_req).unwrap();
        info!("Requesing status of VtubeStudio");
        Message::text(status_req_msg)
    }

    fn auth(token: &Option<String>) -> Message {
        if token.is_some() {
            let tk = token.clone().unwrap();

            let auth_token = requests::Auth {
                plugin_name: "RustyBridgeUi",
                plugin_developer: "ovROG",
                authentication_token: tk.as_str(),
            };

            let auth_req = VTSApiRequest {
                data: Some(auth_token),
                api_name: "VTubeStudioPublicAPI",
                api_version: "1.0",
                request_id: "iiii",
                message_type: "AuthenticationRequest",
            };

            let auth_req_msg = serde_json::to_string(&auth_req).unwrap();

            info!("Authentication Request to VtubeStudio");
            return Message::text(auth_req_msg);
        }

        let auth_data = requests::AuthToken {
            plugin_name: "RustyBridgeUi",
            plugin_developer: "ovROG",
            plugin_icon: None,
        };

        let token_req = VTSApiRequest {
            data: Some(auth_data),
            api_name: "VTubeStudioPublicAPI",
            api_version: "1.0",
            request_id: "iiii",
            message_type: "AuthenticationTokenRequest",
        };

        let token_req_msg = serde_json::to_string(&token_req).unwrap();

        info!("Authentication Token Request: Please accept PopUp in VtubeStudio");
        Message::text(token_req_msg)
    }

    fn precalc_cfg(file_path: &String) -> (Vec<(String, evalexpr::Node)>, VecDeque<Message>) {
        info!("Loadling tranformation config: {}", file_path);

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
            String::from("TongueOut"),
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
            String::from("VoiceFrequencyPlusMouthSmile"),
        ];

        let mut new_params: VecDeque<Message> = VecDeque::new();
        let config = fs::read_to_string(file_path).unwrap();
        let calc_fns: Vec<CalcFn> = serde_json::from_str(&config[..]).unwrap();

        let precalc_fns: Vec<_> = calc_fns
            .into_iter()
            .map(|func| {
                (func.name.clone(), {
                    info!("Loading Param: {}", &func.name);
                    if !def_params.contains(&func.name) {
                        let param_data = requests::ParameterCreation {
                            parameter_name: func.name,
                            explanation: "Custom rusty-bridge param".to_string(),
                            min: func.min,
                            max: func.max,
                            default_value: func.default_value,
                        };

                        let param_req = VTSApiRequest {
                            data: Some(param_data),
                            api_name: "VTubeStudioPublicAPI",
                            api_version: "1.0",
                            request_id: "iiii",
                            message_type: "ParameterCreationRequest",
                        };

                        let param_req_msg = serde_json::to_string(&param_req).unwrap();

                        new_params.push_back(Message::text(param_req_msg));
                    }
                    match evalexpr::build_operator_tree(&func.func[..]) {
                        Ok(calc) => calc,
                        Err(error) => {
                            error!(
                                "Unable to read cfg (probably error or typo in function): {}",
                                error
                            );
                            panic!()
                        }
                    }
                })
            })
            .collect();

        info!("Tranformation config loaded");
        (precalc_fns, new_params)
    }
}
