extern crate toml;
//extern crate serde_derive;

use std::net::SocketAddr;

pub fn sock2str(addr: &SocketAddr) -> String {
    match *addr {
        SocketAddr::V4(ref _addr) => {
            format!("{}", _addr.ip())
        }
        SocketAddr::V6(ref _addr) => {
            format!("{}", _addr.ip())
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigInternal {
    core: Option<CoreConfigInternal>,
    torrents: Option<TorrentsConfigInternal>
}

#[derive(Debug, Deserialize)]
struct CoreConfigInternal {
    bindaddress: Option<String>,
    port: Option<u16>
}

#[derive(Debug, Deserialize)]
struct TorrentsConfigInternal {
    announcetime: Option<u32>
}

pub struct RTConfig {
    pub core: CoreConfig,
    pub torrents: TorrentsConfig
}

pub struct CoreConfig {
    pub bindaddress: String,
    pub port: u16
}

pub struct TorrentsConfig {
    pub announcetime: u32
}

pub fn parse_config(conf: &str) -> RTConfig {
    let decoded: ConfigInternal = toml::from_str(conf).unwrap();
    RTConfig {
        core: match decoded.core {
            Some(_core) => {
                CoreConfig {
                    bindaddress: match _core.bindaddress {
                        Some(bindaddr) => { bindaddr },
                        None => { "0.0.0.0".to_string() }
                    },
                    port: match _core.port {
                        Some(_port) => { _port },
                        None => { 80 }
                    }
                }
            },
            None => {
                CoreConfig {
                    bindaddress: "0.0.0.0".to_string(),
                    port: 80
                }
            }
        },
        torrents: match decoded.torrents {
            Some(_torrents) => {
                TorrentsConfig {
                    announcetime: match _torrents.announcetime {
                        Some(time) => { time },
                        None => 1800
                    }
                }
            },
            None => {
                TorrentsConfig {
                    announcetime: 1800
                }
            }
        }
    }
}
