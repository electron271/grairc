use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

use ctru::prelude::Soc;

use crate::irc::constants::{IRC_CHANNEL, IRC_HOST, IRC_NICK};

pub struct IrcServer {
    addr: SocketAddr,
    stream: TcpStream,
    _soc_service: Soc, // this always needs to be kept alive
}

impl IrcServer {
    pub fn new(hostname: &str) -> Self {
        let soc = Soc::new().expect("Failed to initialize SOC service");
        println!("SOC service initialized");

        let addr = hostname
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
            addr,
            stream,
            _soc_service: soc,
        }
    }

    pub fn irc_ident(&mut self, nick: &str, channel: &str) -> Result<(), Error> {
        let user_cmd = format!("USER {} 0 * :{}\r\n", nick, nick);
        let nick_cmd = format!("NICK {}\r\n", nick);
        let join_cmd = format!("JOIN {}\r\n", channel);

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

    pub fn irc_handler(&mut self, message: &str) {
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

                    if let Some(nick) = nick {
                        println!("{:<10} ({}): {}", nick, channel, message_content);
                    }
                }
            }

            msg if msg.starts_with(":") => {
                if let Some((_, trailing)) = msg.split_once(" :") {
                    println!("{}", trailing.trim());
                } else {
                    println!("{}", msg);
                }
            }

            msg => {
                println!("{}", msg);
            }
        }
    }

    /// checks for incoming messages and handles them
    /// run this in the main loop
    pub fn handler(&mut self) -> Result<(), Error> {
        let mut buffer = [0; 512]; // irc is max 512 bytes per message
        match self.stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                for line in message.split("\r\n") {
                    let line = line.trim();
                    if !line.is_empty() {
                        self.irc_handler(line);
                    }
                }
            }
            Ok(_) => {}                                                // no data read
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {} // no data available right now
            Err(e) if e.kind() == std::io::ErrorKind::NetworkDown => {
                panic!("Network connection lost. Unfortunately grairc cannot recover from this.");
            }
            Err(e) => return Err(e), // actual error
        }
        Ok(())
    }
}
