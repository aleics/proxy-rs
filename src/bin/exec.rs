extern crate multiproxy;

use multiproxy::proxy::Proxy;

fn main() {
  let proxy = Proxy::new("config.toml");
  println!("{:?}", proxy);

  proxy.start();
}