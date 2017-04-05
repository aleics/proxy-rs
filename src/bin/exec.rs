extern crate multiproxy;

use multiproxy::proxy::Proxy;

fn main() {
  let proxy = Proxy::new("config.toml", 10);
  println!("{:?}", proxy.config);

  proxy.start();
}