use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

use ctru::prelude::Soc;

use crate::{config::Config, irc::types::IrcMessage, state::State};

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

    pub fn irc_send(&mut self, message: &str, channel: &str) -> Result<(), Error> {
        let msg_cmd = format!("PRIVMSG {} :{}\r\n", channel, message);
        self.stream.write_all(msg_cmd.as_bytes())?;
        Ok(())
    }

    pub fn irc_handler(&mut self, message: &str, state: &mut State) {
        match message {
            msg if msg.starts_with("PING") => {
                let response = msg.replace("PING", "PONG");
                self.stream
                    .write_all(response.as_bytes())
                    .expect("Failed to send PONG response");
            }

            msg if msg.contains("PRIVMSG") => {
                let parts: Vec<&str> = msg.splitn(4, ' ').collect();
                if parts.len() >= 4 {
                    let nick_user_host = parts[0].trim_start_matches(':');
                    let channel = parts[2];
                    let message_content = parts[3].trim_start_matches(':').trim();

                    let nick = nick_user_host.split('!').next();

                    if let Some(ch) = state.channels.iter_mut().find(|ch| ch.name == channel) {
                        ch.messages.push(IrcMessage {
                            nick: Some(nick.unwrap_or("").to_string()),
                            content: message_content.to_string(),
                        });
                    } else {
                        let mut new_channel = crate::irc::types::IrcChannel {
                            selected: false,
                            name: channel.to_string(),
                            users: vec![],
                            messages: vec![],
                            channel_type: crate::irc::types::IrcChannelType::Channel,
                        };
                        new_channel.messages.push(IrcMessage {
                            nick: Some(nick.unwrap_or("").to_string()),
                            content: message_content.to_string(),
                        });
                        state.channels.push(new_channel);
                    }
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
                self.irc_handler(&message, state);
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
