use std::io::{Read, Result, Write};
use std::marker::PhantomData;
use std::os::unix::io::AsRawFd;

pub struct Guard<'sw, T> {
    old_fd: i32,
    temp_fd: i32,
    _m: PhantomData<*const ()>,
    _m2: PhantomData<&'sw T>,
}

fn check_minus_one_errno(ret: i32) -> Result<i32> {
    match ret {
        -1 => Err(std::io::Error::last_os_error()),
        _ => Ok(ret),
    }
}

impl<'sw, T> Guard<'sw, T> {
    fn acquire_impl(new_fd: i32, old_fd: i32) -> Result<Self> {
        let temp_fd = {
            let t = check_minus_one_errno(unsafe { libc::dup(old_fd) })?;
            unsafe {
                check_minus_one_errno(libc::dup2(new_fd, old_fd))?;
            };
            t
        };

        Ok(Self {
            old_fd,
            temp_fd,
            _m: PhantomData,
            _m2: PhantomData,
        })
    }
}

impl<'sw, T: AsRawFd + Read> Guard<'sw, T> {
    pub fn acquire_read(new: &'sw T, old: &'sw (impl Read + AsRawFd)) -> Result<Self> {
        let new_fd = new.as_raw_fd();
        let old_fd = old.as_raw_fd();

        Self::acquire_impl(new_fd, old_fd)
    }
}

impl<'sw, T: AsRawFd + Write> Guard<'sw, T> {
    pub fn acquire_write(new: &'sw T, old: &'sw (impl Write + AsRawFd)) -> Result<Self> {
        let new_fd = new.as_raw_fd();
        let old_fd = old.as_raw_fd();

        Self::acquire_impl(new_fd, old_fd)
    }
}

impl<'sw, T> Drop for Guard<'sw, T> {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.temp_fd, self.old_fd);
        }
    }
}

pub fn do_tty(fd: i32) -> Result<i32> {
    unsafe {
	check_minus_one_errno(
	    libc::setsid()
	)?;
	check_minus_one_errno(
	    libc::ioctl(fd, libc::TIOCSCTTY, 0)
	)
    }
}
