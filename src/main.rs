use std::{
    process,
    thread::sleep,
    time::{Duration, Instant},
};

// sudo modprobe uinput

use bytemuck::cast_slice;
use evdev::{Device as evDevice, InputEventKind};
use gilrs::{Axis, Event, GamepadId, Gilrs};
use hidapi::{HidApi, HidDevice};
use uinput::event::{self, keyboard};
struct ControllerData {
    real_device: HidDevice,
    tile_angle: f32,
    max_tilt: f32,
    virtual_input_device: uinput::Device,
}
impl ControllerData {
    pub fn new() -> Self {
        let api = HidApi::new().expect("Failed to create HID API instance.");
        let controller = api.open(1356, 2508).expect("Error opening controller");

        let input_device = uinput::default()
            .unwrap()
            .name("Virtual Gamepad")
            .unwrap()
            .event(uinput::event::absolute::Position::X)
            .unwrap()
            .max(255)
            .min(0)
            .event(uinput::event::controller::GamePad::North)
            .unwrap()
            // .flat(1)
            .create()
            .unwrap();

        ControllerData {
            real_device: controller,
            tile_angle: 0.0,
            max_tilt: 70.0,
            virtual_input_device: input_device,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new().expect("Failed to create HID API instance.");
    // println!("{:#?}", api.device_list().collect::<Vec<_>>());
    let controller = api.open(1356, 2508).expect("Error opening controller");
    let mut gilrs = Gilrs::new().unwrap();

    // println!("{}", my_gamepad.unwrap().id());

    let mut controller_data: ControllerData = ControllerData::new();
    let mut mem_buf = [0; 256];

    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }
    loop {
        let mut time = Instant::now();
        match controller.read(&mut mem_buf) {
            Ok(count) => parse_inputs(&mem_buf[..count], &mut controller_data, time),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
        // sleep(Duration::from_millis(150));
    }
    Ok(())
}

fn parse_inputs(mem_buf: &[u8], gamepad_data: &mut ControllerData, mut previous_time: Instant) {
    let sticks = format!(
        "Leftstick ({},{}) Rightstick ({},{})",
        mem_buf[1], mem_buf[2], mem_buf[3], mem_buf[4]
    );

    let gyro_z = i16::from_le_bytes([mem_buf[17], mem_buf[18]]) as f32;
    let accel_x = i16::from_le_bytes([mem_buf[19], mem_buf[20]]) as f32;
    let accel_y = i16::from_le_bytes([mem_buf[21], mem_buf[22]]) as f32;

    let delta_time = previous_time.elapsed().as_secs_f32();
    previous_time = Instant::now();

    gamepad_data.tile_angle += (gyro_z * delta_time);

    let tilt_angle = (accel_y.atan2(accel_x).to_degrees()).abs() - 90.0;

    let steering_input = (tilt_angle / gamepad_data.max_tilt).clamp(-1.0, 1.0);

    gamepad_data
        .virtual_input_device
        .position(
            &uinput::event::absolute::Position::X,
            ((steering_input * 126.0) + 126.0) as i32,
        )
        .unwrap();

    gamepad_data
        .virtual_input_device
        .click(&uinput::event::controller::GamePad::North)
        .unwrap();
    let _sync = gamepad_data.virtual_input_device.synchronize().unwrap();
    // test.s

    println!(
        " ({:0>3}) Steering Angle: {:>7.2}, Tilt Angle: {:>7.2}, Steering input: {:3>0.3}",
        mem_buf[1],
        gamepad_data.tile_angle,
        tilt_angle,
        (steering_input * 126.0) + 126.0
    );

    // println!("{} {}\n", sticks, gyro);
}
