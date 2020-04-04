#![no_std]

use core::convert::Infallible;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicPtr, Ordering};

use ufmt_write::uWrite;

#[doc(hidden)]
pub mod implementation;

#[macro_use]
mod init;

/// RTT up (target to host) channel
///
/// Supports writing binary data directly, or writing strings via `core::fmt` macros such as
/// `write!` as well as the ufmt crate's `uwrite!` macros.
pub struct UpChannel(*mut implementation::RttChannel);

unsafe impl Send for UpChannel {}

impl UpChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut implementation::RttChannel) -> Self {
        UpChannel(channel)
    }

    fn channel(&mut self) -> &mut implementation::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Writes up to `buf.len()` bytes to the channel and returns the number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.channel().write(buf)
    }

    fn write_str(&mut self, mut s: &str) {
        while s.len() > 0 {
            let count = self.channel().write(s.as_bytes());

            s = &s[count..];
        }
    }
}

impl Write for UpChannel {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write_str(s);

        Ok(())
    }
}

impl uWrite for UpChannel {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.write_str(s);

        Ok(())
    }
}

/// RTT down (target to host) channel
pub struct DownChannel(*mut implementation::RttChannel);

unsafe impl Send for DownChannel {}

impl DownChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut implementation::RttChannel) -> Self {
        DownChannel(channel)
    }

    fn channel(&mut self) -> &mut implementation::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Reads up to `buf.len()` bytes from the channel and return the number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.channel().read(buf)
    }
}

static PRINT_CHANNEL: AtomicPtr<implementation::RttChannel> = AtomicPtr::new(core::ptr::null_mut());

pub fn set_print_channel(channel: UpChannel) {
    PRINT_CHANNEL.store(channel.0, Ordering::SeqCst);
}

fn get_print_channel() -> Option<UpChannel> {
    let p = PRINT_CHANNEL.load(Ordering::SeqCst);

    if p.is_null() {
        None
    } else {
        Some(UpChannel(p))
    }
}

/// Public due to access from macro.
#[doc(hidden)]
pub fn print_write_str(s: &str) {
    if let Some(mut chan) = get_print_channel() {
        chan.write_str(s);
    }
}

/// Public due to access from macro.
#[doc(hidden)]
pub fn print_write_fmt(arg: fmt::Arguments) {
    if let Some(mut chan) = get_print_channel() {
        chan.write_fmt(arg).ok();
    }
}

/// Prints to the print RTT channel.
///
/// Before use the print channel has to be set via either [set_print_channel] or [rtt_init_default].
/// If the channel isn't set, the output is ignored without error.
#[macro_export]
macro_rules! rprint {
    ($s:expr) => {
        $crate::print_write_str($s);
    };
    ($($arg:tt)*) => {
        $crate::print_write_fmt(format_args!($($arg)*));
    };
}

/// Prints to the print RTT channel, with a newline.
///
/// Before use the print channel has to be set via either [set_print_channel] or [rtt_init_default].
/// If the channel isn't set, the output is ignored without error.
#[macro_export]
macro_rules! rprintln {
    () => {
        $crate::print_write_str("\n");
    };
    ($fmt:expr) => {
        $crate::print_write_str(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print_write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*));
    };
}
