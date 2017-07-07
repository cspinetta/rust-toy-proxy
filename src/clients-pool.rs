#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate tokio_pool;
extern crate num_cpus;
extern crate tokio_io;

extern crate net2;

use hyper::server::{Http, Request, Response, Service};
use hyper::StatusCode;

use futures::{Stream, Future};
use hyper::Body;

use hyper::Client;
use tokio_core::reactor::Core;

use tokio_core::net::TcpListener;
use std::thread;
use std::sync::Arc;
use std::net::SocketAddr;

use hyper::Uri;
use hyper::error::UriError;
use hyper::client::HttpConnector;

use net2::unix::UnixTcpBuilderExt;


fn main() {
    pretty_env_logger::init().unwrap();
    let mut core = Core::new().expect("Create Client Event Loop");
    let handle = core.handle();

    let remote = core.remote();

    let client = Client::new(&handle.clone());

    thread::spawn(move || {

        let response = &client.get("http://google.com".parse().unwrap()).and_then(|res| {
            println!("Response: {}", res.status());
            Ok(())
        });

        remote.clone().spawn(|_| {
            response.map(|_| { () }).map_err(|_| { () })
        });
    });

    //    for _ in 0..num_cpus::get() {
    //
    ////        thread::spawn(move || {
    //            remote.clone().spawn(move |handle| {
    ////                let client = Client::new(&handle.clone());
    //                let response = client.clone().get("http://google.com".parse().unwrap()).map(|res| {
    //                    println!("Response: {}", res.status());
    //                    ()
    //                });
    //                response.map(|_| { () }).map_err(|_| { () })
    //            });
    //////            let response = response.map(|_| { () }).map_err(|_| { () });
    ////        });
    //    }

    println!("spawn done!");

    core.run(futures::future::empty::<(), ()>()).unwrap();


    //    pretty_env_logger::init().unwrap();
    //    let mut core = Core::new().expect("Create Client Event Loop");
    //    let handle = core.handle();
    //
    //    let (tx, rx) = futures::sync::oneshot::channel::<&Client<HttpConnector, Body>>();
    //    thread::spawn(move || {
    //        let mut core = Core::new().expect("Create Client Event Loop");
    //        let handle = core.handle();
    //        let client = Client::new(&handle.clone());
    //
    //        tx.send(&client.clone());
    //
    //        core.run(futures::future::empty::<(), ()>()).unwrap();
    //
    //    });
    //
    //    let client1 = rx.wait().unwrap();
    //
    //    let response = client1.get("http://google.com".parse().unwrap()).and_then(|res| {
    //        println!("Response: {}", res.status());
    //        Ok(())
    //    });
}

struct Container;

impl Container {

    fn recusive_call(&self, start: u32) -> u32 {
        if start < 3 {
            self.recusive_call(start + 1)
        } else {
            start
        }
    }
}


fn run_thread(addr: &SocketAddr) {

    let mut core = Core::new().expect("Create Event Loop");

    let handle = core.handle();

    let client = Client::new(&handle);

    let listener = net2::TcpBuilder::new_v4().unwrap()
        .reuse_port(true).unwrap()
        .bind(addr).unwrap()
        .listen(128).unwrap();
    let listener = TcpListener::from_listener(listener, addr, &handle).unwrap();

    let all_conns = listener.incoming().for_each(|(socket, addr)| {
        let service = Proxy { client: client.clone() };
        Http::new().bind_connection(&handle, socket, addr, service);
        Ok(())
    }).map_err(|err| {
        error!("Error with Tcp Listener: {:?}", err);
    });

    core.run(all_conns).unwrap();

}

fn server_start_up() {

    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();

    info!("Starting Server on {:?}...", addr);

    let mut threads = Vec::new();
    for _ in 0..num_cpus::get() {
        //        let listener = listener.try_clone().unwrap();
        threads.push(thread::spawn(move || run_thread(&addr)));
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
