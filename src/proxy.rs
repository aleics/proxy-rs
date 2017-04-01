use config::Config;

#[derive(Debug)]
pub struct Proxy {
  config: Config
}

impl Proxy {
  pub fn new(config_path: &str) -> Proxy {
    let config = match Config::read(config_path) {
      Err(err) => panic!("Error: {}", err),
      Ok(c) => c
    };
    Proxy { config: config }
  }
}