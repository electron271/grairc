use ctru::{prelude::*, services::ptm::user::PTMUser};

use crate::grairc::Grairc;

pub mod config;
pub mod gfx;
pub mod grairc;
pub mod irc;
pub mod state;

fn main() {
    let mut apt = Apt::new().expect("Couldn't obtain APT controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    ctru::set_panic_hook(true);

    let mut soc = Soc::new().expect("Couldn't initialize SOC service");
    let mut ptmu = PTMUser::new().expect("Couldn't initialize PTM user service");

    Grairc::new(&mut apt, &mut hid, &gfx, &mut soc, &mut ptmu).run();
}
