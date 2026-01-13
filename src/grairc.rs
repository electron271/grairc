use ctru::{
    applets::swkbd::{Button, Features, SoftwareKeyboard},
    prelude::*,
    services::ptm::user::{BatteryLevel, PTMUser},
};

use crate::{
    config::Config,
    gfx::{eg::DisplayTargets, renderers::render},
    irc::{
        server::IrcServer,
        types::{IrcChannelType, IrcMessage},
    },
    state::{BatteryState, State},
};

pub struct Grairc<'a> {
    pub apt: &'a mut Apt,
    pub hid: &'a mut Hid,
    pub gfx: &'a Gfx,
    pub soc: &'a mut Soc,
    pub ptmu: &'a PTMUser,
    pub targets: DisplayTargets<'a>,
    pub state: State,
    pub running: bool,
}

impl<'a> Grairc<'a> {
    pub fn new(
        apt: &'a mut Apt,
        hid: &'a mut Hid,
        gfx: &'a Gfx,
        soc: &'a mut Soc,
        ptmu: &'a PTMUser,
    ) -> Self {
        Grairc {
            apt,
            hid,
            gfx,
            soc,
            ptmu,
            state: State::default(),
            targets: DisplayTargets::new(&gfx).expect("Failed to create display targets"),
            running: false,
        }
    }

    pub fn run(&mut self) {
        self.running = true;
        self.soc.redirect_to_3dslink(true, true).ok();
        self.apt.set_sleep_allowed(false);

        println!("\x1b[30;47mgrairc v{}\x1b[0m", env!("CARGO_PKG_VERSION"));

        println!("Loading configuration...");
        self.state.config = match Config::load() {
            Some(cfg) => {
                println!("Configuration loaded successfully");
                Some(cfg)
            }
            None => {
                println!("No configuration found, launching setup wizard...");
                let config = Config::default().setup_wizard(self);
                config.save().expect("Failed to save configuration");
                println!("Configuration saved successfully");
                Some(config)
            }
        };

        println!("Initializing IRC server connection...");
        let config = self.state.config.as_ref().unwrap();
        let mut irc_server =
            IrcServer::new(&config.server_hostname, &config.server_port, &self.soc);

        println!("Identifying to IRC server...");
        irc_server
            .irc_ident(&config.nickname, &config.autojoin_channels)
            .expect("Failed to identify to IRC server");

        println!("Entering main loop...");
        while self.running && self.apt.main_loop() {
            render(&mut self.targets, &self.state).expect("Render failed");
            self.targets.flush().expect("Failed to flush display");

            irc_server
                .handler(&mut self.state)
                .expect("IRC handler failed");

            self.hid.scan_input();
            match self.hid.keys_down() {
                keys if keys.contains(KeyPad::START) => {
                    self.running = false;
                }
                keys if keys.contains(KeyPad::DPAD_DOWN) => {
                    self.state.switch_channels(1);
                }
                keys if keys.contains(KeyPad::DPAD_UP) => {
                    self.state.switch_channels(-1);
                }
                keys if keys.contains(KeyPad::A) => {
                    let selected_channel = self.state.current_channel_static().clone();
                    if selected_channel.channel_type == IrcChannelType::System {
                        continue;
                    }

                    let mut keyboard = SoftwareKeyboard::default();
                    keyboard.set_features(Features::PREDICTIVE_INPUT);
                    match keyboard.launch(self.apt, self.gfx) {
                        Ok((text, Button::Right)) => {
                            let nickname = self.state.config.as_ref().unwrap().nickname.clone();
                            irc_server.irc_send(
                                &text,
                                &selected_channel.name,
                                &nickname,
                                &mut self.state,
                            )
                        }
                        Ok((_, Button::Left)) => Ok(()),
                        Ok((_, Button::Middle)) => Ok(()), // impossible to press
                        Err(e) => panic!("Software keyboard failed: {e}"),
                    }
                    .unwrap()
                }
                keys if keys.contains(KeyPad::X) => {
                    let mut keyboard = SoftwareKeyboard::default();
                    keyboard.set_features(Features::PREDICTIVE_INPUT);
                    match keyboard.launch(self.apt, self.gfx) {
                        Ok((text, Button::Right)) => irc_server.irc_raw_send(&text),
                        Ok((_, Button::Left)) => Ok(()),
                        Ok((_, Button::Middle)) => Ok(()), // impossible to press
                        Err(e) => panic!("Software keyboard failed: {e}"),
                    }
                    .unwrap()
                }
                _ => {}
            }

            match (
                self.ptmu.battery_level().unwrap(),
                self.ptmu.is_charging().unwrap(),
            ) {
                (_, true) => self.state.battery_state = BatteryState::Charging,
                (BatteryLevel::Critical, false) => {
                    self.state.battery_state = BatteryState::Critical
                }
                (BatteryLevel::Drained, false) => self.state.battery_state = BatteryState::Drained,
                (BatteryLevel::VeryLow, false) => self.state.battery_state = BatteryState::VeryLow,
                (BatteryLevel::Low, false) => self.state.battery_state = BatteryState::Low,
                (BatteryLevel::Medium, false) => self.state.battery_state = BatteryState::Medium,
                (BatteryLevel::High, false) => self.state.battery_state = BatteryState::High,
            }

            self.gfx.wait_for_vblank();
        }
    }
}
