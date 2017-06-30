#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate tokio_pool;
extern crate num_cpus;
extern crate tokio_io;

use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;

use futures::{Stream, Future};
use hyper::Body;

use hyper::Client;
use tokio_core::reactor::Core;

use std::net::{SocketAddr, TcpListener};
use tokio_core::net::TcpListener as TokTcpListener;
use std::thread;

use hyper::Uri;
use hyper::error::UriError;
use hyper::client::HttpConnector;


fn main() {
    pretty_env_logger::init().unwrap();
    server_start_up();
}


fn run_thread(listener: TcpListener) {

    let mut core = Core::new().expect("Create Server Event Loop");
    let addr = listener.local_addr().unwrap();
    let handle = core.handle();
    let listener = TokTcpListener::from_listener(listener, &addr, &handle).unwrap();
    let incoming = listener.incoming();

    let client = Client::new(&handle);

    let all_conns = incoming.for_each(|(socket, addr)| {
                        let service = Proxy { client: client.clone() };
                        Http::new().bind_connection(&handle.clone(), socket, addr, service);
                        Ok(())
                    }).map_err(|err| {
                        error!("Error with Tcp Listener: {:?}", err);
                    });

    core.run(all_conns).unwrap();

}

fn server_start_up() {

    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    info!("Starting Server on {:?}...", addr);

    let listener = TcpListener::bind(addr).unwrap();

    let mut threads = Vec::new();
    for _ in 0..num_cpus::get() {
        let listener = listener.try_clone().unwrap();
        threads.push(thread::spawn(move || run_thread(listener)));
    }

    for t in threads {
        t.join().unwrap();
    }
}

struct Proxy {
    client: Client<HttpConnector, Body>
}

impl Proxy {

    fn create_proxy_url(&self, host: &str, uri: Uri) -> Result<Uri, UriError> {
        format!("{}{}{}", host, uri.path(), uri.query().unwrap_or("")).parse()
    }
}

impl Service for Proxy {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {

        let host = "http://localhost:9000"; // other host
        let uri = self.create_proxy_url(host, req.uri().clone())
            .expect(&format!("Failed trying to parse uri. Origin: {:?}", &req.uri()));

        let mut client_req = Request::new(req.method().clone(), uri);
        client_req.headers_mut().extend(req.headers().iter());
        client_req.set_body(req.body());

        info!("Dispatching request: {:?}", client_req);

        let resp = self.client.request(client_req).then(move |result| {
            let response = match result {
                Ok(client_resp) => client_resp,
                Err(e) => {
                    error!("{:?}", &e);
                    Response::new().with_status(StatusCode::ServiceUnavailable)
                }
            };
            futures::future::ok(response)
        });

        Box::new(resp) as Self::Future
    }
}
