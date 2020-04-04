#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt2::rtt_init;

#[entry]
fn main() -> ! {
    let channels = rtt_init! {
        up: {
            0: {
                size: 512
                name: "Output zero"
            }
            1: {
                size: 512
                name: "Output one"
            }
        }
        down: {
            0: {
                size: 64
                name: "Input zero"
            }
        }
    };

    let mut output0 = channels.up.0;
    let mut output1 = channels.up.1;
    let mut i = 0;

    loop {
        writeln!(output0, "Channel 0: {}", i).ok();
        writeln!(output1, "Channel 1: {}", i).ok();

        i += 1;
    }
}
