// volfade-rs — Volfaders change the volume levels with smooth fading transitions
//
// Copyright (C) 2024  Juan de Dios Hernández <
//
// GPLv3 or later
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use clap::{Parser, Subcommand};
use libpulse_binding::volume::Volume;
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;
use pulsectl::controllers::types::DeviceInfo;
use std::{env, fs, path::Path, thread, time};

const DEFAULT_VOLUME: Volume = Volume(65536 / 4); // 25% volume
const DEFAULT_INCREMENT: f64 = 5.0;
const DEFAULT_DECREMENT: f64 = 5.0;

const INC_STEPS: u8 = 10;
const DEC_STEPS: u8 = 10;

const FADE_IN_INCREMENT_PER_STEP: f64 = 9.0;
const FADE_OUT_DECREMENT_PER_STEP: f64 = 20.0;

const WAIT_BETWEEN_STEPS: time::Duration = time::Duration::from_millis(26);

// fn get_current_vol(handler: &mut SinkController) -> Result<Volume, pulsectl::Error> {
//     let default_device: DeviceInfo = match handler.get_default_device() {
//         Ok(d) => d,
//         Err(msg) => {
//             eprintln!("Error! Could not get default playback device:\n");
//             return Err(pulsectl::Error::Controller(msg))
//         }
//     };
//     let device = default_device;
//     Ok(device.volume.avg())
// }

fn get_current_vol(handler: &mut SinkController) -> Volume {
    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");


    let device = default_device;

    device.volume.avg()
}

enum ToFile {
    FromBuffer(Volume),
    _FromCurrentVolume
}

enum VolumeCache {}

impl VolumeCache {
    fn get_path() -> String {
        let cache_dir = match env::var("XDG_CACHE_HOME") {
            Ok(dir) => {
                format!("{}/volfade-rs", dir)
            }
            Err(_) => {
                let dir = env::var("HOME")
                    .expect("HOME environment variable not set");
                format!("{}/.cache/volfade-rs", dir)
            }
        };
        if !Path::new(&cache_dir).exists() {
            fs::create_dir(&cache_dir)
                .expect("Failed to create cache directory");
        };

        let filename = "previous_volume";
        let cache_path = cache_dir + "/" + filename;
        cache_path
    }
    fn save(handler: &mut SinkController, t: ToFile) {
        let vol = match t {
            ToFile::FromBuffer(buffered_vol) => buffered_vol,
            ToFile::_FromCurrentVolume => CurrentVolume::get(handler),
        };
        let vol = vol.0;
        fs::write(VolumeCache::get_path(), vol.to_le_bytes())
            .expect("Unable to write pre_vol file");
    }
}

enum CurrentVolume {}

impl CurrentVolume {
    fn get(handler: &mut SinkController) -> Volume {
        let default_device: DeviceInfo = handler
            .get_default_device()
            .expect("Could not get default playback device.");

        let device = default_device;

        device.volume.avg()
    }
}

type PreviousVolume = VolumeCache;

impl PreviousVolume {
    fn query() -> Option<Volume> {
        match fs::read(VolumeCache::get_path()) {
            Ok(data) => {
                let vol = u32::from_le_bytes(
                    data
                    .try_into()
                    .expect("Failed to convert file to u32")
                );
                Some(Volume(vol))
            }
            Err(_) => Some(DEFAULT_VOLUME)
        }
    }
}

fn inc_vol(handler: &mut SinkController, dev_idx: u32, increment: f64, target_volume: Option<Volume>) {
    let inc_percent_per_step: f64 = (increment / 100.0) / INC_STEPS as f64;

    // crescendo
    handler.set_device_mute_by_index(dev_idx, false);
    let mut i = 0;
    while i < INC_STEPS {
        // stop crescendo if target volume is reached between increment steps
        if let Some(target_volume) = target_volume {
            if get_current_vol(handler).ge(&target_volume) {
                break;
            };
        };
        handler.increase_device_volume_by_percent(dev_idx, inc_percent_per_step);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn dec_vol(handler: &mut SinkController, dev_idx: u32, decrement: f64) {
    let dec_percent_per_step: f64 = (decrement / 100.0) / DEC_STEPS as f64;

    // diminuendo
    let mut i = 0;
    while i < DEC_STEPS {
        handler.decrease_device_volume_by_percent(dev_idx, dec_percent_per_step);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn mute(handler: &mut SinkController, dev_idx: u32, decrement_per_step: f64) {
    if get_current_vol(handler).eq(&Volume::MUTED) {
        return;
    };

    // store current volume before fading out
    // VolCache::save(handler, ToFile::_FromCurrentVolume);
    let vol_buffer: Volume = CurrentVolume::get(handler);

    // fade out
    while get_current_vol(handler).gt(&Volume::MUTED) {
        dec_vol(handler, dev_idx, decrement_per_step);
    };
    handler.set_device_mute_by_index(dev_idx, true);

    // in case we want to fade in later
    VolumeCache::save(handler, ToFile::FromBuffer(vol_buffer));
}

fn unmute(handler: &mut SinkController, dev_idx: u32, increment_per_step: f64) {
    // set target volume from previously saved volume
    let target_volume: Volume = PreviousVolume::query().unwrap();

    // fade in
    handler.set_device_mute_by_index(dev_idx, false);
    while get_current_vol(handler).lt(&target_volume) {
        inc_vol(handler, dev_idx, increment_per_step, Some(target_volume));
    };
}

fn toggle_mute(handler: &mut SinkController, dev_idx: u32) {
    if get_current_vol(handler).gt(&Volume::MUTED) {
        mute(handler, dev_idx, FADE_OUT_DECREMENT_PER_STEP);
    } else {
        unmute(handler, dev_idx, FADE_IN_INCREMENT_PER_STEP);
    };
}

/// Volfaders change the volume levels with smooth fading transitions (for PulseAudio).
#[derive(Parser)]
#[command(author = "Juan de Dios Hernández, <86342863+macydnah@users.noreply.github.com>")]
#[command(version, long_about = None, rename_all = "kebab-case")]
#[command(propagate_version = true)]
#[group(id = "dynamics", required = false, multiple = false)]
struct Cli {
    #[command(subcommand)]
    dynamics: Dynamics,
}

/// Dynamics
#[derive(Subcommand)]
#[command(long_about = None, rename_all = "kebab-case")]
enum Dynamics {
    /// increase volume in crescendo
    #[command(arg_required_else_help = false, alias = "i", alias = "inc")]
    // #[command(visible_alias = "i", alias = "inc")]
    Increase {
        /// specify percent to increase
        #[arg(value_name = "VOL", default_value_t = DEFAULT_INCREMENT)]
        increment: f64,
        // wait_between_steps: Option<f64>,
    },

    /// decrease volume in diminuendo
    #[command(alias = "d", alias = "dec")]
    Decrease {
        /// specify percent to decrease
        #[arg(value_name = "VOL", default_value_t = DEFAULT_DECREMENT)]
        decrement: f64,
    },

    /// al niente (fade out to mute)
    #[command(alias = "m")]
    Mute {
        /// how much volume percent to decrease per step
        #[arg(default_value_t = FADE_OUT_DECREMENT_PER_STEP)]
        decrement_per_step: f64,
    },

    /// dal niente (fade in from mute)
    #[command(alias = "u")]
    Unmute {
        /// how much volume percent to increase per step
        #[arg(default_value_t = FADE_IN_INCREMENT_PER_STEP)]
        increment_per_step: f64,
    },

    /// toggle al niente/dal niente
    #[command(alias = "t")]
    ToggleMute,
}

fn main() -> Result<(), pulsectl::Error> {
    let args = Cli::parse();

    // create handler that calls functions on playback devices and apps
    let mut handler = match SinkController::create() {
        Ok(h) => h,
        Err(msg) => {
            eprintln!("Error! Could not create PulseAudio handler:\n");
            return Err(pulsectl::Error::Controller(msg))
        }
    };

    let default_device: DeviceInfo = match handler.get_default_device() {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("Error! Could not get default playback device:\n");
            return Err(pulsectl::Error::Controller(msg))
        }
    };

    let device = default_device;

    match args.dynamics {
        // Dynamics::Increase => {
        Dynamics::Increase { increment } => {
            print!("Crescendo\n");
            inc_vol(&mut handler, device.index, increment, None);
        }
        Dynamics::Decrease { decrement } => {
            print!("Diminuendo\n");
            dec_vol(&mut handler, device.index, decrement);
        }
        Dynamics::Mute { decrement_per_step } => {
            print!("Diminuendo al niente\n");
            mute(&mut handler, device.index, decrement_per_step);
        }
        Dynamics::Unmute { increment_per_step } => {
            print!("Crescendo dal niente\n");
            unmute(&mut handler, device.index, increment_per_step);
        }
        Dynamics::ToggleMute => {
            print!("Toggled mute state\n");
            toggle_mute(&mut handler, device.index);
        }
    };
    Ok(())
}
