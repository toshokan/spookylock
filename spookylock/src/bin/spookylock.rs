use clap::Clap;
use spookylock_sys::vt;
use std::os::unix::{
    io::{AsRawFd, FromRawFd, RawFd},
    process::CommandExt,
};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Clap)]
struct Options {
    #[clap(short, long)]
    relative_interface: bool,
    #[clap(short, long, default_value = "spookylock-interface")]
    interface_path: PathBuf,
}

fn make_stdio(fd: RawFd) -> (Stdio, Stdio, Stdio) {
    let make_one = || unsafe { std::process::Stdio::from_raw_fd(fd) };
    (make_one(), make_one(), make_one())
}

fn main() -> std::io::Result<()> {
    let opts = Options::parse();

    let console = vt::Console::acquire()?;
    let target = console.new_vt();

    console.on_vt_locked(&target, || {
        let stream = target.stream()?;
        let fd = stream.as_raw_fd();

        let (stdin, stdout, stderr) = make_stdio(fd);

        let path = if opts.relative_interface {
            opts.interface_path
                .canonicalize()
                .expect("Failed to find interface binary")
        } else {
            opts.interface_path.clone()
        };

        let user = std::env::var("USER").expect("Failed to get current user");

        unsafe {
            Command::new(path)
                .arg("--user")
                .arg(user)
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
