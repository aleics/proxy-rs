use config::Config;

use std::net::SocketAddr;

use url::Url;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use futures::{future, Future, Stream};
use futures_cpupool::CpuPool;

use hyper;
use hyper::{Client, StatusCode, Body};
use hyper::client::HttpConnector;
use hyper::server::{Service, Http, Request, Response};

#[derive(Clone)]
pub struct Proxy {
  pub config: Config,
  thread_pool: CpuPool
}

impl Proxy {
  pub fn new(config_path: &str, threads: usize) -> Proxy {
    let config = match Config::read(config_path) {
      Err(err) => panic!("Error: {}", err),
      Ok(c) => c
    };
    Proxy { config: config, thread_pool:  CpuPool::new(threads) }
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

    let client = Client::new(&handle);
    let connections = listener.incoming();
    let protocol = Http::new();
    let server = connections.for_each(|(socket, peer_addr)| {
      let conn = ConnHandler { routes: self.config.routes.clone(), client: client.clone() };
      protocol.bind_connection(&handle, socket, peer_addr, conn);
      Ok(())
    });

    core.run(server).unwrap()
  }
}

struct ConnHandler {
  routes: Vec<Url>,
  client: Client<HttpConnector, Body>
}

impl ConnHandler {
  pub fn contains_route(&self, route: &str) -> bool {
    let url = match Url::parse(route) {
      Err(_) => return false,
      Ok(u) => u
    };
    self.routes.contains(&url)
  }
}

impl Service for ConnHandler {
    type Request =  Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = future::BoxFuture<Self::Response, Self::Error>;

  fn call(&self, req: Request) -> Self::Future {
    future::ok(Response::new().with_status(StatusCode::NotImplemented)).boxed()
  }
}