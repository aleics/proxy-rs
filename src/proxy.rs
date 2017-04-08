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
/// Proxy initialize the proxy server given a configuration and a multithread pool instance
pub struct Proxy {
  pub config: Config,
  pool: CpuPool
}

/// Implementation for Proxy
impl Proxy {

  /// Create a new Proxy instance
  ///
  /// # Arguments
  ///
  /// * `config_path`: path of the configuration file
  /// * `threads`: number of threads for the pool instance
  pub fn new(config_path: &str, threads: usize) -> Proxy {
    let config = match Config::read(config_path) {
      Err(err) => panic!("Error: {}", err),
      Ok(c) => c
    };
    Proxy { config: config, pool: CpuPool::new(threads) }
  }

  /// Start the proxy server
  pub fn start(&self) {
    // initialize the core tokio instance
    let mut core = match Core::new() {
      Err(err) => panic!("Couldn't initialize the core instance:  {}", err),
      Ok(c) => c
    };
    let handle = core.handle();

    // read the address from the configuration file and listen to it
    let address = match self.config.proxy.address.as_str().parse::<SocketAddr>() {
      Err(_) => panic!("Not valid listening address '{}': ", self.config.proxy.address),
      Ok(a) => a
    };
    let listener: TcpListener = match TcpListener::bind(&address, &handle) {
      Err(_) => panic!("Couldn't bind listener: {}", self.config.proxy.address),
      Ok(l) => l
    };

    // create a new client and manage all the connections
    let client = Client::new(&handle);
    let connections = listener.incoming();
    let server = connections.for_each(|(socket, peer_addr)| {

      // Here the connection needs to be  asynchronous and iterate through all the routes
      let routes = self.config.routes.clone();
      for route in routes {
        let conn = ConnHandler { route: route.clone(), client: client.clone() };
        println!("Handling '{}'...", route);
      }
      Ok(())
    });

    // run the server
    core.run(server).unwrap()
  }
}

/// ConnHandler manages the single routing request
struct ConnHandler {
  route: Uri,
  client: Client<client::HttpConnector, Body>
}

/// ConnHandler implementation
impl ConnHandler {

  /// Build a request from a server request to a client request
  ///
  /// # Arguments
  ///
  /// * `req`: server request
  pub fn build_request(&self, req: server::Request) -> client::Request {
    let (method, _, version, headers, body) = req.deconstruct();
    let mut request = hyper::client::Request::new(method, self.route.clone());
    request.set_body(body);
    request.set_version(version);
    *request.headers_mut() = headers;
    request
  }
}


/// Service implementation for ConnHandler
impl Service for ConnHandler {
    type Request =  server::Request;
    type Response = server::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

  /// Make a request to a defined route
  ///
  /// # Arguments
  ///
  /// * `req`: server request
  fn call(&self, req: server::Request) -> Self::Future {
    let request = self.build_request(req);

    // make a request to the defined route
    let response = self.client.request(request).map(|res| {
      // create a server response from the client response
      let mut resp = server::Response::new();
      resp = resp.with_headers(res.headers().clone());
      resp.set_status(res.status().clone());
      resp.set_body(res.body());
      resp
    });
    Box::new(response)
  }
}
