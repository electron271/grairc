use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

use ctru::prelude::Soc;
use regex::Regex;

use crate::{
    irc::{
        regex::{JOIN_REGEX, PART_REGEX, PRIVMSG_REGEX, RPL_NAMREPLY_REGEX},
        types::{IrcChannel, IrcChannelType, IrcMessage},
    },
    state::State,
};

pub struct IrcServer<'a> {
    _addr: SocketAddr,
    stream: TcpStream,
    _soc_service: &'a Soc, // this always needs to be kept alive
}

impl<'a> IrcServer<'a> {
    pub fn new(hostname: &str, port: &u16, soc: &'a Soc) -> Self {
        let addr = format!("{}:{}", hostname, port)
            .to_socket_addrs()
            .expect("Invalid IRC hostname")
            .next()
            .expect("No addresses found for IRC hostname");
        println!("Connecting to IRC server at {}", addr);

        let stream = TcpStream::connect(addr).expect("Failed to connect to IRC server");
        println!("Setting stream to non-blocking mode");
        stream
            .set_nonblocking(true)
            .expect("set_nonblocking call failed");
        println!("Connected to the IRC socket successfully");

        IrcServer {
            _addr: addr,
            stream,
            _soc_service: soc,
        }
    }

    pub fn irc_ident(&mut self, nick: &str, channels: &[String]) -> Result<(), Error> {
        let user_cmd = format!("USER {} 0 * :{}\r\n", nick, nick);
        let nick_cmd = format!("NICK {}\r\n", nick);
        let join_cmd = channels
            .iter()
            .map(|ch| format!("JOIN {}\r\n", ch))
            .collect::<String>();

        self.stream.write_all(user_cmd.as_bytes())?;
        self.stream.write_all(nick_cmd.as_bytes())?;
        self.stream.write_all(join_cmd.as_bytes())?;

        Ok(())
    }

    pub fn irc_send(
        &mut self,
        message: &str,
        channel: &str,
        nick: &str,
        state: &mut State,
    ) -> Result<(), Error> {
        let msg_cmd = format!("PRIVMSG {} :{}\r\n", channel, message);
        self.stream.write_all(msg_cmd.as_bytes())?;

        state
            .get_channel_by_name(channel)
            .unwrap()
            .messages
            .push(IrcMessage {
                nick: Some(nick.to_string()),
                content: message.to_string(),
            });

        Ok(())
    }

    pub fn irc_raw_send(&mut self, message: &str) -> Result<(), Error> {
        self.stream
            .write_all(format!("{}\r\n", message).as_bytes())?;
        Ok(())
    }

    pub fn irc_handler(&mut self, message: &str, state: &mut State) {
        let privmsg_regex = Regex::new(PRIVMSG_REGEX).unwrap();
        let rpl_namreply_regex = Regex::new(RPL_NAMREPLY_REGEX).unwrap();
        let part_regex = Regex::new(PART_REGEX).unwrap();
        let join_regex = Regex::new(JOIN_REGEX).unwrap();

        match message {
            msg if msg.starts_with("PING") => {
                let response = msg.replace("PING", "PONG");
                self.stream
                    .write_all(response.as_bytes())
                    .expect("Failed to send PONG response");
            }

            caps if privmsg_regex.captures(caps).is_some() => {
                let captures = privmsg_regex.captures(caps).unwrap();
                let nick = captures.get(1).unwrap().as_str();
                let channel_name = captures.get(4).unwrap().as_str();
                let content = captures.get(5).unwrap().as_str();

                if !channel_name.starts_with('#') {
                    return;
                }

                if let Some(ch) = state.get_channel_by_name(channel_name) {
                    println!("{:12}: {}", nick, content);
                    ch.messages.push(IrcMessage {
                        nick: Some(nick.to_string()),
                        content: content.to_string(),
                    });
                }
            }

            caps if rpl_namreply_regex.captures(caps).is_some() => {
                let captures = rpl_namreply_regex.captures(caps).unwrap();
                let channel_name = captures.get(4).unwrap().as_str();
                let user_list = captures.get(5).unwrap().as_str();

                if !channel_name.starts_with('#') {
                    return;
                }

                let channel_exists = state.get_channel_by_name(channel_name).is_some();
                if !channel_exists {
                    state.channels.push(IrcChannel {
                        selected: false,
                        name: channel_name.to_string(),
                        users: vec![],
                        messages: vec![],
                        channel_type: IrcChannelType::Channel,
                    });
                }

                let channel = state
                    .channels
                    .iter_mut()
                    .find(|c| c.name == channel_name)
                    .expect("what the fuck why the fuck ???? what (error code 839569838945 or something)");

                channel.users = user_list
                    .split_whitespace()
                    .map(|user| user.to_string())
                    .collect();
            }

            caps if join_regex.captures(caps).is_some() => {
                let captures = join_regex.captures(caps).unwrap();
                let nick = captures.get(1).unwrap().as_str();
                let channel_name = captures.get(4).unwrap().as_str();

                if !channel_name.starts_with('#') {
                    return;
                }

                if let Some(ch) = state.get_channel_by_name(channel_name) {
                    ch.users.push(nick.to_string());
                    ch.messages.push(IrcMessage {
                        nick: None,
                        content: format!("-> {} joined", nick),
                    });
                }
            }

            caps if part_regex.captures(caps).is_some() => {
                let captures = part_regex.captures(caps).unwrap();
                let nick = captures.get(1).unwrap().as_str();
                let channel_name = captures.get(4).unwrap().as_str();

                if !channel_name.starts_with('#') {
                    return;
                }

                if let Some(ch) = state.get_channel_by_name(channel_name) {
                    ch.users.retain(|user| user != nick);
                    ch.messages.push(IrcMessage {
                        nick: None,
                        content: format!("<- {} left", nick),
                    });
                }
            }

            msg if msg.starts_with(":") => {
                if let Some((_, trailing)) = msg.split_once(" :") {
                    state.get_system_channel().messages.push(IrcMessage {
                        nick: None,
                        content: trailing.to_string(),
                    });
                } else {
                    state.get_system_channel().messages.push(IrcMessage {
                        nick: None,
                        content: msg.to_string(),
                    });
                }
            }

            msg => {
                println!("{}", msg);
            }
        }
    }

    /// checks for incoming messages and handles them
    /// run this in the main loop
    pub fn handler(&mut self, state: &mut State) -> Result<(), Error> {
        let mut buffer = [0; 512]; // irc is max 512 bytes per message
        match self.stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                for line in message.split("\r\n") {
                    if !line.is_empty() {
                        self.irc_handler(line, state);
                    }
                }
            }
            Ok(_) => {}                                                // no data read
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {} // no data available right now
            Err(e) if e.kind() == std::io::ErrorKind::NetworkDown => {
                panic!(
                    "Network connection lost. Unfortunately there is no way to resume the network connection. If you know a way to do this, please open an issue on GitHub."
                );
            }
            Err(e) => return Err(e), // actual error
        }
        Ok(())
    }
}
