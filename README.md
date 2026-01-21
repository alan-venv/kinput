# ⌨️ Kinput

**Low-level Rust library for input injection and global key capture on Linux.**

Creates virtual devices and captures global input events directly via the kernel subsystem, operating independently of any graphical environment or display server.

## ✨ Features

- 🐧 **Kernel-Level:** Native virtual device management.
- ⌨️ **Full Input Support:** Keyboard and mouse (relative/absolute) events injection.
- 🎧 **Global Capture:** Reads events from physical devices regardless of window focus.
- 🚫 **Headless:** No dependencies on X11, Wayland, or compositors.
- ⚡ **Minimal API:** Simple, idiomatic Rust interface.

## 🚀 Usage

### Injection

```rust
use kinput::{InputDevice, Key::*};

fn main() {
    let device = InputDevice::new();

    device.keyboard.text([H, E, L, L, O, Space, W, O, R, L, D]);

    // Relative movement.
    device.mouse.rel.reset_axis();
    device.mouse.rel.move_xy(500, 350);
    device.mouse.rel.left_click();

    // Absolute positioning.
    device.mouse.abs.move_xy(300, 300);
    device.mouse.abs.left_click();
}
```

### Capture

```rust
use kinput::{InputReader, Key::*};

fn main() {
    let mut reader = InputReader::new();
    reader.start().unwrap();

    while let Ok(key) = reader.receive() {
        if key == A {
            println!("A key pressed");
        }
    }
}
```

## 🔧 Non-Root Setup

Run this script to configure permissions and use the library without `sudo`.

```bash
#!/bin/bash

MODULE_GROUP=$(stat -c %G /dev/uinput 2>/dev/null || echo "root")
if [ "$MODULE_GROUP" = "input" ]; then
    sudo usermod -aG input "$USER"
else
    RULE_FILE="/etc/udev/rules.d/99-uinput.rules"
    RULE_CONTENT="KERNEL==\"uinput\", MODE=\"0660\", GROUP=\"input\""

    echo "$RULE_CONTENT" | sudo tee "$RULE_FILE" > /dev/null
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    sudo usermod -aG input "$USER"
fi

```

> **Note:** A system logout/login is required for changes to take effect.

## 🎯 Scope

* 🤖 **Bots & Automation**
* 🧪 **Automated Testing**
* ♿ **Accessibility Tooling**
* 🎮 **Device Emulation**
