### udev — overview

Access to `/dev/uinput` is controlled by **udev**, the Linux userspace device manager.
Whenever a kernel device appears, udev automatically applies owner, group, and permission policies.

Since `/dev/uinput` is created by the kernel, we define a **persistent udev rule specific to `/dev/uinput`**, setting the group to `input` and the mode to `0660`.
The user is then added to the `input` group. After logout/login, access works **without sudo**.
