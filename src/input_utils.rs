use device_query::{DeviceQuery, DeviceState, Keycode};

pub struct InputManager {
    device_state: DeviceState,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
        }
    }

    pub fn is_left_key_pressed(&self) -> bool {
        self.device_state.get_keys().contains(&Keycode::Left)
    }

    pub fn is_right_key_pressed(&self) -> bool {
        self.device_state.get_keys().contains(&Keycode::Right)
    }

    fn is_control_key_pressed(&self) -> bool {
        let keys = self.device_state.get_keys();
        keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl)
    }

    fn is_shift_key_pressed(&self) -> bool {
        let keys = self.device_state.get_keys();
        keys.contains(&Keycode::LShift) || keys.contains(&Keycode::RShift)
    }

    fn is_d_key_pressed(&self) -> bool {
        self.device_state.get_keys().contains(&Keycode::D)
    }

    fn is_v_key_pressed(&self) -> bool {
        self.device_state.get_keys().contains(&Keycode::V)
    }

    pub fn is_control_v_pressed(&self) -> bool {
        self.is_control_key_pressed() && self.is_v_key_pressed()
    }

    pub fn is_control_shift_d_pressed(&self) -> bool {
        self.is_control_key_pressed() && self.is_shift_key_pressed()  && self.is_d_key_pressed()
    }
}