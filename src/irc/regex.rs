#![allow(unused)]

/// 1: nickname, 2: username, 3: host, 4: channel, 5: message content
pub const PRIVMSG_REGEX: &str = r":(.*)!(.*)@(.*) PRIVMSG (.*) :(.*)";

/// 1: server, 2: client, 3: symbol, 4: channel, 5: user list
///
/// for more information see https://modern.ircdocs.horse/#rplnamreply-353
pub const RPL_NAMREPLY_REGEX: &str = r":(.*) 353 (.*) (.{1}) (.*) :(.*)";

/// 1: nickname, 2: username, 3: host, 4: channel, 5: optional leaving message
pub const PART_REGEX: &str = r":(.*)!(.*)@(.*) PART (.*) :(.*)";

/// idk why its having the : i cant find it on the spec
/// 1: nickname, 2: username, 3: host, 4: channel
pub const JOIN_REGEX: &str = r":(.*)!(.*)@(.*) JOIN :?(.*)";
