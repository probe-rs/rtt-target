#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt2::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut i = 0;
    loop {
        rprintln!("Hello from rprintln! {}", i);

        i += 1;
    }
}
