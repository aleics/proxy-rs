use config::Config;

use std::net::SocketAddr;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_service::Service;
use tokio_io;

use futures::future::FutureResult;
use futures_cpupool::CpuPool;

use hyper;
use hyper::{Get, Post};
use hyper::status::StatusCode;
use hyper::header::ContentLength;
use hyper::server::{Request, Response};


#[derive(Debug, Copy, Clone)]
pub struct Proxy {
  config: Config,
  thread_pool: CpuPool
}

impl Proxy {
  pub fn new(config_path: &str) -> Proxy {
    let config = match Config::read(config_path) {
      Err(err) => panic!("Error: {}", err),
      Ok(c) => c
    };
    Proxy { config: config }
  }

  pub fn start(&self) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let address = match self.config.proxy.address.as_str().parse::<SocketAddr>() {
      Err(_) => panic!("Not valid listening address '{}': ", self.config.proxy.address),
      Ok(a) => a
    };

    let listener: TcpListener = match TcpListener::bind(&address, &handle) {
      Err(_) => panic!("Couldn't bind listener: {}", self.config.proxy.address),
      Ok(l) => l
    };

    let connections = listener.incoming();
    let server = connections.for_each(|(_socket, _peer_addr)| {
      // writes on the socket the message returned as a future
      let serve_one = tokio_io::io::write_all(_socket, b"Hello, world!\n")
        .then(|_| Ok(()));
      // handle the connections asynchronously
      handle.spawn(serve_one);
      Ok(())
    });

    core.run(server).unwrap()
  }
}

impl Service for Proxy {
  type Request = Request;
  type Response = Response;
  type Error = hyper::Error;
  type Future = FutureResult<Response, hyper::Error>;

  fn call(&self, req: Request) -> Self::Future {
  }
}