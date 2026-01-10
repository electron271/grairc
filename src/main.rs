use ctru::prelude::*;

use crate::irc::{
    constants::{IRC_CHANNEL, IRC_HOST, IRC_NICK},
    server::IrcServer,
};

mod irc;

fn main() {
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());
    ctru::set_panic_hook(true);

    println!("Press Start to exit");
    println!("Initializing IRC client...");

    let mut _irc_server = IrcServer::new(IRC_HOST);

    println!("Identifying to IRC server...");
    _irc_server.irc_ident(IRC_NICK, IRC_CHANNEL).unwrap();

    println!("Entering main loop...");

    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        _irc_server.handler().unwrap();
    }
}
