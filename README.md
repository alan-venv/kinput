# kinput

Low-level Rust library for input injection on Linux.

Creates and manages virtual keyboard and mouse devices directly in the kernel input subsystem, independent of distribution or any graphical environment.

---

## Features

- Kernel-level virtual input devices
- Keyboard and mouse support
- Global input delivery (real-hardware equivalent)
- No graphical or compositor dependencies
- Minimal, low-level API

---

## Usage

```rust
use kinput::{InputDevice, Key::*};

fn main() {
    let device = InputDevice::new();

    device.keyboard.text([H, E, L, L, O, Space, W, O, R, L, D]);

    device.mouse.reset_axis();
    device.mouse.move_relative(500, 350);
    device.mouse.left_click();
}
```

---

## Setup script for non-root user
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
Logout and login required.

---

## Scope

* Bots
* Automated testing
* Accessibility tooling
* Device emulation
