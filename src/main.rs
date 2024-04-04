use std::fs::File;
use std::process;
use std::{fs, io::Read};

use clap::Parser;
use console::pad_str;

const BLOCKFS_ROOT: &str = "/sys/block";
const BYTES_PER_BLOCK: u64 = 512;
const UNKNOWN_MODEL_STR: &str = "[UNKNOWN]";

#[derive(Debug, Clone)]
struct DiskInfo {
    device_name: String,
    size_bytes: u64,
    model_name: String,
    is_removable: bool,
    is_loop_device: bool,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Print a list of attached disks
struct Args {
    /// Return only non-loop devices
    #[arg(short, long, default_value_t = false)]
    non_loop_only: bool,

    /// Return only removable devices
    #[arg(short, long, default_value_t = false)]
    removable_only: bool,

    /// Return error if matching devices not one
    #[arg(short, long, default_value_t = false)]
    expect_one: bool,

    /// Only print the device path
    #[arg(short, long, default_value_t = false)]
    brief: bool,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let block_dirs = fs::read_dir(BLOCKFS_ROOT)?;

    let disks = read_disks(block_dirs);

    match disks.len() {
        0 => println!("No disks found"),
        _ => list_disk_info(disks, &args),
    }

    Ok(())
}

fn list_disk_info(disks: Vec<DiskInfo>, config: &Args) {
    let mut filtered: Vec<DiskInfo> = disks
        .iter()
        .filter(|di| {
            #[allow(clippy::nonminimal_bool)]
            !((config.non_loop_only && !di.is_loop_device) || (config.removable_only && !di.is_removable))
        })
        .cloned()
        .collect();

    if config.expect_one && filtered.len() != 1 {
        eprintln!(
            "Expected a single device but {} were found: {:?}",
            filtered.len(),
            filtered
                .iter()
                .map(|di| { &di.device_name })
                .collect::<Vec<&String>>()
        );
        process::exit(1);
    }

    filtered.sort_unstable_by(|di1, di2| di1.device_name.cmp(&di2.device_name));

    filtered.iter().for_each(|di| match config.brief {
        true => println!("/dev/{}", di.device_name),
        false => {
            let rem_label = if di.is_removable {
                "removable"
            } else {
                "fixed"
            };
            println!(
                "/dev/{} {} {} {}",
                pad_str(
                    di.device_name.as_str(),
                    10,
                    console::Alignment::Left,
                    Some("…")
                ),
                pad_str(
                    bytesize::to_string(di.size_bytes, true).as_str(),
                    10,
                    console::Alignment::Left,
                    Some("…")
                ),
                pad_str(
                    di.model_name.as_str(),
                    14,
                    console::Alignment::Left,
                    Some("…")
                ),
                pad_str(rem_label, 10, console::Alignment::Left, Some("…")),
            );
        }
    })
}

fn read_disks(dir_entries: fs::ReadDir) -> Vec<DiskInfo> {
    dir_entries
        .filter_map(|bd| bd.ok())
        .map(|bd| {
            let device_name = bd.file_name().into_string().unwrap();
            let device_path = bd.path().into_os_string().into_string().unwrap();
            let size_path = format!("{}/size", &device_path);
            let removable_path = format!("{}/removable", &device_path);

            let size_bytes =
                (usize_from_file(&size_path).expect("has size") as u64) * BYTES_PER_BLOCK;
            let model_name = device_model(bd.file_name().to_str().unwrap_or_default())
                .unwrap_or(UNKNOWN_MODEL_STR.to_string());
            let is_removable = match usize_from_file(&removable_path) {
                Ok(removable_flag) => match removable_flag {
                    0 => false,
                    1 => true,
                    _ => panic!("Unrecognized value for removable: {}", removable_path),
                },
                _ => panic!("Unable to read: {}", removable_path),
            };
            let is_loop_device = if let Ok(children) = fs::read_dir(bd.path()) {
                !children
                    .filter_map(|bd| bd.ok())
                    .any(|p| p.file_name() == "loop")
            } else {
                false
            };

            DiskInfo {
                device_name,
                size_bytes,
                model_name,
                is_removable,
                is_loop_device,
            }
        })
        .collect()
}

// source ~ https://github.com/brayniac/sysfs-rs/blob/master/src/util.rs#L4
pub fn usize_from_file(path: &str) -> Result<usize, &'static str> {
    if let Ok(mut f) = File::open(path) {
        let mut s = String::new();
        match f.read_to_string(&mut s) {
            Ok(_) => match s.trim().parse() {
                Ok(i) => Ok(i),
                Err(_) => Err("unable to parse"),
            },
            Err(_) => Err("unable to read file contents"),
        }
    } else {
        Err("unable to open file")
    }
}

pub fn device_model(device_name: &str) -> Result<String, &'static str> {
    if let Ok(mut f) = File::open(format!("/sys/class/block/{}/device/model", device_name)) {
        let mut s = String::new();
        match f.read_to_string(&mut s) {
            Ok(_) => match s.trim().parse() {
                Ok(i) => Ok(i),
                Err(_) => Err("unable to parse"),
            },
            Err(_) => Err("unable to read file contents"),
        }
    } else {
        Err("unable to open file")
    }
}
