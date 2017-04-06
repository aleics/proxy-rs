use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read};
use std::fs::File;

use hyper::Uri;
use toml;

#[derive(Deserialize, Debug, Clone)]
pub struct ProxyConfig {
  pub address: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct TomlConfig {
  pub proxy: ProxyConfig,
  pub routes: HashMap<String, String>
}

#[derive(Debug, Clone)]
pub struct Config {
  pub proxy: ProxyConfig,
  pub routes: Vec<Uri>
}

impl Config {
  pub fn read(path: &str) -> Result<Config, Error> {
    File::open(path)
      .and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map(|_| contents)
      }).and_then(|contents| {
        toml::from_str(contents.as_str())
          .map_err(|err| Error::new(ErrorKind::InvalidData, err))
      }).and_then(|config| Config::parse_uris(config))
  }

  fn parse_uris(toml_config: TomlConfig) -> Result<Config, Error> {
    let mut parsed_routes: Vec<Uri> = Vec::new();
    for (_, url) in toml_config.routes.iter() {
      match url.parse::<Uri>() {
        Err(err) => return Err(Error::new(ErrorKind::InvalidData, err)),
        Ok(u) => parsed_routes.push(u)
      }
    }
    Ok(Config {
      proxy: toml_config.proxy,
      routes: parsed_routes
    })
  }
}