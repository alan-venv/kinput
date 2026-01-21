# Kinput

Low-level Rust library for input injection and global key capture on Linux.

Creates and manages virtual keyboard and mouse devices directly in the kernel input subsystem, independent of distribution or any graphical environment. Includes a reader for global key events from real input devices.

## Features

- Kernel-level virtual input devices
- Keyboard and mouse support (real-hardware equivalent)
- Global key capture
- No graphical or compositor dependencies
- Minimal, low-level API

## Usage (Input)

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

## Usage (Reader)

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

## Scope

* Bots
* Automated testing
* Accessibility tooling
* Device emulation
