use crate::{
    config::Config,
    irc::types::{IrcChannel, IrcChannelType},
};

pub struct State {
    pub config: Option<Config>,
    pub channels: Vec<IrcChannel>,
    pub battery_state: BatteryState,
}

pub enum BatteryState {
    Charging,
    Critical,
    Drained,
    High,
    Low,
    Medium,
    VeryLow,
}

impl Default for State {
    fn default() -> Self {
        State {
            config: None,
            channels: vec![IrcChannel {
                selected: true,
                name: "<system>".to_string(),
                users: vec![],
                messages: vec![],
                channel_type: IrcChannelType::System,
            }],
            battery_state: BatteryState::Drained,
        }
    }
}

impl State {
    pub fn switch_channels(&mut self, change: isize) {
        let len = self.channels.len() as isize;
        if len == 0 {
            return;
        }

        let current_index = self.channels.iter().position(|ch| ch.selected).unwrap_or(0) as isize;

        let new_index = (current_index + change + len) % len;

        for ch in &mut self.channels {
            ch.selected = false;
        }
        self.channels[new_index as usize].selected = true;
    }

    pub fn current_channel(&mut self) -> &mut IrcChannel {
        self.channels.iter_mut().find(|ch| ch.selected).unwrap()
    }

    /// 99% chance theres a better way to do this
    pub fn current_channel_static(&self) -> &IrcChannel {
        self.channels.iter().find(|ch| ch.selected).unwrap()
    }

    pub fn get_system_channel(&mut self) -> &mut IrcChannel {
        self.channels
            .iter_mut()
            .find(|ch| ch.channel_type == IrcChannelType::System)
            .unwrap()
    }

    pub fn get_channel_by_name(&mut self, name: &str) -> Option<&mut IrcChannel> {
        self.channels.iter_mut().find(|ch| ch.name == name)
    }
}
