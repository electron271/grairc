use std::io::Write;

use anyhow::Error;
use ctru::applets::swkbd::{ButtonConfig, CallbackResult, Features, Kind, SoftwareKeyboard};
use serde::{Deserialize, Serialize};

use crate::grairc::Grairc;

pub const CONFIG_FILE: &str = "/3ds/grairc/config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// nickname to use in IRC
    /// if taken a number will be appended to it
    pub nickname: String,

    /// irc server info
    pub server_hostname: String,
    pub server_port: u16,

    /// list of autojoin channels
    pub autojoin_channels: Vec<String>,
}

impl Config {
    /// this config wont work for connecting, just a placeholder
    pub fn default() -> Self {
        Config {
            nickname: "null".to_string(),
            server_hostname: "null".to_string(),
            server_port: 0,
            autojoin_channels: vec!["#null".to_string()],
        }
    }

    fn keyboard(
        &self,
        hint_text: &str,
        mut keyboard: SoftwareKeyboard,
        features: Option<Features>,
        grairc: &mut Grairc,
    ) -> Result<String, Error> {
        keyboard.set_hint_text(Some(hint_text));
        if let Some(f) = features {
            keyboard.set_features(f);
        }
        let result = keyboard.launch(grairc.apt, grairc.gfx)?;
        Ok(result.0)
    }

    pub fn setup_wizard(&self, grairc: &mut Grairc) -> Self {
        let nickname = self
            .keyboard(
                "enter irc nickname",
                SoftwareKeyboard::new(Kind::Normal, ButtonConfig::Right),
                None,
                grairc,
            )
            .expect("Failed to get nickname input");

        let server_hostname = self
            .keyboard(
                "enter irc server hostname (no port, e.g. irc.example.com)",
                SoftwareKeyboard::new(Kind::Normal, ButtonConfig::Right),
                None,
                grairc,
            )
            .expect("Failed to get server hostname input");

        let mut server_port_keyboard = SoftwareKeyboard::new(Kind::Numpad, ButtonConfig::Right);
        server_port_keyboard.set_filter_callback(Some(Box::new(move |input| {
            if input.parse::<u16>().is_ok() {
                CallbackResult::Ok
            } else {
                CallbackResult::Retry("Please enter a valid port number".into())
            }
        })));
        let server_port = self
            .keyboard(
                "enter irc server port (e.g. 6667)",
                server_port_keyboard,
                None,
                grairc,
            )
            .expect("Failed to get server port input")
            .parse::<u16>()
            .expect("Failed to parse server port input");

        let autojoin_channels = self
            .keyboard(
                "enter autojoin channels (one per line, e.g. #general)",
                SoftwareKeyboard::new(Kind::Normal, ButtonConfig::Right),
                Some(Features::MULTILINE),
                grairc,
            )
            .expect("Failed to get autojoin channels input")
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        Config {
            nickname,
            server_hostname,
            server_port,
            autojoin_channels,
        }
    }

    pub fn load() -> Option<Self> {
        toml::from_str(&std::fs::read_to_string(CONFIG_FILE).ok()?).ok()
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let path = std::path::Path::new(CONFIG_FILE);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let contents = toml::to_string_pretty(self)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}
