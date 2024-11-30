use std::{
    thread,
    time::{Duration, Instant},
};

const LEFT_TRIGGER_DEADZONE_ADJUST: f32 = 1.0;
const RIGHT_TRIGGER_DEADZONE_ADJUST: f32 = 1.1;

// sudo modprobe uinput
// sudo nano /etc/udev/rules.d/99-virtual-gamepad.rules
// KERNEL=="event*", SUBSYSTEM=="input", ATTRS{name}=="Virtual Gamepad", TAG+="uaccess", ENV{ID_INPUT_JOYSTICK}="1"

use gilrs::Gilrs;
use hidapi::{HidApi, HidDevice};
struct ControllerData {
    device: HidDevice,
    max_tilt: f32,
    virtual_input_device: uinput::Device,
    mem_buf: Box<[u8]>,
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
            .event(uinput::event::absolute::Position::Y)
            .unwrap()
            .event(uinput::event::absolute::Position::Z)
            .unwrap()
            .max(255)
            .min(0)
            .event(uinput::event::absolute::Position::RZ)
            .unwrap()
            .max(255)
            .min(0)
            .event(uinput::event::Controller::GamePad(
                uinput::event::controller::GamePad::North,
            ))
            .unwrap()
            .event(uinput::event::Controller::GamePad(
                uinput::event::controller::GamePad::East,
            ))
            .unwrap()
            .event(uinput::event::Controller::GamePad(
                uinput::event::controller::GamePad::South,
            ))
            .unwrap()
            .event(uinput::event::Controller::GamePad(
                uinput::event::controller::GamePad::West,
            ))
            .unwrap()
            .event(uinput::event::Controller::DPad(
                uinput::event::controller::DPad::Up,
            ))
            .unwrap()
            .event(uinput::event::Controller::DPad(
                uinput::event::controller::DPad::Right,
            ))
            .unwrap()
            .event(uinput::event::Controller::DPad(
                uinput::event::controller::DPad::Down,
            ))
            .unwrap()
            .event(uinput::event::Controller::DPad(
                uinput::event::controller::DPad::Left,
            ))
            .unwrap()
            .create()
            .unwrap();

        ControllerData {
            device: controller,
            max_tilt: 70.0,
            virtual_input_device: input_device,
            mem_buf: Box::new([0; 256]),
        }
    }

    fn read_data(&mut self) -> Result<usize, hidapi::HidError> {
        self.device.read(&mut self.mem_buf)
    }

    fn calculate_steering_angle(&mut self) -> f32 {
        let gyro_z = i16::from_le_bytes([self.mem_buf[17], self.mem_buf[18]]) as f32;
        let accel_x = i16::from_le_bytes([self.mem_buf[19], self.mem_buf[20]]) as f32;
        let accel_y = i16::from_le_bytes([self.mem_buf[21], self.mem_buf[22]]) as f32;

        let tilt_angle = (accel_y.atan2(accel_x).to_degrees()).abs() - 90.0;

        let steering_input = (tilt_angle / self.max_tilt).clamp(-1.0, 1.0);

        let _ = self.virtual_input_device.position(
            &uinput::event::absolute::Position::X,
            ((steering_input * 126.0) + 126.0) as i32,
        );

        steering_input
    }

    fn calculate_triggers(&mut self) -> (f32, f32) {
        let l_tr = (self.mem_buf[8] as f32 * LEFT_TRIGGER_DEADZONE_ADJUST)
            .floor()
            .clamp(0.0, 255.0);
        let r_tr = (self.mem_buf[9] as f32 * RIGHT_TRIGGER_DEADZONE_ADJUST)
            .floor()
            .clamp(0.0, 255.0);

        let _ = self
            .virtual_input_device
            .position(&uinput::event::absolute::Position::Z, l_tr as i32);
        let _ = self
            .virtual_input_device
            .position(&uinput::event::absolute::Position::RZ, r_tr as i32);
        // self

        (l_tr, r_tr)
    }

    fn check_face_buttons(&mut self) {
        let triangle: bool = (self.mem_buf[5] & 128) == 128;
        let circle: bool = (self.mem_buf[5] & 64) == 64;
        let x: bool = (self.mem_buf[5] & 32) == 32;
        let square: bool = (self.mem_buf[5] & 16) == 16;

        let w: bool = matches!(self.mem_buf[5], 5 | 6 | 7);
        let s: bool = matches!(self.mem_buf[5], 4 | 3 | 5);
        let e: bool = matches!(self.mem_buf[5], 2 | 1 | 3);
        let n: bool = matches!(self.mem_buf[5], 0 | 1 | 7);

        if triangle {
            self.virtual_input_device
                .press(&uinput::event::controller::GamePad::North)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::GamePad::North)
                .unwrap();
        }
        if circle {
            self.virtual_input_device
                .press(&uinput::event::controller::GamePad::East)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::GamePad::East)
                .unwrap();
        }
        if x {
            self.virtual_input_device
                .press(&uinput::event::controller::GamePad::South)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::GamePad::South)
                .unwrap();
        }
        if square {
            self.virtual_input_device
                .press(&uinput::event::controller::GamePad::West)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::GamePad::West)
                .unwrap();
        }

        if n {
            self.virtual_input_device
                .press(&uinput::event::controller::DPad::Up)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::DPad::Up)
                .unwrap();
        }
        if e {
            self.virtual_input_device
                .press(&uinput::event::controller::DPad::Right)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::DPad::Right)
                .unwrap();
        }

        if s {
            self.virtual_input_device
                .press(&uinput::event::controller::DPad::Down)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::DPad::Down)
                .unwrap();
        }

        if w {
            self.virtual_input_device
                .press(&uinput::event::controller::DPad::Left)
                .unwrap();
        } else {
            self.virtual_input_device
                .release(&uinput::event::controller::DPad::Left)
                .unwrap();
        }

        println!(
            "\n{:>5} {:>5} {:>5} {:>5} w:{:>5} s:{:>5} e:{:>5} n:{:>5}",
            triangle, circle, x, square, w, s, e, n
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new().expect("Failed to create HID API instance.");
    // println!("{:#?}", api.device_list().collect::<Vec<_>>());
    let controller = api.open(1356, 2508).expect("Error opening controller");
    let mut gilrs = Gilrs::new().unwrap();

    // println!("{}", my_gamepad.unwrap().id());

    let mut controller_data: ControllerData = ControllerData::new();

    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }
    loop {
        // sleep(Duration::from_millis(10));
        match controller_data.read_data() {
            Ok(_) => parse_inputs(&mut controller_data),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn parse_inputs(gamepad_data: &mut ControllerData) {
    let steering_input = gamepad_data.calculate_steering_angle();
    gamepad_data.check_face_buttons();
    let triggers = gamepad_data.calculate_triggers();
    // gamepad_data
    //     .virtual_input_device
    //     .click(&uinput::event::controller::GamePad::North)
    //     .unwrap();
    let _sync = gamepad_data.virtual_input_device.synchronize().unwrap();

    println!(
        " ({:0>3}) Steering input: {:3>0.3}, Button Bit: {} Ltr {} Rtr {}",
        gamepad_data.mem_buf[1],
        (steering_input * 126.0) + 126.0,
        gamepad_data.mem_buf[5],
        triggers.0,
        triggers.1
    );

    // println!("{} {}\n", sticks, gyro);
}
