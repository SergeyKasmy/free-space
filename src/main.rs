use std::{collections::HashSet, convert::Infallible, process::Command, str::FromStr};

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
struct Min {
    /// ignore
    #[argh(option)]
    ignore: Option<IgnoreList>,
}

#[derive(Debug)]
struct IgnoreList(Vec<String>);

#[derive(Deserialize, Hash, PartialEq, Eq, Debug)]
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
    let act = args
        .action
        .unwrap_or_else(|| Action::Min(Min { ignore: None }));

    let duf = Command::new("duf")
        .arg("-json")
        .output()
        .wrap_err("Couldn't open duf")?;

    let devices = match serde_json::from_slice::<HashSet<Device>>(&duf.stdout) {
        Ok(devices) => {
            let mut devices = devices
                .into_iter()
                .filter(|x| {
                    x.device_type == "local" && x.fs_type != "ramfs" && x.fs_type != "autofs"
                })
                .collect::<Vec<_>>();

            devices.sort_by_key(|x| (x.free, x.mount_point.clone()));

            devices
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
        Action::Min(Min { ignore }) => {
            let min = devices
                .iter()
                .filter(|dev| {
                    let Some(ignore) = &ignore else {
                        return true;
                    };

                    !ignore.0.contains(&dev.mount_point)
                })
                .min_by_key(|dev| dev.free)
                .expect("checked for .is_empty() just up above");

            println!("{}: {}", min.mount_point, format_size(min.free, BINARY));
        }
    }

    Ok(())
}

impl FromStr for IgnoreList {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split(',').map(|x| x.to_owned()).collect()))
    }
}
