extern crate multiproxy;
extern crate clap;

use multiproxy::proxy::Proxy;
use clap::{Arg, App};

fn main() {
  let matches = App::new("Multiproxy")
                    .version("0.1.0")
                    .arg(Arg::with_name("config")
                      .short("c")
                      .long("config")
                      .value_name("FILE")
                      .help("Path of the configuration file")
                      .takes_value(true))
                    .get_matches();

  let config = matches.value_of("config").unwrap();

  let proxy = Proxy::new(config, 10);
  println!("{:?}", proxy.config);

  proxy.start();
}