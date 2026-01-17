#include <fcntl.h>
#include <linux/input.h>
#include <linux/uinput.h>
#include <string.h>
#include <sys/ioctl.h>
#include <unistd.h>

enum {
  SCREEN_WIDTH = 1920,
  SCREEN_HEIGHT = 1080,
};

static int abs_from_px(int px, int size_px) {
  if (size_px <= 1) {
    return 0;
  }

  if (px < 0) {
    px = 0;
  } else if (px > size_px - 1) {
    px = size_px - 1;
  }

  return (int)(((long long)px * 65535 + (size_px - 2) / 2) / (size_px - 1));
}

void emit(int fd, int type, int code, int val) {
  struct input_event ie;
  memset(&ie, 0, sizeof(ie)); // NEW

  ie.type = type;
  ie.code = code;
  ie.value = val;

  write(fd, &ie, sizeof(ie));
}

int main(void) {
  struct uinput_setup usetup;

  int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);

  /* enable mouse button left and absolute positioning */
  ioctl(fd, UI_SET_EVBIT, EV_KEY);
  ioctl(fd, UI_SET_KEYBIT, BTN_LEFT);

  ioctl(fd, UI_SET_EVBIT, EV_ABS);
  ioctl(fd, UI_SET_ABSBIT, ABS_X);
  ioctl(fd, UI_SET_ABSBIT, ABS_Y);

  /* define ABS axis ranges so userspace accepts ABS events */
  struct uinput_abs_setup abs;
  memset(&abs, 0, sizeof(abs));
  abs.code = ABS_X;
  abs.absinfo.minimum = 0;
  abs.absinfo.maximum = 65535;
  ioctl(fd, UI_ABS_SETUP, &abs);
  memset(&abs, 0, sizeof(abs));
  abs.code = ABS_Y;
  abs.absinfo.minimum = 0;
  abs.absinfo.maximum = 65535;
  ioctl(fd, UI_ABS_SETUP, &abs);
  // =====

  memset(&usetup, 0, sizeof(usetup));
  usetup.id.bustype = BUS_USB;
  usetup.id.vendor = 0x1234;  /* sample vendor */
  usetup.id.product = 0x5678; /* sample product */
  strcpy(usetup.name, "Example device");

  ioctl(fd, UI_DEV_SETUP, &usetup);
  ioctl(fd, UI_DEV_CREATE);

  /*
   * On UI_DEV_CREATE the kernel will create the device node for this
   * device. We are inserting a pause here so that userspace has time
   * to detect, initialize the new device, and can start listening to
   * the event, otherwise it will not notice the event we are about
   * to send. This pause is only needed in our example code!
   */
  sleep(1);

  /* Move the mouse to a specific pixel position (x,y) */
  emit(fd, EV_ABS, ABS_X, abs_from_px(1092, SCREEN_WIDTH));
  emit(fd, EV_ABS, ABS_Y, abs_from_px(47, SCREEN_HEIGHT));
  emit(fd, EV_SYN, SYN_REPORT, 0);
  usleep(15000);

  /*
   * Give userspace some time to read the events before we destroy the
   * device with UI_DEV_DESTROY.
   */
  sleep(1);

  ioctl(fd, UI_DEV_DESTROY);
  close(fd);

  return 0;
}
