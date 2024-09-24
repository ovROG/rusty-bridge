extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use std::{
    cell::Cell,
    env, fs,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

use nwd::NwgUi;
use nwg::NativeUi;
use rusty_bridge_lib::{
    vtspc::VtsPc,
    vtsphone::{TrackingResponce, VtsPhone},
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiCfg {
    transform_path: Option<String>,
    ip: Option<String>,
}

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(size: (300, 135), position: (300, 300), title: "Rusty Bridge", flags: "WINDOW|VISIBLE")]
    #[nwg_events( OnWindowClose: [App::close], OnInit: [App::init] )]
    window: nwg::Window,

    #[nwg_resource]
    embed: nwg::EmbedResource,

    #[nwg_control(size: (280, 25), position: (10, 52), placeholder_text: Some("(0.0.0.0) IPhone Ip"))]
    #[nwg_events( OnTextInput: [App::save] )]
    phone_ip: nwg::TextInput,

    #[nwg_control(text: "Connect", size: (280, 30), position: (10, 90))]
    #[nwg_events( OnButtonClick: [App::connect] )]
    connect_button: nwg::Button,

    #[nwg_control(size: (240, 25), position: (10, 12), placeholder_text: Some("Path to transform file"))]
    #[nwg_events( OnTextInput: [App::save] )]
    transform_file_path: nwg::TextInput,

    #[nwg_control(text: "📃", size: (30, 30), position: (260, 10))]
    #[nwg_events( OnButtonClick: [App::open_file] )]
    file_button: nwg::Button,

    #[nwg_resource( action: FileDialogAction::Open, title: "Select Transfom File")]
    file_dialog: nwg::FileDialog,

    #[nwg_control(tip: Some("Rusty Bridge"), icon : data.embed.icon_str("APP_ICON", None).as_ref())]
    #[nwg_events(MousePressLeftUp: [App::show_menu], OnContextMenu: [App::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Show")]
    #[nwg_events(OnMenuItemSelected: [App::show])]
    tray_item1: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [App::exit])]
    tray_item3: nwg::MenuItem,

    connected: Cell<bool>,
}

impl App {
    fn init(&self) {
        let em = &self.embed;
        self.window.set_icon(em.icon_str("APP_ICON", None).as_ref());
        if let Ok(last_config) = fs::read_to_string("ui-cfg.json") {
            let cfg = serde_json::from_str::<UiCfg>(&last_config).unwrap();

            if cfg.transform_path.is_some() {
                self.transform_file_path
                    .set_text(&cfg.transform_path.unwrap());
            }

            if cfg.ip.is_some() {
                self.phone_ip.set_text(&cfg.ip.unwrap());
            }
        };
    }

    fn save(&self) {
        let cfg = UiCfg {
            transform_path: Some(self.transform_file_path.text()),
            ip: Some(self.phone_ip.text()),
        };
        let cfg_str = serde_json::to_string(&cfg).unwrap();
        fs::write("ui-cfg.json", cfg_str).unwrap();
    }

    fn connect(&self) {
        if self.connected.get() {
            let path = self.transform_file_path.text().clone();
            let ip = self.phone_ip.text().clone();

            let (sender, receiver): (Sender<TrackingResponce>, Receiver<TrackingResponce>) =
                mpsc::channel();

            let pctr_handler = thread::spawn(move || {
                VtsPc::run(receiver, path);
            });

            let phonetr_handler = thread::spawn(move || VtsPhone::run(ip, sender));

            //TODO: stop loop to kill thread =c
            // use mpsc

            self.transform_file_path.set_readonly(true);
            self.phone_ip.set_readonly(true);
            self.file_button.set_enabled(false);
            self.connect_button.set_text("Disconnect");

            self.connected.set(true);
        }

        // let _ = pctr_handler.join();
        // let _ = phonetr_handler.join();
    }

    fn open_file(&self) {
        if let Ok(d) = env::current_dir() {
            if let Some(d) = d.to_str() {
                self.file_dialog
                    .set_default_folder(d)
                    .expect("Failed to set default folder.");
            }
        }

        if self.file_dialog.run(Some(&self.window)) {
            {
                self.transform_file_path.set_text("");
                if let Ok(path) = self.file_dialog.get_selected_item() {
                    let dir = path.into_string().unwrap();
                    self.transform_file_path.set_text(&dir);
                }
            }
        };
    }

    fn close(&self) {
        self.window.minimize();
    }

    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn show(&self) {
        self.window.restore();
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    let log_config = include_str!("../../configs/log_cfg.yml");
    let raw_log_config = serde_yaml::from_str(log_config).unwrap();
    log4rs::init_raw_config(raw_log_config).unwrap();

    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("PT Sans").expect("Failed to set default font");

    let _app = App::build_ui(Default::default()).expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
