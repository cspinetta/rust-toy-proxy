#[macro_use]
extern crate hyper;
extern crate futures;
extern crate tokio_core;

use futures::Future;
use futures::Stream;

use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response};
use hyper::client::HttpConnector;
use hyper::Get;

use tokio_core::reactor::Core;
//use error::HttpError;
use std::sync::Arc;

fn main() {

    let mut core = Core::new().expect("Event Loop");
    let handle = core.handle();
    let client = Client::new(&handle.clone());

    let req = Request::new(Get, "http://localhost:8080/echo".parse().unwrap());
    let resp = ClientHttp{ max_retry: Arc::new(10) }.dispatch_request(&client, req, 1);

    core.run(resp.map(|_| {()}).map_err(|_| {()}));
}


//impl Clone for ReusableBody {
//    fn clone(&self) -> ReusableBody { *self }
//}

struct ClientHttp {
    max_retry: Arc<u32>
}

impl ClientHttp {


//fn get_body_contents(body: Body) -> Result<Vec<u8>, Error> {
//    let body = body.wait().fold(Ok(Vec::new()), |r, input| {
//        if let Ok(mut v) = r {
//            input.map(move |next_body_chunk| { v.extend_from_slice(&next_body_chunk); v })
//        } else {
//            r
//        }
//    });
//    body
////    if let Some(reader) = body.take() {
//////        match body {
//////            Ok(body) => self.extensions.insert::<RequestBodyKey>(body),
//////            Err(e) => return Err(e),
//////        };
////        body
////    } else {
////        Body::empty()
////    }
//}

    fn clone_req(req: & Request) -> Request {
        let mut forwarded_req = Request::new(req.method().clone(), req.uri().clone());
        forwarded_req.headers_mut().extend(req.headers().iter());
    //    forwarded_req.set_body(*body.clone());
        forwarded_req
    }

    fn dispatch_request(self, client: &Client<HttpConnector, Body>, req: Request<Body>, n_retry: u32) -> Box<Future<Error=hyper::Error, Item=Response>>
    {
        println!("Attemp {}", n_retry);
//        let max_retry = Arc::new(self.max_retry);

        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();


    //    let (method, uri, version, headers, body) = req.deconstruct();

        let cloned_req = ClientHttp::clone_req(&req);

        let resp = client.request(req).then(move |result| {
            println!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
            match result {
                Ok(client_resp) => {
                    if client_resp.status() == hyper::StatusCode::Ok {
                        Box::new(futures::future::ok(client_resp))
                    } else if n_retry < *ref_max.clone() {
                        self.dispatch_request(&client_clone, ClientHttp::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }
                },
                Err(e) => {
                    println!("Connection error: {:?}", &e);
                    if n_retry < *ref_max.clone() {
                        self.dispatch_request(&client_clone, ClientHttp::clone_req(&cloned_req), n_retry + 1)
                    } else {
                        Box::new(futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable)))
                    }

                }
            }
        });
        Box::new(resp)
    }
}
