mod sys {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    include!(concat!(env!("OUT_DIR"), "/vt_bindings.rs"));
}

use std::fs::File;
use std::marker::PhantomData;
use std::os::unix::io::RawFd;

pub fn set_controlling_tty(fd: RawFd) -> std::io::Result<()> {
    let ret = unsafe {
        libc::setsid();
        libc::ioctl(fd, libc::TIOCSCTTY, 0)
    };
    if ret == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Console {
    inner: Inner,
}

#[derive(Debug)]
pub struct Inner {
    f: File,
}

impl Inner {
    pub fn acquire() -> std::io::Result<Self> {
        File::open("/dev/console").map(|f| Self { f })
    }

    pub fn fd(&self) -> i32 {
        use std::os::unix::io::AsRawFd;
        self.f.as_raw_fd()
    }
}

impl Console {
    pub fn acquire() -> std::io::Result<Self> {
        Inner::acquire().map(|inner| Self { inner })
    }

    pub fn current_vt(&self) -> Vt {
        self.inner.get_current_vt()
    }

    pub fn new_vt(&self) -> Vt {
        self.inner.get_next_vt(self)
    }

    pub fn lock_switch(&self) -> VtLockGuard {
        VtLockGuard::acquire(self)
    }

    pub fn on_vt<T>(&self, target: &Vt, action: impl Fn() -> T) -> T {
        let current = self.current_vt();
        let _guard = VtSwitchGuard::acquire(self, &current, &target);

        action()
    }

    pub fn on_vt_locked<T>(&self, target: &Vt, action: impl Fn() -> T) -> T {
        self.on_vt(target, || {
            let _lock = self.lock_switch();
            action()
        })
    }
}

pub struct VtLockGuard<'c> {
    console: &'c Console,
}

impl<'c> VtLockGuard<'c> {
    fn acquire(console: &'c Console) -> Self {
        console.inner.lockswitch();
        Self { console }
    }
}

impl<'c> Drop for VtLockGuard<'c> {
    fn drop(&mut self) {
        self.console.inner.unlockswitch();
    }
}

pub struct VtSwitchGuard<'c, 'vt> {
    console: &'c Console,
    from: &'vt Vt<'c>,
}

impl<'c, 'vt> VtSwitchGuard<'c, 'vt> {
    fn acquire(console: &'c Console, from: &'vt Vt<'c>, to: &'vt Vt<'c>) -> Self {
        console.inner.activate(to);
        console.inner.wait_active(to);

        Self { console, from }
    }
}

impl<'c, 'vt> Drop for VtSwitchGuard<'c, 'vt> {
    fn drop(&mut self) {
        self.console.inner.activate(self.from);
        self.console.inner.wait_active(self.from)
    }
}

trait IoctlConsole {
    fn get_current_vt(&self) -> Vt;
    fn get_next_vt<'c>(&self, console: &'c Console) -> Vt<'c>;
    fn activate(&self, vt: &Vt);
    fn wait_active(&self, vt: &Vt);
    fn lockswitch(&self);
    fn unlockswitch(&self);
    unsafe fn disallocate(&self, vt: &Vt);
}

impl IoctlConsole for Inner {
    fn get_current_vt(&self) -> Vt {
        use std::convert::TryInto;

        let mut s = sys::vt_stat::default();
        unsafe {
            libc::ioctl(
                self.fd(),
                sys::VT_GETSTATE.into(),
                &mut s as *mut sys::vt_stat,
            );
        }
        Vt::with_number(VtNumber(s.v_active.try_into().unwrap()))
    }

    fn get_next_vt<'c>(&self, console: &'c Console) -> Vt<'c> {
        let mut n: libc::c_int = 0;
        unsafe {
            libc::ioctl(
                self.fd(),
                sys::VT_OPENQRY.into(),
                &mut n as *mut libc::c_int,
            );
        }

        Vt::allocate_with_number(console, VtNumber(n))
    }

    fn activate(&self, vt: &Vt) {
        let n: libc::c_int = vt.number.0;
        unsafe {
            libc::ioctl(self.fd(), sys::VT_ACTIVATE.into(), n);
        }
    }

    fn wait_active(&self, vt: &Vt) {
        let n: libc::c_int = vt.number.0;
        unsafe {
            libc::ioctl(self.fd(), sys::VT_WAITACTIVE.into(), n);
        }
    }

    fn lockswitch(&self) {
        unsafe {
            libc::ioctl(self.fd(), sys::VT_LOCKSWITCH.into(), 1);
        }
    }

    fn unlockswitch(&self) {
        unsafe {
            libc::ioctl(self.fd(), sys::VT_UNLOCKSWITCH.into(), 1);
        }
    }

    unsafe fn disallocate(&self, vt: &Vt) {
        let n: libc::c_int = vt.number.0;
        libc::ioctl(self.fd(), sys::VT_DISALLOCATE.into(), n);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VtNumber(i32);

#[derive(Debug)]
pub struct Vt<'c> {
    number: VtNumber,
    allocated_by: Option<&'c Console>,
}

impl<'c> Vt<'c> {
    pub fn allocate_with_number(console: &'c Console, number: VtNumber) -> Self {
        Self {
            number,
            allocated_by: Some(console),
        }
    }

    pub fn with_number(number: VtNumber) -> Self {
        Self {
            number,
            allocated_by: None,
        }
    }

    pub fn stream(&self) -> std::io::Result<VtStream> {
        VtStream::from_vt(self)
    }
}

impl<'c> Drop for Vt<'c> {
    fn drop(&mut self) {
        if let Some(console) = self.allocated_by {
            unsafe {
                console.inner.disallocate(self);
            }
        }
    }
}

#[derive(Debug)]
pub struct VtStream<'vt> {
    file: std::fs::File,
    _vt: PhantomData<&'vt ()>,
}

impl<'vt> VtStream<'vt> {
    pub fn with_file(file: File) -> Self {
        Self {
            file,
            _vt: PhantomData,
        }
    }

    pub fn from_vt(vt: &'vt Vt) -> std::io::Result<Self> {
        let path = format!("/dev/tty{}", vt.number.0);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)?;

        Ok(Self::with_file(file))
    }
}

impl<'vt> std::os::unix::io::AsRawFd for VtStream<'vt> {
    fn as_raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
}
