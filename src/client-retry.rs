#[macro_use]
extern crate hyper;
extern crate futures;
extern crate tokio_core;

use futures::Future;
use futures::Stream;
use futures::stream::Concat2;

use hyper::{Client, Body, Uri, StatusCode};
use hyper::server::{Request, Response};
use hyper::client::HttpConnector;
use hyper::Get;
use hyper::header::ContentLength;
use hyper::HttpVersion;
use hyper::Method;
use hyper::Chunk;

use hyper::header::{Headers, Host};

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


#[derive(Clone, Debug, PartialEq)]
struct RequestHead {
    pub version: HttpVersion,
    pub method: Method,
    pub uri: Uri,
    pub headers: Headers
}

impl RequestHead {
    fn new(version: HttpVersion, method: Method, uri: Uri, headers: Headers) -> RequestHead {
        RequestHead { version, method, uri, headers }
    }

    fn from_request(req: &Request) -> Self {
        Self::new(req.version().clone(), req.method().clone(), req.uri().clone(), req.headers().clone())
    }
}

struct ClientHttp {
    max_retry: Arc<u32>
}

impl ClientHttp {

    fn get_content_length_req(req: &Request<Body>) -> Option<u64> {
        req.headers().get::<ContentLength>().map(|length| {length.0})
    }

    fn clone_req(req: & Request) -> Request {
        let mut forwarded_req = Request::new(req.method().clone(), req.uri().clone());
        forwarded_req.headers_mut().extend(req.headers().iter());
        forwarded_req
    }

    fn copy_body(chunk_ref: &Chunk) -> Chunk {
        Chunk::from(chunk_ref.as_ref().clone().to_vec())
    }

    fn build_req(head: &RequestHead, body: Chunk) -> Request {
        let mut new_req = Request::new(head.method.clone(), head.uri.clone());
        new_req.headers_mut().extend(head.headers.iter());
        new_req.set_version(head.version.clone());
        new_req.set_body(body);
        new_req
    }

    fn dispatch_request(self, client: &Client<HttpConnector, Body>, req: Request<Body>, n_retry: u32) -> Box<Future<Error=hyper::Error, Item=Response>>
    {
        println!("Attemp {}", n_retry);

        let client_clone = client.clone();
        let ref_max = self.max_retry.clone();

        let head = RequestHead::from_request(&req);
        let content_length = Self::get_content_length_req(&req);


        match content_length {
            Some(length) if length <= 1024 => {
                println!("The request has a small length, so it's possible to retry. Content Length: {}", length);
                let shared_body: Concat2<Body> = req.body().concat2();

                let r = shared_body.and_then(move |whole_body| {

                    let new_req = Self::build_req(&head.clone(), Self::copy_body(&whole_body));

                    println!("Attemp {} for url: {:?}", n_retry, new_req);

                    let resp = client_clone.request(new_req).then(move |result| {
                        println!("Max retry: {}. Current attemp: {}", ref_max.clone(), n_retry);
                        match result {
                            Ok(client_resp) => Box::new(futures::future::ok(client_resp)),
                            Err(e) => {
                                println!("Connection error: {:?}", &e);
                                if (n_retry < *ref_max.clone()) && (head.method == Get) {

                                    let new_req = Self::build_req(&head.clone(), Self::copy_body(&whole_body));
                                    self.dispatch_request(&client_clone, new_req, n_retry + 1)
                                } else {
                                    Box::new(futures::future::ok(Response::new().with_status(StatusCode::ServiceUnavailable)))
                                }

                            }
                        }
                    });
                    Box::new(resp)
                });
                Box::new(r)

            },
            _ => {
                println!("The request has a length enough big or unknown, so it's not possible to consume the body and retry");
                Box::new(client_clone.request(req))
            }
        }
    }
}
