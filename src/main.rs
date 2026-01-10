use ctru::{
    applets::swkbd::{Button, SoftwareKeyboard},
    prelude::*,
};

use crate::irc::{
    constants::{IRC_CHANNEL, IRC_HOST, IRC_NICK},
    server::IrcServer,
};

mod irc;

fn main() {
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    ctru::set_panic_hook(true);

    let top_screen = Console::new(gfx.top_screen.borrow_mut());
    let bottom_screen = Console::new(gfx.bottom_screen.borrow_mut());
    let mut keyboard = SoftwareKeyboard::default();
    bottom_screen.select();
    println!("welcome to grairc!");
    println!("START | exit");
    println!("(A)   | send a message");

    top_screen.select();
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
        } else if hid.keys_down().contains(KeyPad::A) {
            match keyboard.launch(&apt, &gfx) {
                Ok((text, Button::Right)) => {
                    _irc_server.irc_send(&text, IRC_CHANNEL).unwrap();
                }
                Ok((_, Button::Left)) => {}
                Ok((_, Button::Middle)) => {}
                Err(e) => println!("Error launching keyboard: {:?}", e),
            }
        }

        _irc_server.handler().unwrap();
    }
}
