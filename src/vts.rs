use core::time;
use std::{
    fs,
    net::{TcpStream, UdpSocket},
    thread,
};

use tungstenite::{stream::MaybeTlsStream, WebSocket};

use self::requests::ParameterCreation;

pub mod responces {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Discovery {
        pub active: bool,
        pub port: String,
        #[serde(rename(deserialize = "instanceID"))]
        pub instance_id: String,
        pub window_title: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Auth {
        pub authentication_token: String,
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
    data: T,
}

pub enum VtsError {
    UnableToBind,
    UnableToSend,
    UnableToReceive,
    UnableToDeserialize,
    UnableToSerialize,
    NoWebSocket,
    UnableToStringifyMessage,
    UnableToSave,
    UnableToRead,
    UnableToSetTimeout,
}

pub struct Vts {
    ws_port: String,
    auth_token: Option<String>,
    websocket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl Vts {
    pub fn new() -> Self {
        let token = fs::read_to_string("token").ok();
        Self {
            ws_port: "8001".to_string(),
            auth_token: token,
            websocket: None,
        }
    }

    pub fn connect(&mut self) {
        while self.websocket.is_none() {
            match tungstenite::connect(format!("ws://localhost:{}", self.ws_port)) {
                Ok((websocket, _response)) => {
                    self.websocket = Some(websocket);
                    println!("Connected to VTS via Websocket");
                    break;
                }
                Err(_e) => {
                    println!("Unable connect to VTS retrying...");
                    match try_to_discover_port() {
                        Ok(_s) => continue,
                        Err(_e) => {
                            thread::sleep(time::Duration::from_secs(5));
                            println!("Unable to discover port retrying...");
                            continue;
                        }
                    };
                }
            }
        }
    }

    pub fn auth(&mut self) -> Result<(), VtsError> {
        let Some(ref mut ws) = self.websocket else {
            return Err(VtsError::NoWebSocket);
        };

        let token = match &self.auth_token {
            Some(a) => a.clone(),
            None => try_to_get_token(ws)?,
        };

        let auth_token = requests::Auth {
            plugin_name: "RustyBridge",
            plugin_developer: "ovROG",
            authentication_token: token.as_str(),
        };

        let auth_req = VTSApiRequest {
            data: auth_token,
            api_name: "VTubeStudioPublicAPI",
            api_version: "1.0",
            request_id: "iiii",
            message_type: "AuthenticationRequest",
        };

        let token_msg =
            serde_json::to_string(&auth_req).map_err(|_| VtsError::UnableToStringifyMessage)?;

        ws.send(token_msg.into())
            .map_err(|_| VtsError::UnableToSend)?;

        loop {
            match ws.read() {
                Ok(m) => {
                    if m.is_text() {
                        return Ok(());
                    }
                }
                Err(_e) => return Err(VtsError::UnableToRead),
            }
        }
    }

    pub fn add_params(&mut self, params: Vec<ParameterCreation>) {
        for param in params {
            _ = self.send(param, "ParameterCreationRequest");
            _ = self.read_next(); //TODO: Errors
        }
    }

    pub fn send<T: serde::Serialize>(
        &mut self,
        data: T,
        message_type: &str,
    ) -> Result<(), VtsError> {
        let Some(ref mut ws) = self.websocket else {
            return Err(VtsError::NoWebSocket);
        };

        let new_req = VTSApiRequest {
            data,
            api_name: "VTubeStudioPublicAPI",
            api_version: "1.0",
            request_id: "iiii",
            message_type,
        };

        let param_msg = serde_json::to_string(&new_req).map_err(|_| VtsError::UnableToSend)?;
        ws.send(param_msg.into())
            .map_err(|_| VtsError::UnableToSend)?;
        Ok(())
    }

    pub fn read_next(&mut self) -> Result<String, VtsError> {
        let Some(ref mut ws) = self.websocket else {
            return Err(VtsError::NoWebSocket);
        };

        loop {
            let msg = ws.read().map_err(|_| VtsError::UnableToRead)?;
            if msg.is_text() {
                return Ok(msg
                    .to_text()
                    .map_err(|_| VtsError::UnableToStringifyMessage)?
                    .to_string());
            }
        }
    }
}

fn try_to_discover_port() -> Result<String, VtsError> {
    let mut buf = [0; 4096];

    let discovery_socket = match UdpSocket::bind("0.0.0.0:47779") {
        Ok(s) => s,
        Err(_e) => return Err(VtsError::UnableToBind),
    };

    match discovery_socket.set_read_timeout(Some(time::Duration::from_secs(3))) {
        Ok(m) => m,
        Err(_e) => return Err(VtsError::UnableToSetTimeout),
    };

    let (amt, _src) = match discovery_socket.recv_from(&mut buf) {
        Ok(m) => m,
        Err(_e) => return Err(VtsError::UnableToReceive),
    };

    let data: VTSApiResponce<responces::Discovery> = match serde_json::from_slice(&buf[..amt]) {
        Ok(d) => d,
        Err(_e) => return Err(VtsError::UnableToDeserialize),
    };

    Ok(data.data.port)
}

fn try_to_get_token(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<String, VtsError> {
    let auth_data = requests::AuthToken {
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

    let auth_msg =
        serde_json::to_string(&auth_token_req).map_err(|_| VtsError::UnableToSerialize)?;

    ws.send(auth_msg.into())
        .map_err(|_| VtsError::UnableToSend)?;

    loop {
        let msg = ws.read().map_err(|_| VtsError::UnableToSend)?;

        if msg.is_text() {
            let msg_str = msg
                .to_text()
                .map_err(|_| VtsError::UnableToStringifyMessage)?;

            let token_res: VTSApiResponce<responces::Auth> =
                serde_json::from_str(msg_str).map_err(|_| VtsError::UnableToDeserialize)?;

            //TODO: VTS ERROR HANDLING

            fs::write("token", &token_res.data.authentication_token)
                .map_err(|_| VtsError::UnableToSave)?;

            return Ok(token_res.data.authentication_token);
        }
    }
}
