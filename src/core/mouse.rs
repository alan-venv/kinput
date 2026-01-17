use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use crate::core::device::Device;
use crate::types::constants::{BTN_LEFT, BTN_RIGHT};

/// Relative mouse for movement and clicks.
pub struct RelativeMouse {
    device: Rc<Device>,
}

impl RelativeMouse {
    /// Creates a `Mouse`.
    pub fn new(device: Rc<Device>) -> Self {
        return Self { device };
    }

    /// Left click.
    pub fn left_click(&self) {
        self.device.press(BTN_LEFT);
        sleep(Duration::from_millis(10));
        self.device.release(BTN_LEFT);
        sleep(Duration::from_millis(30));
    }

    /// Right click.
    pub fn right_click(&self) {
        self.device.press(BTN_RIGHT);
        sleep(Duration::from_millis(10));
        self.device.release(BTN_RIGHT);
        sleep(Duration::from_millis(30));
    }

    /// Moves cursor to top-left-ish.
    pub fn reset_axis(&self) {
        self.device.move_relative(-10000, -10000);
        sleep(Duration::from_millis(30));
    }

    /// Moves the mouse by a relative delta.
    pub fn move_xy(&self, x: i32, y: i32) {
        self.device.move_relative(x, y);
        sleep(Duration::from_millis(30));
    }
}

/// Absolute mouse for movement and clicks.
pub struct AbsoluteMouse {
    device: Rc<Device>,
}

impl AbsoluteMouse {
    /// Creates a `Mouse`.
    pub fn new(device: Rc<Device>) -> Self {
        return Self { device };
    }

    /// Left click.
    pub fn left_click(&self) {
        self.device.press(BTN_LEFT);
        sleep(Duration::from_millis(10));
        self.device.release(BTN_LEFT);
        sleep(Duration::from_millis(30));
    }

    /// Right click.
    pub fn right_click(&self) {
        self.device.press(BTN_RIGHT);
        sleep(Duration::from_millis(10));
        self.device.release(BTN_RIGHT);
        sleep(Duration::from_millis(30));
    }

    /// Moves cursor to 0x0.
    pub fn reset_axis(&self) {
        self.device.move_absolute(0, 0);
        sleep(Duration::from_millis(30));
    }

    /// Moves the mouse by a relative delta.
    pub fn move_xy(&self, x: i32, y: i32) {
        self.device.move_absolute(x, y);
        sleep(Duration::from_millis(30));
    }
}

/*
Ainda existe uma regra “de base” do EV_ABS no kernel: eixo absoluto tem estado, e o kernel so gera evento quando o valor muda.
O acontece com 2 devices:
- O seu device ABS tem um “estado interno” para ABS_X/ABS_Y.
- Quando voce chama move_absolute(300, 300), esse estado vira (300,300).
- Se depois voce move com o device REL, o cursor na tela muda, mas o estado do device ABS continua (300,300) (porque foi outro device que mexeu).
- Quando voce chama de novo move_absolute(300, 300), para o device ABS isso nao e “mover” (e o kernel tende a nem emitir evento se o valor nao mudou). Resultado: parece que “nao funciona para mesma coordenada anterior”.
 */
