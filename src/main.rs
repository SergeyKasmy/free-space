use std::process::Command;

use argh::FromArgs;
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use humansize::{format_size, BINARY};
use serde::Deserialize;

/// show free space
#[derive(FromArgs, Debug)]
struct Args {
    /// action
    #[argh(subcommand)]
    action: Option<Action>,

    /// ignore
    #[argh(option)]
    ignore: Option<String>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Action {
    All(All),
    Min(Min),
}

/// all
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "all")]
struct All {}

/// min
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "min")]
struct Min {}

#[derive(Deserialize, Debug)]
struct Device {
    device_type: String,
    free: u64,
    mount_point: String,
    fs_type: String,
    // #[serde(rename = "device")]
    // name: String,
    // #[serde(rename = "type")]
    // kind: String,
    // opts: String,
    // total: u64,
    // used: u64,
    // inodes: u64,
    // inodes_free: u64,
    // inodes_used: u64,
    // blocks: u64,
    // block_size: u64,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let act = args.action.unwrap_or(Action::Min(Min {}));

    let duf = Command::new("duf")
        .arg("-json")
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

    match act {
        Action::All(_) => {
            for dev in devices {
                println!("{}: {}", dev.mount_point, format_size(dev.free, BINARY));
            }
        }
        Action::Min(_) => {
            let min = devices
                .iter()
                .filter(|dev| {
                    let Some(ignore) = &args.ignore else {
                        return true;
                    };

                    dev.mount_point != ignore.as_str()
                })
                .min_by_key(|dev| dev.free)
                .expect("checked for .is_empty() just up above");

            println!("{}: {}", min.mount_point, format_size(min.free, BINARY));
        }
    }

    Ok(())
}
