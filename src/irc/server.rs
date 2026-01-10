use std::{
    io::{Error, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
};

use ctru::prelude::Soc;

pub struct IrcServer {
    addr: SocketAddr,
    stream: TcpStream,
    soc_service: Soc,
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
        println!("Connected to IRC server!");

        IrcServer {
            addr,
            stream,
            soc_service: soc,
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

    /// checks for incoming messages and handles them
    /// run this in the main loop
    pub fn handler(&mut self) -> Result<(), Error> {
        let mut buffer = [0; 512]; // irc is max 512 bytes per message
        match self.stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let message = String::from_utf8_lossy(&buffer[..size]);
                println!("{}", message);
                // handle the message (parsing, responding, etc.)
            }
            Ok(_) => {
                // no data received
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // this error is fine just retry
            }
            Err(e) => {
                println!("Error reading from IRC server: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }
}
