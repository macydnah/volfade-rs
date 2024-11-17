use pulsectl::controllers::DeviceControl;
use pulsectl::controllers::SinkController;

fn dec_vol() {
}

fn inc_vol(mut handler: SinkController, device_index: u32) {
    handler.increase_device_volume_by_percent(device_index, 0.05);
}

fn mute_unmute() {
}

fn main() {
    // create handler that calls functions on playback devices and apps
    let handler = SinkController::create().unwrap();

    let device_index = 0;

    inc_vol(handler, device_index);
}
