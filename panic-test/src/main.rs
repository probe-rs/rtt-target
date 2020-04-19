#![no_main]

#![no_std]

use panic_rtt_target as _;
use rtt_target::rtt_init_default;

#[cortex_m_rt::entry]
fn main() -> ! {
    // you can use any init macro as long as it creates channel 0
    rtt_init_default!();

    panic!("Something has gone terribly wrong");
}