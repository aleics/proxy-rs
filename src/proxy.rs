use config::Config;

use std::net::SocketAddr;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use futures::{Future, Stream};
use futures_cpupool::CpuPool;

use hyper;
use hyper::Uri;
use hyper::{Client, Body, client, server};
use hyper::server::{Service, Http};

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
      let conn = ConnHandler { route: self.config.routes[0].clone(), client: client.clone() };
      // Here the connection needs to be  asynchronous and iterate through all the routes
      protocol.bind_connection(&handle, socket, peer_addr, conn);
      Ok(())
    });

    core.run(server).unwrap()
  }
}

struct ConnHandler {
  route: Uri,
  client: Client<client::HttpConnector, Body>
}

impl ConnHandler {
  pub fn build_request(&self, req: server::Request) -> client::Request {
    let (method, _, version, headers, body) = req.deconstruct();
    let mut request = hyper::client::Request::new(method, self.route.clone());
    request.set_body(body);
    request.set_version(version);
    *request.headers_mut() = headers;
    request
  }
}

impl Service for ConnHandler {
    type Request =  server::Request;
    type Response = server::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

  fn call(&self, req: server::Request) -> Self::Future {
    let request = self.build_request(req);
    let response = self.client.request(request).map(|res| {
      let mut resp = server::Response::new();
      resp = resp.with_headers(res.headers().clone());
      resp.set_status(res.status().clone());
      resp.set_body(res.body());
      resp
    });
    Box::new(response)
  }
}
