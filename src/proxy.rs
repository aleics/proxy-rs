use config::Config;

use std::collections::HashMap;
use std::net::SocketAddr;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use futures::{future, Future, Stream};

use hyper;
use hyper::Uri;
use hyper::{Client, Body, client, server, StatusCode};
use hyper::server::{Service, Http};

#[derive(Clone)]
/// Proxy initialize the proxy server given a configuration and a multithread pool instance
pub struct Proxy {
  pub config: Config
}

/// Implementation for Proxy
impl Proxy {

  /// Create a new Proxy instance
  ///
  /// # Arguments
  ///
  /// * `config_path`: path of the configuration file
  /// * `threads`: number of threads for the pool instance
  pub fn new(config_path: &str) -> Proxy {
    let config = match Config::read(config_path) {
      Err(err) => panic!("Error: {}", err),
      Ok(c) => c
    };
    Proxy { config: config }
  }

  /// Start the proxy server
  pub fn start(&self) {
    // read the address from the configuration file and listen to it
    let address = match self.config.proxy.address.as_str().parse::<SocketAddr>() {
      Err(_) => panic!("Not valid listening address '{}': ", self.config.proxy.address),
      Ok(a) => a
    };

    // initialize the core tokio instance
    let mut core = match Core::new() {
      Err(err) => panic!("Couldn't initialize the core instance:  {}", err),
      Ok(c) => c
    };
    let handle = core.handle();

    let listener: TcpListener = match TcpListener::bind(&address, &handle) {
      Err(_) => panic!("Couldn't bind listener: {}", self.config.proxy.address),
      Ok(l) => l
    };

    // create a new client and reads incoming connections
    let client = Client::new(&handle);
    let connections = listener.incoming();
    let protocol = Http::new();;

    // manage connections concurrently as a stream
    let server = connections.for_each(|(socket, peer_addr)| {
      let conn = ProxyService { client: client.clone(), routes: self.config.routes.clone() };

      // bind the connection
      protocol.bind_connection(&handle, socket, peer_addr, conn);
      Ok(())
    });

    // run the server
    core.run(server).unwrap()
  }
}

/// ProxyService manages the single routing request
struct ProxyService {
  client: Client<client::HttpConnector, Body>,
  routes: HashMap<String, Uri>
}

/// ProxyService implementation
impl ProxyService {

  /// Build a request from a server request to a client request
  ///
  /// # Arguments
  ///
  /// * `req`: server request
  pub fn build_request(&self, req: server::Request, route: Uri) -> client::Request {
    let (method, _, version, headers, body) = req.deconstruct();
    let mut request = hyper::client::Request::new(method, route);
    request.set_body(body);
    request.set_version(version);
    *request.headers_mut() = headers;
    request
  }
}


/// Service implementation for ProxyService
impl Service for ProxyService {
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
    // get 
    let address = match self.routes.get(req.path()) {
      None => {
        println!("Path {} not defined in configuration. Returning 404...", req.path());

        let mut resp = server::Response::new();
        resp.set_status(StatusCode::NotFound);
        return Box::new(future::ok(resp));
      },
      Some(uri) => uri.clone()
    };

    let request = self.build_request(req, address);
    println!("{:?}", request);

    // make a request to the defined route
    Box::new(self.client.request(request).map(|res| {
      println!("{:?}", res);
      // create a server response from the client response
      let mut resp = server::Response::<Body>::new();
      resp = resp.with_headers(res.headers().clone());
      resp.set_status(res.status().clone());
      resp.set_body(res.body());
      resp
    }))
  }
}
