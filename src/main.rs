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

    /// al niente/dal niente
    /// (fade out and mute/unmute and fade in)
    #[arg(short, long)]
    mute: bool,
}

fn get_vol(handler: DeviceInfo) -> Volume {
    let avg_volume = handler.volume.avg();
    avg_volume
}
fn get_vol_string(handler: DeviceInfo) -> String {
    handler.volume.to_string()
}

fn dec_vol(handler: &mut SinkController, device_index: u32, steps: Option<f64>) {
    let mut i = 0;
    while i <= 7 {
        // handler.decrease_device_volume_by_percent(device_index, 0.017);
        handler.decrease_device_volume_by_percent(device_index, steps.unwrap_or(0.017));
        let ms = time::Duration::from_millis(100);
        thread::sleep(ms);
        i += 1;
    };
}

fn inc_vol(handler: &mut SinkController, device_index: u32, steps: Option<f64>) {
    handler.set_device_mute_by_index(device_index, false);
    let mut i = 0;
    while i <= 7 {
        // handler.increase_device_volume_by_percent(device_index, 0.01375);
        handler.increase_device_volume_by_percent(device_index, steps.unwrap_or(0.01375));
        let ms = time::Duration::from_millis(100);
        thread::sleep(ms);
        i += 1;
    };
}

fn mute_unmute(handler: &mut SinkController, device_index: u32) {
    let xdg_runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap();
    println!("xdg_runtime_dir: {}", xdg_runtime_dir);
    let pre_vol_file = format!("{}/preVol", xdg_runtime_dir);
    println!("pre_vol_file: {}", pre_vol_file);
    let pre_vol: u32 = match fs::read_to_string(&pre_vol_file) {
        Ok(content) => content.trim().parse().unwrap_or(30),
        Err(_) => 0,
    };
    println!("pre_vol: {}", pre_vol);
    // if pre_vol > 0 {
    //     // save current volume to file
    //     fs::write(&pre_vol_file, get_vol(handler.get_device_by_index(device_index).unwrap()).to_string())
    //         .expect("Unable to write preVol file");
    //     // fade out
    //     while get_vol(handler.get_device_by_index(device_index).unwrap()).to_u32() > 0 {
    //         // handler.decrease_device_volume_by_percent(device_index, 4.0);
    //         dec_vol(handler, device_index, Some(4.0));
    //     };
    //     handler.set_device_mute_by_index(device_index, true);
    // } else {
    //     handler.set_device_mute_by_index(device_index, false);
    //     // fade in
    //     while get_vol(handler.get_device_by_index(device_index).unwrap()).to_u32() <= pre_vol {
    //         // handler.increase_device_volume_by_percent(device_index, 2.5);
    //         inc_vol(handler, device_index, Some(2.5));
    //     };
    // };
    let default_device = handler
        .get_default_device()
        .expect("Could not get default playback device.");
    // if pre_vol > 0 {
    // if get_vol(default_device.clone()) > Volume::MUTED {
    println!("is valid? {}", get_vol(default_device.clone()).is_valid());
    println!("is normal? {}", get_vol(default_device.clone()).is_normal());
    if get_vol(default_device.clone()).gt(&Volume::MUTED) {
        // save current volume to file
        // fs::write(&pre_vol_file, get_vol(default_device.clone()))
        //     .expect("Unable to write preVol file");
        // fade out
        // while get_vol(default_device.clone()) > Volume::MUTED {
        let mut current_vol = get_vol(default_device);
        // while get_vol(default_device.clone()).gt(&Volume::MUTED) {
        while current_vol.gt(&Volume::MUTED) {
            println!("Current volume: {}", current_vol);
            // handler.decrease_device_volume_by_percent(device_index, 4.0);
            dec_vol(handler, device_index, Some(4.0));
        };
        handler.set_device_mute_by_index(device_index, true);
    } else {
        handler.set_device_mute_by_index(device_index, false);
        // fade in
        // while get_vol(default_device) <= Volume::from_u32(pre_vol) {
        //     handler.increase_device_volume_by_percent(device_index, 2.5);
        //     inc_vol(handler, device_index, Some(2.5));
        // };
    };
}

fn main() {

    let args = Args::parse();

    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let default_device = handler
        .get_default_device()
        .expect("Could not get default playback device.");
    
    println!("{}", default_device.volume.avg());
    println!("vol_string: {}", get_vol_string(default_device.clone()));
    println!("Is get_vol() working? Lets see: {}\n", get_vol(default_device.clone()));
    println!("This is Volume::MUTED {}", Volume::MUTED);
    println!("This is Volume::NORMAL {}", Volume::NORMAL);
    println!("This is Volume::default() {}", Volume::default());

    if args.increase {
        inc_vol(&mut handler, default_device.index, Some(0.01375));
        println!("Volume increased.");
    } else if args.decrease {
        dec_vol(&mut handler, default_device.index, Some(0.017));
        println!("Volume decreased.");
    } else if args.mute {
        mute_unmute(&mut handler, default_device.index);
        println!("Volume muted/unmuted with fade in/out.");
    } else {
        println!("No action specified. Use --increase, --decrease, or --mute.");
    }



/*
    let mut server_info = SinkController::get_server_info(&mut handler);
    println!("User Name: {:?}", server_info.as_mut().unwrap().user_name);
    println!("Host Name: {:?}", server_info.as_mut().unwrap().host_name);
    println!("Server Version: {:?}", server_info.as_mut().unwrap().server_version);
    println!("Server Name: {:?}", server_info.as_mut().unwrap().server_name);
    println!("Sample Spec: {:?}", server_info.as_mut().unwrap().sample_spec);
    println!("Default Sink Name: {:?}", server_info.as_mut().unwrap().default_sink_name);
    println!("Default Source Name: {:?}", server_info.as_mut().unwrap().default_source_name);
    println!("Cookie: {:?}", server_info.as_mut().unwrap().cookie);
    println!("Default channel map: {:?}", server_info.as_mut().unwrap().channel_map);
*/

/*
    println!("\n\nDefault Device: ");
    println!(
        "[Index: {}], [Name: {}], [Description: {}], [Volume: {}]\n\n",
        default_device.index,
        default_device.name.as_ref().unwrap(),
        default_device.description.as_ref().unwrap(),
        default_device.volume
    );
*/
}
