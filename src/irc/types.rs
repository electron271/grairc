#[derive(PartialEq, Eq)]
pub struct IrcChannel {
    pub selected: bool,
    pub name: String,
    pub users: Vec<String>,
    pub messages: Vec<IrcMessage>,
    pub channel_type: IrcChannelType,
}

#[derive(PartialEq, Eq)]
pub struct IrcMessage {
    /// none for system messages like in a system channel or joins/leaves/etc
    pub nick: Option<String>,
    pub content: String,
}

#[derive(PartialEq, Eq)]
pub enum IrcChannelType {
    System,
    Channel,
    PrivateMessage,
}
