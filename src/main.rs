use libpulse_binding::volume::Volume;
use std::{thread, time};
use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;
use pulsectl::controllers::types::DeviceInfo;

fn get_vol(handler: DeviceInfo) -> Volume {
    let avg_volume = handler.volume.avg();
    avg_volume
}

fn dec_vol(handler: &mut SinkController, device_index: u32) {
    let mut i = 0;
    while i <= 7 {
        handler.decrease_device_volume_by_percent(device_index, 0.017);
        let ms = time::Duration::from_millis(26);
        thread::sleep(ms);
        i += 1;
    };
}

fn inc_vol(handler: &mut SinkController, device_index: u32) {
    handler.set_device_mute_by_index(device_index, false);
    let mut i = 0;
    while i <= 7 {
        handler.increase_device_volume_by_percent(device_index, 0.01375);
        let ms = time::Duration::from_millis(26);
        thread::sleep(ms);
        i += 1;
    };
}

fn main() {
    // create handler that calls functions on playback devices and apps
    let mut handler = SinkController::create().unwrap();

    let default_device = handler
        .get_default_device()
        .expect("Could not get default playback device.");
    
    println!("Is get_vol() working? Lets see: {}\n", get_vol(default_device.clone()));
    // get_vol(&mut default_device);

    inc_vol(&mut handler, default_device.index);
    // dec_vol(&mut handler, default_device.index);

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
