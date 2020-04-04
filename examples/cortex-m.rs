#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;

use rtt2::rtt_init_default;

#[entry]
fn main() -> ! {
    let channels = rtt_init_default!();

    let mut output = channels.up.0;
    let mut i = 0;

    loop {
        writeln!(output, "Hello from RTT in Rust! {}", i).ok();

        i += 1;
    }
}
