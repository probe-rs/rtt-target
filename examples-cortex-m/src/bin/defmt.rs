#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::rtt_init_defmt;

#[entry]
fn main() -> ! {
    rtt_init_defmt!();

    let mut i = 0;
    loop {
        defmt::println!("Loop {}...", i);

        i += 1;
    }
}
