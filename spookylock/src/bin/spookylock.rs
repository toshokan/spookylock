use spookylock_sys::vt;
use std::os::unix::{
    io::{AsRawFd, FromRawFd, RawFd},
    process::CommandExt,
};
use std::process::{Command, Stdio};

fn make_stdio(fd: RawFd) -> (Stdio, Stdio, Stdio) {
    let make_one = || unsafe { std::process::Stdio::from_raw_fd(fd) };
    (make_one(), make_one(), make_one())
}

fn main() -> std::io::Result<()> {
    let console = vt::Console::acquire()?;
    let target = console.new_vt();

    console.on_vt_locked(&target, || {
        let stream = target.stream()?;
        let fd = stream.as_raw_fd();

        let (stdin, stdout, stderr) = make_stdio(fd);

        unsafe {
            Command::new("/home/toshokan/dev/rust/spookylock/target/debug/spookylock-interface")
                .stdin(stdin)
                .stdout(stdout)
                .stderr(stderr)
                .pre_exec(move || spookylock_sys::vt::set_controlling_tty(fd))
                .spawn()?
                .wait()?;
        }

        Ok(())
    })
}
