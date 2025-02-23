use csgo_server::{info::ServerInfo, players::Players};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::RwLock;

use csgo_server::info;
use csgo_server::players;

use crate::socket::ServerSocketValue;
use crate::Error;

#[derive(Debug, Clone, Serialize)]
pub struct Info {
    pub server_info: ServerInfo,
    pub players: Players,
    timestamp: SystemTime,
    pub elapsed: Duration,
    pub image: Option<Box<str>>,
}

static INFO: Lazy<Arc<RwLock<HashMap<String, Info>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub struct MapData {
    map: Box<str>,
    time: SystemTime,
    image: Option<Box<str>>,
}

impl MapData {
    fn new(map: &str) -> Self {
	let time = SystemTime::now();

	Self {
	    map: Box::from(map),
	    time,
	    image: Self::get_image(map, &time),
	}
    }

    fn update(&mut self, map: &str) {
	let time = SystemTime::now();

        self.map = Box::from(map);
	self.image = Self::get_image(map, &time);
        self.time = time;
    }

    fn get_image(map: &str, time: &SystemTime) -> Option<Box<str>> {
	const SIROCCO: &[&str] = &[
	    "sirocco1.jpg",
	    "sirocco2.jpg",
	    "sirocco3.jpg",
	    "sirocco4.jpg",
	];

	const BLACKSITE: &[&str] = &[
	    "blacksite1.jpg",
	    "blacksite2.jpg",
	    "blacksite3.jpg",
	];

	const EMBER: &[&str] = &[
	    "ember1.jpg",
	    "ember2.jpg",
	    "ember3.jpg",
	    "ember4.jpg",
	    "ember5.jpg",
	];

	const VINEYARD: &[&str] = &[
	    "vineyard1.jpg",
	    "vineyard2.jpg",
	    "vineyard3.jpg",
	];

	const COUNTY: &[&str] = &[
	    "county1.jpg",
	    "county2.jpg",
	    "county3.jpg",
	];

	let arr = match map {
	    "dz_sirocco" => SIROCCO,
	    "dz_blacksite" => BLACKSITE,
	    "dz_ember" => EMBER,
	    "dz_vineyard" => VINEYARD,
	    "dz_county" => COUNTY,
	    _ => {return None},
	};

	let time = time.duration_since(UNIX_EPOCH).unwrap().as_nanos();
	let index = (time as usize) % arr.len();

	Some(Box::from(arr[index]))
    }
}

static MAP_DATA: Lazy<Arc<RwLock<HashMap<String, MapData>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

async fn map_data_setup(name: &String, server_info: &ServerInfo) {
    let mut data = MAP_DATA.write().await;

    match data.get_mut(name) {
        Some(v) => {
            if v.map != server_info.map {
                v.update(&server_info.map);
            }
        }
        None => {
            data.insert(
                name.clone(),
		MapData::new(&server_info.map),
            );
        }
    }
}

pub async fn get_server_info(socks: &ServerSocketValue, name: &String) -> Result<Info, Error> {
    let socks = socks
        .get(name)
        .ok_or(format!("SocketError: Unable to get socket of {}", name))?;

    let mut infomap = INFO.write().await;

    match infomap.get(name) {
        Some(v) => {
            if let Ok(duration) = SystemTime::now().duration_since(v.timestamp) {
                if duration.as_secs() >= 5 {
                    let server_info = info::get_server_info(&socks.0).await?;
                    let players = players::get_players(&socks.1).await?;

		    map_data_setup(name, &server_info).await;
                    // if this doesn't unwrap i will explode
		    let mapdata = MAP_DATA.read().await;
		    let mapdata = mapdata.get(name).unwrap();

		    let now = SystemTime::now();

                    let info = Info {
                        server_info,
                        players,
                        timestamp: now,
			elapsed: now.duration_since(mapdata.time)?,
			image: mapdata.image.clone(),
                    };

                    infomap.insert(name.clone(), info.clone());

                    return Ok(info);
                }
            }

            Ok(v.clone())
        }
        None => {
            println!("First data fetch for {}", name);

            let server_info = info::get_server_info(&socks.0).await?;
            let players = players::get_players(&socks.1).await?;

	    map_data_setup(name, &server_info).await;
            // if this doesn't unwrap i will explode
	    let mapdata = MAP_DATA.read().await;
            let mapdata = mapdata.get(name).unwrap();

	    let now = SystemTime::now();

            let info = Info {
                server_info,
                players,
                timestamp: now,
		elapsed: now.duration_since(mapdata.time)?,
		image: mapdata.image.clone(),
            };

            infomap.insert(name.clone(), info.clone());

            Ok(info)
        }
    }
}
