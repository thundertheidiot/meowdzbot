use crate::byte;
use crate::long_long;
use crate::short;
use crate::string;
use serde::Serialize;
use std::io::Bytes;
use std::io::{self, Read};
use tokio::net::UdpSocket;

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
pub struct SourceTV {
    pub port: i16,
    pub name: Box<str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub protocol: u8,
    pub name: Box<str>,
    pub map: Box<str>,
    pub folder: Box<str>,
    pub game: Box<str>,
    pub id: i16,
    pub players: u8,
    pub max_players: u8,
    pub bots: u8,
    pub server_type: ServerType,
    pub server_environment: ServerEnvironment,
    pub public: bool,
    pub vac: bool,
    pub version: Box<str>,
    pub edf: u8,
    pub port: Option<i16>,
    pub steam_id: Option<u64>,
    pub source_tv: Option<SourceTV>,
    pub keywords: Option<Box<str>>,
    pub game_id: Option<u64>,
}

use crate::Error;

impl TryFrom<Bytes<&[u8]>> for ServerInfo {
    type Error = Error;

    fn try_from(mut data: Bytes<&[u8]>) -> Result<Self, Error> {
        data.advance_by(5).or(Err("Unable to advance by 5"))?;

        let protocol: u8 = byte(&mut data)?;

        let name = string(&mut data)?;
        let map = string(&mut data)?;
        let folder = string(&mut data)?;
        let game = string(&mut data)?;

        let id: i16 = short(&mut data)?;

        let players: u8 = byte(&mut data)?;
        let max_players: u8 = byte(&mut data)?;
        let bots: u8 = byte(&mut data)?;

        let server_type: ServerType = byte(&mut data)?.into();

        let server_environment: ServerEnvironment = byte(&mut data)?.into();

        let public: bool = byte(&mut data)? == 0;
        let vac: bool = byte(&mut data)? == 1;

        let version = string(&mut data)?;

        let edf: u8 = byte(&mut data)?;

        let port: Option<i16> = if (edf & 0x80) != 0 {
            Some(short(&mut data)?)
        } else {
            None
        };

        let steam_id: Option<u64> = if (edf & 0x10) != 0 {
            Some(long_long(&mut data)?)
        } else {
            None
        };

        let source_tv: Option<SourceTV> = if (edf & 0x40) != 0 {
            let port = short(&mut data)?;
            let name = string(&mut data)?;

            Some(SourceTV { port, name })
        } else {
            None
        };

        let keywords: Option<Box<str>> = if (edf & 0x20) != 0 {
            Some(string(&mut data)?)
        } else {
            None
        };

        let game_id: Option<u64> = if (edf & 0x10) != 0 {
            Some(long_long(&mut data)?)
        } else {
            None
        };

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
            edf,
            port,
            steam_id,
            source_tv,
            keywords,
            game_id,
        })
    }
}

pub async fn get_server_info(sock: &UdpSocket) -> io::Result<ServerInfo> {
    let data = send_request(sock, Query::Info).await?;
    let bytes = data.bytes();

    ServerInfo::try_from(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
