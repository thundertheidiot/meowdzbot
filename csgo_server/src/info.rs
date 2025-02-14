use core::str;
use std::{io, str::Utf8Error};
use serde::Serialize;
use tokio::net::UdpSocket;
use crate::parse_to_string;

use crate::request::{send_request, Query};

#[derive(Debug, Clone, Serialize)]
pub enum ServerType {
    Dedicated,
    NonDedicated,
    Proxy,
    Invalid,
}

impl From<u8> for ServerType {
    fn from(i: u8) -> Self {
        match i {
            b'd' => Self::Dedicated,
            b'l' => Self::NonDedicated,
            b'p' => Self::Proxy,
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ServerEnvironment {
    Linux,
    Windows,
    Mac,
    Invalid,
}

impl From<u8> for ServerEnvironment {
    fn from(i: u8) -> Self {
        match i {
            b'l' => Self::Linux,
            b'w' => Self::Windows,
            b'm' | b'o' => Self::Mac,
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ServerVisibility {
    Public,
    Private,
    Invalid,
}

impl From<u8> for ServerVisibility {
    fn from(i: u8) -> Self {
        match i {
            0 => Self::Public,
            1 => Self::Private,
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum VAC {
    Unsecured,
    Secured,
    Invalid,
}

impl From<u8> for VAC {
    fn from(i: u8) -> Self {
        match i {
            0 => Self::Unsecured,
            1 => Self::Secured,
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub protocol: u8,
    pub name: String,
    pub map: String,
    pub folder: String,
    pub game: String,
    pub id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: ServerType,
    pub server_environment: ServerEnvironment,
    pub public: bool,
    pub vac: bool,
    pub version: String,
}

impl TryFrom<Vec<u8>> for ServerInfo {
    type Error = Utf8Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Utf8Error> {
        let mut index = 5; // skip over header and header byte
        let protocol = data[index];
	index += 1;
        let name: String;
        (name, index) = parse_to_string(&data, index)?;
        let map: String;
        (map, index) = parse_to_string(&data, index)?;
        let folder: String;
        (folder, index) = parse_to_string(&data, index)?;
        let game: String;
        (game, index) = parse_to_string(&data, index)?;

        let id: i16 = i16::from_le_bytes([data[index], data[index + 1]]);
        index += 2;

        let players: u8 = data[index];
        index += 1;

        let max_players: u8 = data[index];
        index += 1;

        let bots: u8 = data[index];
        index += 1;

        let server_type: ServerType = data[index].into();
        index += 1;

        let server_environment: ServerEnvironment = data[index].into();
        index += 1;

	let public = if data[index] == 0 {true} else {false};
	index += 1;

	let vac = if data[index] == 0 {false} else {true};
	index += 1;

	let version: String;
        (version, index) = parse_to_string(&data, index)?;

	// TODO: data flag and optional properties

        Ok(ServerInfo {
            protocol,
            name,
            map,
            folder,
            game,
            id,
            players,
            max_players,
            bots,
            server_type,
            server_environment,
	    public,
            vac,
	    version,
        })
    }
}

pub async fn get_server_info(sock: &UdpSocket) -> io::Result<ServerInfo> {
    let data = send_request(sock, Query::Info).await?;

    Ok(ServerInfo::try_from(data)
       .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_from_data() {
// 	let data = [u8; 100] = [
//             0xFF, 0xFF, 0xFF, 0xFF, 0x49, 0x02, 0x67, 0x61, 0x6D, 0x65, 0x32, 0x78, 0x73, 0x2E, 0x63,
//             0x6F, 0x6D, 0x20, 0x43, 0x6F, 0x75, 0x6E, 0x74, 0x65, 0x72, 0x2D, 0x53, 0x74, 0x72, 0x69,
//             0x6B, 0x65, 0x20, 0x53, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x20, 0x23, 0x31, 0x00, 0x64, 0x65,
//             0x5F, 0x64, 0x75, 0x73, 0x74, 0x00, 0x63, 0x73, 0x74, 0x72, 0x69, 0x6B, 0x65, 0x00, 0x43,
//             0x6F, 0x75, 0x6E, 0x74, 0x65, 0x72, 0x2D, 0x53, 0x74, 0x72, 0x69, 0x6B, 0x65, 0x3A, 0x20,
//             0x53, 0x6F, 0x75, 0x72, 0x63, 0x65, 0x00, 0xF0, 0x00, 0x05, 0x10, 0x04, 0x64, 0x6C, 0x00,
//             0x00, 0x31, 0x2E, 0x30, 0x2E, 0x30, 0x2E, 0x32, 0x32, 0x00,
// 	].to_vec();

// 	let result = ServerInfo {
	    
// 	}
//     }
// }
