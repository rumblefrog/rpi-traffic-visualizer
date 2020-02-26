use std::sync::{Arc, RwLock};
use std::process::{Command, Stdio};
use std::thread;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

use serde::Deserialize;

use segdisplay::SegDisplay;

use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, Result as NotifyResult};

use chrono::prelude::*;
use chrono::Duration;

fn main() {
    let seg_display = Arc::new(RwLock::new(0u32));

    println!("Killing processes");

    Command::new("airmon-ng")
        .args(&["check", "kill"])
        .output()
        .expect("Unable to kill processes");

    println!("Starting interface");

    Command::new("airmon-ng")
        .args(&["start", "wlan1"])
        .output()
        .expect("Unable to start monitor interface");

    println!("Dumping");

    let mut airodump_handle = Command::new("airodump-ng")
        .args(&["wlan1mon", "--output-format", "json", "-w", "out.dump"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .current_dir("airodump")
        .spawn()
        .expect("Unable to spawn airodump");

    let mut seg_hw = SegDisplay::new().expect("Unable to create seg display");

    let a_seg = seg_display.clone();

    thread::spawn(move || {
        loop {
            let x = a_seg.read().unwrap();
    
            seg_hw.write_int(*x);
        }
    });

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res: NotifyResult<Event>| {
        match res {
           Ok(event) => {
               for path in &event.paths {
                   let stations = parse_cap_file(path);

                   match stations {
                       Some(s) => {
                            // println!("{:?}", s);

                            let z_device = s.into_iter().find(|x| x.mac.as_ref().unwrap() == "1C:36:BB:8F:35:A7");

                            let mut c = seg_display.write().unwrap();

                            if let Some(z_device_resolved) = z_device {
                                // Power is between -100 and 0 afaik
                                *c = (100 + z_device_resolved.power) as u32;
                            } else {
                                *c = 0;
                            }

                            // let mut c = b_seg.write().unwrap();

                            // *c = s.len() as u32;
                       }
                       None => println!("Can't read stations"),
                   }
               }
           },
           Err(e) => println!("watch error: {:?}", e),
        }
    }).expect("Can't watch");

    watcher.watch("./airodump", RecursiveMode::NonRecursive).expect("Can't start watcher");

    airodump_handle.wait().expect("Unable to wait dump thread");
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Station {
    #[serde(rename = "StationMAC")]
    mac: Option<String>,

    #[serde(rename = "FirstTimeSeen")]
    first_time_seen: String,

    #[serde(rename = "LastTimeSeen")]
    last_time_seen: String,

    #[serde(rename = "Power")]
    power: i8,

    #[serde(rename = "#packets")]
    packet_count: Option<u64>,

    #[serde(rename = "BSSID")]
    bssid: String,

    #[serde(rename = "ESSID")]
    essid: String,

    #[serde(rename = "ProbedESSIDS")]
    probed_essids: Option<String>,

    #[serde(rename = "Manufacturer")]
    manufacturer: String,

    wlan_type: Option<String>,

    timestamp: String,
}

fn parse_cap_file<T: AsRef<Path>>(path: T) -> Option<Vec<Station>> {
    use serde_json::from_str;

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_e) => return None,
    };

    let reader = BufReader::new(file);

    let mut stations: Vec<Station> = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            if let Ok(e) = from_str(&l) {
                stations.push(e);
            }
        }
    }

    Some(
        stations
            .into_iter()
            .filter(|i| {
                if i.mac.is_none() {
                    return false
                }

                let l_seen = NaiveDateTime::parse_from_str(&i.last_time_seen, "%Y-%m-%d %H:%M:%S");

                if l_seen.is_err() {
                    return false
                }

                let x = Local::now().naive_local().signed_duration_since(l_seen.unwrap());

                x < Duration::seconds(30)
            })
            .collect()
    )
}