// volfade-rs — smooth volume transitions for PulseAudio
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

use clap::Parser;
use libpulse_binding::volume::Volume;
use std::{env, fs, thread, time};
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;
use pulsectl::controllers::types::DeviceInfo;

// parse command line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// increase volume in crescendo
    #[arg(short, long)]
    increase: bool,

    /// decrease volume in diminuendo
    #[arg(short, long)]
    decrease: bool,

    /// al niente (fade out)
    #[arg(short, long)]
    mute: bool,

    /// dal niente (fade in)
    #[arg(short, long)]
    unmute: bool,

    /// toggle al niente/dal niente
    #[arg(short, long)]
    toggle_mute: bool,
}

fn get_vol(handler: &mut SinkController) -> Volume {
    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");

    default_device.volume.avg()
}

fn dec_vol(handler: &mut SinkController, device_index: u32, steps: Option<f64>) {
    let ms = time::Duration::from_millis(100);
    let mut i = 0;
    while i <= 7 {
        // handler.decrease_device_volume_by_percent(device_index, 0.017);
        handler.decrease_device_volume_by_percent(device_index, steps.unwrap_or(0.017));
        thread::sleep(ms);
        i += 1;
    };
}

fn inc_vol(handler: &mut SinkController, device_index: u32, steps: Option<f64>) {
    handler.set_device_mute_by_index(device_index, false);
    let ms = time::Duration::from_millis(100);
    let mut i = 0;
    while i <= 7 {
        // handler.increase_device_volume_by_percent(device_index, 0.01375);
        handler.increase_device_volume_by_percent(device_index, steps.unwrap_or(0.01375));
        thread::sleep(ms);
        i += 1;
    };
}

fn toggle_mute(handler: &mut SinkController, device_index: u32) {

    let pre_vol = get_vol(handler).0.to_string();
    println!("the previous volume was: {}", pre_vol);
    let open_pre_vol_file = env::temp_dir().join("pre_vol");

    if get_vol(handler).gt(&Volume::MUTED) {
        // save previous volume to a file
        fs::write(open_pre_vol_file, pre_vol).expect("Unable to write pre_vol file");

        // fade out
        // while handler.get_device_by_index(device_index).unwrap().volume.avg().gt(&Volume::MUTED) {
        while get_vol(handler).gt(&Volume::MUTED) {
            dec_vol(handler, device_index, Some(0.04));
            println!("Current volume after dec_vol: {}", handler.get_device_by_index(device_index).unwrap().volume.avg());
        };
        handler.set_device_mute_by_index(device_index, true);
    } else {
        // read previous volume from file
        let saved_str = fs::read_to_string("/tmp/pre_vol").expect("Failed to read volume");
        let saved_val: u32 = saved_str.trim().parse().expect("Failed to parse volume");
        let target_volume = Volume(saved_val);
        println!("Target volume for fade in: {}", target_volume.0.to_string());

        // fade in
        handler.set_device_mute_by_index(device_index, false);
        while get_vol(handler).lt(&target_volume) {
            inc_vol(handler, device_index, Some(0.025));
            println!("Current volume after inc_vol: {}", handler.get_device_by_index(device_index).unwrap().volume.avg());
        };
    };
}

fn mute(handler: &mut SinkController, device_index: u32) {
    let pre_vol = get_vol(handler).0.to_string();
    let open_pre_vol_file = env::temp_dir().join("pre_vol");

    // save previous volume to a file
    fs::write(open_pre_vol_file, pre_vol).expect("Unable to write pre_vol file");

    // fade out
    while get_vol(handler).gt(&Volume::MUTED) {
        dec_vol(handler, device_index, Some(0.04));
        // println!("Current volume after dec_vol: {}", handler.get_device_by_index(device_index).unwrap().volume.avg());
    };
    handler.set_device_mute_by_index(device_index, true);
}

fn unmute(handler: &mut SinkController, device_index: u32) {
    // read previous volume from file
    let saved_str = fs::read_to_string("/tmp/pre_vol").expect("Failed to read volume");
    let saved_val: u32 = saved_str.trim().parse().expect("Failed to parse volume");
    let target_volume = Volume(saved_val);
    // println!("Target volume for fade in: {}", target_volume.0.to_string());

    // fade in
    handler.set_device_mute_by_index(device_index, false);
    while get_vol(handler).lt(&target_volume) {
        inc_vol(handler, device_index, Some(0.025));
        // println!("Current volume after inc_vol: {}", handler.get_device_by_index(device_index).unwrap().volume.avg());
    };
}

fn main() {
    let args = Args::parse();

    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let default_device: DeviceInfo = handler
        .get_default_device()
        .expect("Could not get default playback device.");


    if args.increase && args.decrease {
        println!("Cannot increase and decrease volume at the same time.");
    } else if args.increase && args.mute {
        println!("Cannot increase volume while muting.");
    } else if args.decrease && args.mute {
        println!("Already muting... Can it decrease volume at the same time?");
    } else if args.increase {
        inc_vol(&mut handler, default_device.index, Some(0.01375));
        println!("Volume increased.");
    } else if args.decrease {
        dec_vol(&mut handler, default_device.index, Some(0.017));
        println!("Volume decreased.");
    } else if args.mute {
        mute(&mut handler, default_device.index);
        println!("Volume muted/unmuted with fade in/out.");
    } else if args.unmute {
        unmute(&mut handler, default_device.index);
        println!("Volume unmuted with fade in.");
    } else if args.toggle_mute {
        toggle_mute(&mut handler, default_device.index);
        println!("Volume toggled mute/unmute with fade in/out.");
    } else {
        println!("No action specified. Use --increase, --decrease, or --mute.");
    }
}
