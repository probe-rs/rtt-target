#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::rtt_init;

#[entry]
fn main() -> ! {
    let channels = rtt_init! {
        up: {
            0: {
                size: 1024,
                name: "defmt"
            }
        }
    };

    rtt_target::set_defmt_channel(channels.up.0);

    let mut i = 0;
    loop {
        defmt::println!("Loop {}...", i);

        i += 1;
    }
}
