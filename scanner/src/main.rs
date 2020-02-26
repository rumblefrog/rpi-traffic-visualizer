use std::sync::{Arc, RwLock};
use std::process::Command;
use std::thread;
use std::fs::File;
use std::io::{Error, Read, Cursor};
use std::path::Path;

use csv::{ReaderBuilder, Trim, Error as CSVError};

use serde::Deserialize;

use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, Result as NotifyResult};

fn main() {
    let seg_display = Arc::new(RwLock::new(0));

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
        .args(&["wlan1mon", "--output-format", "csv", "-w", "out.dump"])
        .current_dir("airodump")
        .spawn()
        .expect("Unable to spawn airodump");

    // let a_seg = seg_display.clone();

    // thread::spawn(move || {
    //     loop {
    //         let x = a_seg.read().unwrap();
    
    //         println!("{}", x);
    //     }
    // });

    let b_seg = seg_display.clone();

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res: NotifyResult<Event>| {
        match res {
           Ok(event) => {
               for path in &event.paths {
                   let stations = parse_cap_file(path);

                   match stations {
                       Some(s) => {
                            let mut c = b_seg.write().unwrap();

                            *c = s.len();
        
                            println!("{}", *c);
                       }
                       None => println!("Can't read stations"),
                   }
               }
           },
           Err(e) => println!("watch error: {:?}", e),
        }
    }).expect("Can't watch");

    watcher.watch("./airodump", RecursiveMode::Recursive).expect("Can't start watcher");

    airodump_handle.wait().expect("Unable to wait dump thread");
}

// #[derive(Debug, Deserialize, PartialEq, Eq)]
// struct AP {
//     #[serde(rename = "BSSID")]
//     bssid: String,

//     #[serde(rename = "First time seen")]
//     first_time_seen: String,

//     #[serde(rename = "Last time seen")]
//     last_time_seen: String,

//     channel: u64,

//     speed: i64,

//     privacy: String,

//     cipher:  String,

//     authentication: String,

//     power: i64,

//     #[serde(rename = "# beacons")]
//     beacon_count: u64,

//     #[serde(rename = "# IV")]
//     iv: u64,

//     #[serde(rename = "LAN IP")]
//     lan_ip: String,

//     #[serde(rename = "ID-length")]
//     id_length: u64,

//     essid: String,

//     key: String,
// }

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Station {
    #[serde(rename = "Station MAC")]
    mac: String,

    #[serde(rename = "First time seen")]
    first_time_seen: String,

    #[serde(rename = "Last time seen")]
    last_time_seen: String,

    #[serde(rename = "Power")]
    power: i64,

    #[serde(rename = "# packets")]
    packet_count: u64,

    #[serde(rename = "BSSID")]
    bssid: String,

    #[serde(rename = "Probed ESSIDs")]
    probed_essids: String,
}

fn parse_cap_file<T: AsRef<Path>>(path: T) -> Option<Vec<Station>> {
    let file = File::open(path);

    if file.is_err() {
        return None
    }

    let mut buffer = String::new();

    if file.unwrap().read_to_string(&mut buffer).is_err() {
        return None
    }

    let offset = buffer.find("Station MAC");

    if offset.is_none() {
        return None
    }

    let reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(Cursor::new(&buffer[offset.unwrap()..]));

    let stations = reader
        .into_deserialize()
        .collect::<Result<Vec<Station>, CSVError>>();

    if stations.is_err() {
        return None
    }

    Some(stations.unwrap())
}