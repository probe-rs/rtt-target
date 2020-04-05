use crate::UpChannel;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicPtr, Ordering};

static CRITICAL_SECTION: AtomicPtr<CriticalSectionFunc> = AtomicPtr::new(core::ptr::null_mut());
static PRINT_CHANNEL: AtomicPtr<crate::rtt::RttChannel> = AtomicPtr::new(core::ptr::null_mut());

/// Type-erased critical section function used to synchronize printing.
///
/// When called, the function must establish a critical section and call `f` within it, passing
/// `arg` as the argument.
pub type CriticalSectionFunc = fn(arg: *const (), f: fn(arg: *const ()) -> ()) -> ();

/// Sets the channel to use for [`rprint`] and [`rprintln`] and the critical section function used
/// to synchronize printing. You should only use this function if the [`set_print_channel`] function
/// isn't available on your platform.
///
/// # Safety
///
/// This function is unsafe because the user must guarantee that the `cs` function pointer passed in
/// adheres to the [`CriticalSectionFunc`] specification.
pub unsafe fn set_print_channel_cs(channel: UpChannel, cs: *const CriticalSectionFunc) {
    CRITICAL_SECTION.store(cs as *mut _, Ordering::SeqCst);
    PRINT_CHANNEL.store(channel.0, Ordering::SeqCst);
}

/// Sets the channel to use for [`rprint`] and [`rprintln`].
///
/// This function is available only if you have enabled a platform support feature. Otherwise,
/// [`set_print_channel_cs`] must be used.
#[cfg(feature = "cortex-m")]
pub fn set_print_channel(channel: UpChannel) {
    unsafe {
        set_print_channel_cs(
            channel,
            &((|a, f| cortex_m::interrupt::free(|_| f(a))) as CriticalSectionFunc) as *const _,
        );
    }
}

/// Public due to access from macro.
#[doc(hidden)]
pub mod print_impl {
    use super::*;

    fn with_print_channel<F: Fn(&mut UpChannel) -> ()>(f: F) {
        let cs = CRITICAL_SECTION.load(Ordering::SeqCst);

        if !cs.is_null() {
            unsafe {
                (&*cs)(&f as *const _ as *mut (), |f_ptr| {
                    let chan = PRINT_CHANNEL.load(Ordering::SeqCst);

                    if !chan.is_null() {
                        let f = &*(f_ptr as *const F);
                        f(&mut UpChannel(chan));
                    }
                });
            }
        }
    }

    /// Public due to access from macro.
    #[doc(hidden)]
    pub fn write_str(s: &str) {
        with_print_channel(|chan| {
            chan.write_str(s).ok();
        });
    }

    /// Public due to access from macro.
    #[doc(hidden)]
    pub fn write_fmt(arg: fmt::Arguments) {
        with_print_channel(|chan| {
            chan.write_fmt(arg).ok();
        });
    }
}

/// Prints to the print RTT channel. Works just like the standard `print`.
///
/// Before use the print channel has to be set with [`rtt_init_print`] or [`set_print_channel`]. If
/// the channel isn't set, the output is ignored without error.
#[macro_export]
macro_rules! rprint {
    ($s:expr) => {
        $crate::print_impl::write_str($s);
    };
    ($($arg:tt)*) => {
        $crate::print_impl::write_fmt(format_args!($($arg)*));
    };
}

/// Prints to the print RTT channel, with a newline. Works just like the standard `println`.
///
/// Before use the print channel has to be set with [`rtt_init_print`] or [`set_print_channel`]. If
/// the channel isn't set, the output is ignored without error.
#[macro_export]
macro_rules! rprintln {
    () => {
        $crate::print_impl::write_str("\n");
    };
    ($fmt:expr) => {
        $crate::print_impl::write_str(concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt(format_args!(concat!($fmt, "\n"), $($arg)*));
    };
}

/// Initializes RTT with a single up channel and sets it as the print channel for the printing
/// macros.
///
/// The optional arguments specify the blocking mode (default: `NoBlockTrim`) and size of the buffer
/// in bytes (default: 1024). See [`rtt_init`] for more details.
///
/// This macro is defined only if the [`set_print_channel`] function is available, i.e. if you have
/// enabled a platform support feature.
#[cfg(any(feature = "cortex-m"))]
#[macro_export]
macro_rules! rtt_init_print {
    ($mode:ident, $size:literal) => {
        let channels = $crate::rtt_init! {
            up: {
                0: {
                    size: $size
                    mode: $mode
                    name: "Terminal"
                }
            }
        };

        $crate::set_print_channel(channels.up.0);
    };

    ($mode:ident) => {
        $crate::rtt_init_print!($mode, 1024);
    };

    () => {
        $crate::rtt_init_print!(NoBlockTrim, 1024);
    };
}
