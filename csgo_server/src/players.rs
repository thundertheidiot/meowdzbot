use serde::Serialize;
use std::{io, str::Utf8Error};
use tokio::net::UdpSocket;

use crate::{
    parse_to_string,
    request::{send_request, Query},
};

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    pub index: u8,
    pub name: Box<str>,
    pub score: i32,
    pub duration: f32,
}

impl Player {
    fn make(data: &[u8], mut index: usize) -> Result<(Player, usize), Utf8Error> {
        let i = data[index];
        index += 1;

        let name: Box<str>;
        (name, index) = parse_to_string(data, index)?;

        let score = i32::from_le_bytes([
            data[index],
            data[index + 1],
            data[index + 2],
            data[index + 3],
        ]);
        index += 4;

        let duration = f32::from_le_bytes([
            data[index],
            data[index + 1],
            data[index + 2],
            data[index + 3],
        ]);
        index += 4;

        Ok((
            Player {
                index: i,
                name,
                score,
                duration,
            },
            index,
        ))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Players(pub Vec<Player>);

impl TryFrom<Vec<u8>> for Players {
    type Error = Utf8Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Utf8Error> {
        let mut index = 5; // skip over header and header byte

        let num_players = data[index];
        index += 1;

        let mut players: Vec<Player> = Vec::new();

        for _ in 0..num_players {
            let p: Player;
            (p, index) = Player::make(&data, index)?;
            players.push(p);
        }

        Ok(Players(players))
    }
}

impl Players {
    pub fn real(self) -> Self {
        Players(
            self.0
                .into_iter()
                .filter(|p| {
		    !(*p.name).is_empty() &&
			&*p.name != "DatHost - GOTV"
                })
                .collect::<Vec<Player>>(),
        )
    }
}

pub async fn get_players(sock: &UdpSocket) -> io::Result<Players> {
    let data = send_request(sock, Query::Player).await?;

    Players::try_from(data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn yea() {
//         let data = [
//             255, 255, 255, 255, 68, 8, 0, 68, 97, 116, 72, 111, 115, 116, 32, 45, 32, 71, 79, 84,
//             86, 0, 0, 0, 0, 0, 204, 97, 175, 70, 0, 0, 0, 0, 0, 0, 21, 145, 89, 70, 0, 112, 111,
//             107, 105, 115, 52, 52, 52, 118, 50, 0, 29, 0, 0, 0, 199, 245, 219, 69, 0, 78, 111, 109,
//             97, 100, 0, 36, 0, 0, 0, 38, 198, 218, 69, 0, 71, 108, 97, 109, 117, 82, 0, 37, 0, 0,
//             0, 245, 204, 101, 69, 0, 36, 116, 105, 108, 108, 80, 97, 108, 109, 84, 114, 101, 101,
//             115, 226, 132, 162, 0, 0, 0, 0, 0, 143, 38, 193, 67, 0, 67, 114, 97, 115, 104, 101,
//             114, 0, 0, 0, 0, 0, 112, 127, 17, 67, 0, 67, 117, 112, 111, 102, 106, 117, 105, 99,
//             101, 0, 0, 0, 0, 0, 114, 71, 17, 67,
//         ]
//         .to_vec();

//         println!("{:#?}", Players::try_from(data));

//         assert_eq!(1, 1);
//     }
// }
