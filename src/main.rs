use std::{env::args, process::Command};

use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use serde::Deserialize;

enum Cmd {
    ListAll,
    Max,
}

#[derive(Deserialize, Debug)]
struct Device {
    device_type: String,
    free: i64,
    mount_point: String,
    fs_type: String,
    // #[serde(rename = "device")]
    // name: String,
    // #[serde(rename = "type")]
    // kind: String,
    // opts: String,
    // total: i64,
    // used: i64,
    // inodes: i64,
    // inodes_free: i64,
    // inodes_used: i64,
    // blocks: i64,
    // block_size: i64,
}

fn main() -> Result<()> {
    #[allow(clippy::wildcard_in_or_patterns)]
    let cmd = match args().nth(1).as_deref() {
        Some("all") => Cmd::ListAll,
        Some("max") | _ => Cmd::Max,
    };

    let duf = Command::new("duf")
        .arg("--json")
        .output()
        .wrap_err("Couldn't open duf")?;

    let devices = match serde_json::from_slice::<Vec<Device>>(&duf.stdout) {
        Ok(mut devices) => {
            devices.sort_by_key(|x| x.free);

            devices
                .into_iter()
                .filter(|x| x.device_type == "local" && x.fs_type != "ramfs")
                .collect::<Vec<_>>()
        }
        Err(e) => {
            bail!(
                "{e}\nduf returned an error:\n\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&duf.stdout),
                String::from_utf8_lossy(&duf.stderr)
            );
        }
    };

    if devices.is_empty() {
        bail!("No devices found");
    }

    match cmd {
        Cmd::ListAll => {
            for dev in devices {
                println!("{}: {}", dev.mount_point, dev.free);
            }
        }
        Cmd::Max => {
            let min = devices
                .iter()
                .min_by_key(|dev| dev.free)
                .expect("checked for .is_empty() just up above");

            println!("{}: {}", min.mount_point, min.free);
        }
    }

    Ok(())
}
