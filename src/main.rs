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

fn get_vol(handler: &mut SinkController) -> Volume {
    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");

    default_device.volume.avg()
}

fn dec_vol(handler: &mut SinkController, device_index: u32) {
    let mut i = 0;
    while i <= 7 {
        handler.decrease_device_volume_by_percent(device_index, DEC_PERCENT_STEP);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn inc_vol(handler: &mut SinkController, device_index: u32) {
    handler.set_device_mute_by_index(device_index, false);
    let mut i = 0;
    while i <= 7 {
        handler.increase_device_volume_by_percent(device_index, INC_PERCENT_STEP);
        thread::sleep(WAIT_BETWEEN_STEPS);
        i += 1;
    };
}

fn mute(handler: &mut SinkController, device_index: u32) {
    // save previous volume to a file in case we want to fade in later
    let pre_vol = get_vol(handler).0.to_string();
    let open_pre_vol_file = env::temp_dir().join("pre_vol");
    fs::write(open_pre_vol_file, pre_vol).expect("Unable to write pre_vol file");

    // fade out
    while get_vol(handler).gt(&Volume::MUTED) {
        dec_vol(handler, device_index);
    };
    handler.set_device_mute_by_index(device_index, true);
}

fn unmute(handler: &mut SinkController, device_index: u32) {
    // read previous volume from file
    let saved_str = fs::read_to_string("/tmp/pre_vol").expect("Failed to read volume");
    let saved_val: u32 = saved_str.trim().parse().expect("Failed to parse volume");
    let target_volume = Volume(saved_val);

    // fade in
    handler.set_device_mute_by_index(device_index, false);
    while get_vol(handler).lt(&target_volume) {
        inc_vol(handler, device_index);
    };
}

fn toggle_mute(handler: &mut SinkController, device_index: u32) {
    if get_vol(handler).gt(&Volume::MUTED) {
        mute(handler, device_index);
    } else {
        unmute(handler, device_index);
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
    #[command(visible_alias = "i")]
    Increase,

    /// decrease volume in diminuendo
    #[command(visible_alias = "d")]
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

    match args.dynamics {
        Dynamics::Increase => {
            print!("Crescendo\n");
            inc_vol(&mut handler, default_device.index);
        }
        Dynamics::Decrease => {
            print!("Diminuendo\n");
            dec_vol(&mut handler, default_device.index);
        }
        Dynamics::Mute => {
            print!("Diminuendo al niente\n");
            mute(&mut handler, default_device.index);
        }
        Dynamics::Unmute => {
            print!("Crescendo dal niente\n");
            unmute(&mut handler, default_device.index);
        }
        Dynamics::ToggleMute => {
            print!("Toggled mute state\n");
            toggle_mute(&mut handler, default_device.index);
        }
    };
}
