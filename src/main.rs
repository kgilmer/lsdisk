use std::fs::File;
use std::fs::{self, DirEntry};
use std::io::Read;

use console::pad_str;

const BLOCKFS_ROOT: &str = "/sys/block";
const BYTES_PER_BLOCK: u64 = 512;
const UNKNOWN_MODEL_STR: &str = "[UNKNOWN]";

fn main() -> Result<(), std::io::Error> {
    let block_dirs = fs::read_dir(BLOCKFS_ROOT)?;

    traverse_block_devices(block_dirs)?;

    Ok(())
}

fn traverse_block_devices(block_dirs: fs::ReadDir) -> Result<(), std::io::Error> {
    block_dirs
        .filter_map(|bd| bd.ok())
        .filter(|bd| disk_filter(bd))
        .for_each(|bd| {
            let device_path = bd.path().into_os_string().into_string().unwrap();
            let size_path = format!("{}/size", &device_path);
            let removable_path = format!("{}/removable", &device_path);

            let size = usize_from_file(&size_path).expect("has size") as u64;
            let size = bytesize::to_string(size * BYTES_PER_BLOCK, true);
            let model = device_model(bd.file_name().to_str().unwrap_or_default());
            let disk_type = match usize_from_file(&removable_path) {
                Ok(removable_flag) => match removable_flag {
                    0 => "Fixed".to_string(),
                    1 => "Removable".to_string(),
                    _ => UNKNOWN_MODEL_STR.to_string(),
                },
                _ => UNKNOWN_MODEL_STR.to_string(),
            };
            println!(
                "/dev/{} {} {} {}",
                pad_str(bd.file_name().to_str().unwrap_or_default(), 8, console::Alignment::Left, Some("…")),
                pad_str(&size, 10, console::Alignment::Left, Some("…")),
                pad_str(&model.unwrap_or(UNKNOWN_MODEL_STR.to_string()), 14, console::Alignment::Left, Some("…")),
                pad_str(&disk_type, 10, console::Alignment::Left, Some("…")),                
            );
        });

    Ok(())
}

fn disk_filter(dev: &DirEntry) -> bool {
    if let Ok(children) = fs::read_dir(dev.path()) {
        !children
            .filter_map(|bd| bd.ok())
            .any(|p| p.file_name() == "loop")
    } else {
        false
    }
}

// source ~ https://github.com/brayniac/sysfs-rs/blob/master/src/util.rs#L4
pub fn usize_from_file(path: &str) -> Result<usize, &'static str> {
    if let Ok(mut f) = File::open(&path) {
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
