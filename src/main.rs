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
use std::{env, fs, thread, time};

const INC_STEPS: u8 = 10;
const DEC_STEPS: u8 = 10;
const INC_PERCENT: f64 = 4.9;
const DEC_PERCENT: f64 = 6.0;
const FADE_IN_PERCENT_PER_STEP: f64 = 9.0;
const FADE_OUT_PERCENT_PER_STEP: f64 = 20.0;
const WAIT_BETWEEN_STEPS: time::Duration = time::Duration::from_millis(26);

fn get_current_vol(handler: &mut SinkController) -> Volume {
    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");

    let device = default_device;

    device.volume.avg()
}

enum PreVolCommand {
    Query,
    Save
}

fn pre_vol(handler: &mut SinkController, action: PreVolCommand) -> Option<Volume> {
    let file = env::temp_dir().join("pre_vol");
    match action {
        PreVolCommand::Query => {
            // read previous volume from file
            let file = fs::read(file)
                .expect("Failed to read pre_vol file");

            let vol = u32::from_le_bytes(
                file
                .try_into()
                .expect("Failed to convert file to u32")
            );

            return Some(Volume(vol))
        }
        PreVolCommand::Save => {
            // save current volume to a file
            let vol = get_current_vol(handler).0;
            fs::write(file, vol.to_le_bytes())
                .expect("Unable to write pre_vol file");

            return None
        }
    }
}

fn inc_vol(handler: &mut SinkController, dev_idx: u32, inc_percent_step: f64) {
    let inc_percent_step = inc_percent_step / 100.0;
    let target_volume: Volume = pre_vol(handler, PreVolCommand::Query).unwrap();

    // crescendo
    handler.set_device_mute_by_index(dev_idx, false);
    let mut i = 0;
    while i < INC_STEPS {
        // stop crescendo if target volume is reached between increment steps
        if get_current_vol(handler).ge(&target_volume) {
            break;
        };
        handler.increase_device_volume_by_percent(dev_idx, inc_percent_step);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn dec_vol(handler: &mut SinkController, dev_idx: u32, dec_percent_step: f64) {
    let dec_percent_step = dec_percent_step / 100.0;

    // diminuendo
    let mut i = 0;
    while i < DEC_STEPS {
        handler.decrease_device_volume_by_percent(dev_idx, dec_percent_step);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn mute(handler: &mut SinkController, dev_idx: u32) {
    // save current volume as a previous volume
    // in case we want to fade in later
    pre_vol(handler, PreVolCommand::Save);

    // fade out
    while get_current_vol(handler).gt(&Volume::MUTED) {
        dec_vol(handler, dev_idx, FADE_OUT_PERCENT_STEP);
    };
    handler.set_device_mute_by_index(dev_idx, true);
}

fn unmute(handler: &mut SinkController, dev_idx: u32) {
    // set target volume to previously saved volume
    let target_volume: Volume = pre_vol(handler, PreVolCommand::Query).unwrap();

    // fade in
    handler.set_device_mute_by_index(dev_idx, false);
    while get_current_vol(handler).lt(&target_volume) {
        inc_vol(handler, dev_idx, FADE_IN_PERCENT_STEP);
    };
}

fn toggle_mute(handler: &mut SinkController, dev_idx: u32) {
    if get_current_vol(handler).gt(&Volume::MUTED) {
        mute(handler, dev_idx);
    } else {
        unmute(handler, dev_idx);
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
        /// increment volume by a percentage step
        #[arg(value_name = "INCREMENT", default_value_t = INC_PERCENT_STEP)]
        inc_percent_step: f64,
        // wait_between_steps: Option<f64>,
    },

    /// decrease volume in diminuendo
    #[command(alias = "d", alias = "dec")]
    Decrease {
        /// decrement volume by a percentage step
        #[arg(value_name = "DECREMENT", default_value_t = DEC_PERCENT_STEP)]
        dec_percent_step: f64,
    },

    /// al niente (fade out to mute)
    #[command(alias = "m")]
    Mute {
        /// decrement volume by a percentage step
        decrement: f64,
    },

    /// dal niente (fade in from mute)
    #[command(alias = "u")]
    Unmute {
        /// increment volume by a percentage step
        increment: f64,
    },

    /// toggle al niente/dal niente
    #[command(alias = "t")]
    ToggleMute,
}

// Increment and decrement volume by a percentage step
// and fade in and out by a percentage step.
// #[derive(Args)]
// struct IncreaseArgs {
//     increment: Option<f64>,
//     // inc_percent_step: Option<f64>,
//     // wait_between_steps: Option<f64>,
// }
// #[derive(Args)]
// struct DecreaseArgs {
//     decrement: Option<f64>,
//     // dec_percent_step: Option<f64>,
//     // wait_between_steps: Option<f64>,
// }
// #[derive(Args)]
// struct FadeInArgs {
//     increment: Option<f64>,
//     // fade_in_percent_step: Option<f64>,
//     // wait_between_steps: Option<f64>,
// }
// #[derive(Args)]
// struct FadeOutArgs {
//     decrement: Option<f64>,
//     // fade_out_percent_step: Option<f64>,
//     // wait_between_steps: Option<f64>,
// }

fn main() {
    let args = Cli::parse();

    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");

    let device = default_device;

    match args.dynamics {
        // Dynamics::Increase => {
        Dynamics::Increase { inc_percent_step } => {
            print!("Crescendo\n");
            inc_vol(&mut handler, device.index, inc_percent_step);
        }
        Dynamics::Decrease { dec_percent_step } => {
            print!("Diminuendo\n");
            dec_vol(&mut handler, device.index, dec_percent_step);
        }
        Dynamics::Mute { decrement: _ } => {
            print!("Diminuendo al niente\n");
            mute(&mut handler, device.index);
        }
        Dynamics::Unmute { increment: _ } => {
            print!("Crescendo dal niente\n");
            unmute(&mut handler, device.index);
        }
        Dynamics::ToggleMute => {
            print!("Toggled mute state\n");
            toggle_mute(&mut handler, device.index);
        }
    };
}
