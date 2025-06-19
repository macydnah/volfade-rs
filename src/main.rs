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

const INC_PERCENT_STEP: f64 = 1.375 / 100.0;
const DEC_PERCENT_STEP: f64 = 1.7 / 100.0;
// const FADE_IN_PERCENT_STEP: f64 = 2.5 / 100.0;
// const FADE_OUT_PERCENT_STEP: f64 = 4.0 / 100.0;
const WAIT_BETWEEN_STEPS: time::Duration = time::Duration::from_millis(26);

fn get_current_vol(handler: &mut SinkController) -> Volume {
    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");

    let device = default_device;

    device.volume.avg()
}

fn dec_vol(handler: &mut SinkController, dev_idx: u32) {
    let mut i = 0;
    while i <= 7 {
        handler.decrease_device_volume_by_percent(dev_idx, DEC_PERCENT_STEP);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn inc_vol(handler: &mut SinkController, dev_idx: u32) {
    handler.set_device_mute_by_index(dev_idx, false);
    let mut i = 0;
    while i <= 7 {
        handler.increase_device_volume_by_percent(dev_idx, INC_PERCENT_STEP);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
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
            let vol = u32::from_le_bytes(file.try_into().expect("Failed to convert bytes to u32"));

            Some(Volume(vol))
        }
        PreVolCommand::Save => {
            // save current volume to a file
            let vol = get_current_vol(handler).0;
            fs::write(file, vol.to_le_bytes())
                .expect("Unable to write pre_vol file");

            None
        }
    }
}

fn mute(handler: &mut SinkController, dev_idx: u32) {
    // save current volume in case we want to fade in later
    pre_vol(handler, PreVolCommand::Save);

    // fade out
    while get_current_vol(handler).gt(&Volume::MUTED) {
        dec_vol(handler, dev_idx);
    };
    handler.set_device_mute_by_index(dev_idx, true);
}

fn unmute(handler: &mut SinkController, dev_idx: u32) {
    // read previous volume from file
    let target_volume: Volume = pre_vol(handler, PreVolCommand::Query).unwrap();

    // fade in
    handler.set_device_mute_by_index(dev_idx, false);
    while get_current_vol(handler).lt(&target_volume) {
        inc_vol(handler, dev_idx);
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
    #[command(visible_alias = "i", alias = "inc")]
    Increase,

    /// decrease volume in diminuendo
    #[command(visible_alias = "d", alias = "dec")]
    Decrease,

    /// al niente (fade out to mute)
    #[command(visible_alias = "m")]
    Mute,

    /// dal niente (fade in from mute)
    #[command(visible_alias = "u")]
    Unmute,

    /// toggle al niente/dal niente
    #[command(visible_alias = "t")]
    ToggleMute,
}

fn main() {
    let args = Cli::parse();

    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");

    let device = default_device;

    match args.dynamics {
        Dynamics::Increase => {
            print!("Crescendo\n");
            inc_vol(&mut handler, device.index);
        }
        Dynamics::Decrease => {
            print!("Diminuendo\n");
            dec_vol(&mut handler, device.index);
        }
        Dynamics::Mute => {
            print!("Diminuendo al niente\n");
            mute(&mut handler, device.index);
        }
        Dynamics::Unmute => {
            print!("Crescendo dal niente\n");
            unmute(&mut handler, device.index);
        }
        Dynamics::ToggleMute => {
            print!("Toggled mute state\n");
            toggle_mute(&mut handler, device.index);
        }
    };
}
