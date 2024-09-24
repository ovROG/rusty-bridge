use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use clap::Parser;
use rusty_bridge_lib::{vtspc::VtsPc, vtsphone::{TrackingResponce, VtsPhone}};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to json file with transformation config
    #[arg(short, long)]
    transform_cfg: String,
    /// Set phone ip
    #[arg(short, long)]
    phone_ip: String,
}

fn main() {
    let args = Args::parse();


    let log_config = include_str!("../configs/log_cfg.yml");
    let raw_log_config = serde_yaml::from_str(log_config).unwrap();
    log4rs::init_raw_config(raw_log_config).unwrap();

    let (sender, receiver): (Sender<TrackingResponce>, Receiver<TrackingResponce>) =
        mpsc::channel();

    let pctr_handler = thread::spawn(move || {
        VtsPc::run(receiver, args.transform_cfg);
    });

    let phonetr_handler = thread::spawn(move || VtsPhone::run(args.phone_ip, sender));

    let _ = pctr_handler.join();
    let _ = phonetr_handler.join();
}
