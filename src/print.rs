use crate::{TerminalChannel, TerminalWriter, UpChannel};
use core::fmt::{self, Write as _};
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

static CRITICAL_SECTION: AtomicPtr<CriticalSectionFunc> = AtomicPtr::new(core::ptr::null_mut());
static mut PRINT_TERMINAL: MaybeUninit<TerminalChannel> = MaybeUninit::uninit();

/// Type-erased critical section function used to synchronize printing.
///
/// When called, the function must establish a critical section and call `f` within it, passing
/// `arg` as the argument.
pub type CriticalSectionFunc = fn(arg: *mut (), f: fn(arg: *mut ()) -> ()) -> ();

/// Sets the channel to use for [`rprint`] and [`rprintln`] and the critical section function used
/// to synchronize printing. You should only use this function if the [`set_print_channel`] function
/// isn't available on your platform.
///
/// # Safety
///
/// This function is unsafe because the user must guarantee that the `cs` function pointer passed in
/// adheres to the [`CriticalSectionFunc`] specification.
pub unsafe fn set_print_channel_cs(channel: UpChannel, cs: *const CriticalSectionFunc) {
    (&*cs)(channel.0 as *mut (), |channel_ptr| {
        ptr::write(
            PRINT_TERMINAL.as_mut_ptr(),
            TerminalChannel::new(UpChannel(channel_ptr as *mut crate::rtt::RttChannel)),
        );
    });

    CRITICAL_SECTION.store(cs as *mut _, Ordering::SeqCst);
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

    fn with_writer<F: Fn(TerminalWriter) -> ()>(number: u8, f: F) {
        let cs = CRITICAL_SECTION.load(Ordering::SeqCst);

        if !cs.is_null() {
            // If the critical section pointer has been set, PRINT_TERMINAL must also have been set.

            let args = (number, f);

            unsafe {
                (&*cs)(&args as *const _ as *mut (), |args_ptr| {
                    let args = &*(args_ptr as *const (u8, F));
                    let term = &mut *PRINT_TERMINAL.as_mut_ptr();

                    (args.1)(term.write(args.0));
                });
            }
        }
    }

    /// Public due to access from macro.
    #[doc(hidden)]
    pub fn write_str(number: u8, s: &str) {
        with_writer(number, |mut w| {
            w.write_str(s).ok();
        });
    }

    /// Public due to access from macro.
    #[doc(hidden)]
    pub fn write_fmt(number: u8, arg: fmt::Arguments) {
        with_writer(number, |mut w| {
            w.write_fmt(arg).ok();
        });
    }
}

/// Prints to the print RTT channel. Works just like the standard `print`.
///
/// Before use the print channel has to be set with [`rtt_init_print`] or [`set_print_channel`]. If
/// the channel isn't set, the output is ignored without error.
///
/// The macro also supports output to multiple virtual terminals on the channel. Use the syntax
/// ```rprint!(=> 1, "Hello!");``` to write to terminal number 1, for example. Terminal numbers
/// range from 0 to 15.
#[macro_export]
macro_rules! rprint {
    ($s:expr) => {
        $crate::print_impl::write_str(0, $s);
    };
    ($($arg:tt)*) => {
        $crate::print_impl::write_fmt(0, format_args!($($arg)*));
    };
    (=> $terminal:expr, $s:expr) => {
        $crate::print_impl::write_str($terminal, $s);
    };
    (=> $terminal:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt($terminal, format_args!($($arg)*));
    };
}

/// Prints to the print RTT channel, with a newline. Works just like the standard `println`.
///
/// Before use the print channel has to be set with [`rtt_init_print`] or [`set_print_channel`]. If
/// the channel isn't set, the output is ignored without error.
///
/// The macro also supports output to multiple virtual terminals on the channel. Use the syntax
/// ```rprintln!(=> 1, "Hello!");``` to write to terminal number 1, for example. Terminal numbers
/// range from 0 to 15.
#[macro_export]
macro_rules! rprintln {
    () => {
        $crate::print_impl::write_str(0, "\n");
    };
    ($fmt:expr) => {
        $crate::print_impl::write_str(0, concat!($fmt, "\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt(0, format_args!(concat!($fmt, "\n"), $($arg)*));
    };
    (=> $terminal:expr) => {
        $crate::print_impl::write_str($terminal, "\n");
    };
    (=> $terminal:expr, $fmt:expr) => {
        $crate::print_impl::write_str($terminal, concat!($fmt, "\n"));
    };
    (=> $terminal:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::print_impl::write_fmt($terminal, format_args!(concat!($fmt, "\n"), $($arg)*));
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
