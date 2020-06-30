#include <linux/vt.h>
#include <stdio.h>
#include <unistd.h>
#include <termios.h>

void stdout_into_raw_mode() {
    struct termios t;
    tcdrain(1);
    tcgetattr(1, &t);
    cfmakeraw(&t);
    tcsetattr(1, 0, &t);
}
