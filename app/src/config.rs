// This module is responsible for parsing configuration options
extern crate ini;
use self::ini::Ini;
use self::ini::ini::Properties;

use errors::*;

use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Clone)]
pub struct Config {
    pub hostkey_path: String,
    pub api_socket: SocketAddr,
    pub p2p_socket: SocketAddr,
    pub min_hop_count: u8
}

#[allow(or_fun_call)]
fn read_property(section: &Properties, property: &'static str) -> Result<String> {
    Ok(section.get(property)
        .ok_or(Error::from(format!("[{}] property not found in config file", property).to_string()))?
        .to_string())
}

/** Parses the config file and creates an object to be used across the app **/
#[allow(or_fun_call)]
pub fn read_config_file(config_file_path: String) -> Result<Config> {
    let config_file = Ini::load_from_file(config_file_path)
        .chain_err(|| "Config file not found")?;

    let onion_section = config_file.section(Some("onion".to_owned()))
        .ok_or(Error::from("[onion] section not found in config file"))?;

    let config = Config {
        hostkey_path: read_property(onion_section, "hostkey")?,
        api_socket: SocketAddr::from_str(&read_property(onion_section, "api_addr")?)
            .chain_err(|| "[api_addr] property failed to parse")?,
        p2p_socket: SocketAddr::from_str(&format!("0.0.0.0:{}",
            read_property(onion_section, "p2p_port")?))
                .chain_err(|| "[p2p_port] property failed to parse")?,
        min_hop_count: read_property(onion_section, "min_hop_count")?.parse()
            .chain_err(|| "[min_hop_count] property failed to parse")?
    };

    Ok(config)
}
