extern crate futures;
extern crate hyper;

use futures::future;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::{Body, HeaderMap, Method, Request, Response, Server, StatusCode};

const SERVER_NAME: &str = "bali/1.0";
const PHRASE: &str = "Hello, World!";

trait Component {
    fn render(&self) -> String;
}

struct Text<'a> {
    content: &'a str,
}

impl<'a> Text<'a> {
    fn new(content: &'a str) -> Self {
        Self { content }
    }
}

impl<'a> Component for Text<'a> {
    fn render(&self) -> String {
        self.content.to_owned()
    }
}

struct Document {
    components: Vec<Box<Component>>,
}

impl Document {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    fn insert_element<'a>(&mut self, el: Box<Component>) {
        self.components.push(el);
    }

    fn headers(&self) -> HeaderMap {
        let mut headers: HeaderMap = HeaderMap::default();
        headers.insert("content-type", "text/html".parse().unwrap());
        headers.insert("server", SERVER_NAME.parse().unwrap());

        headers
    }

    fn body(&self) -> Body {
        let mut buf = String::new();
        buf.push_str("<!DOCTYPE html>\n");
        for c in &self.components {
            buf.push_str(&c.render());
        }

        Body::from(buf)
    }
}

fn homepage() -> Document {
    let text = Text::new(PHRASE);
    let mut doc = Document::new();
    doc.insert_element(Box::new(text));

    doc
}

// Just a simple type alias
type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn handler(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let doc = homepage();
            *response.headers_mut() = doc.headers();
            *response.body_mut() = doc.body();
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Box::new(future::ok(response))
}

fn main() {
    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`

        service_fn(handler)
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    hyper::rt::run(server);
}
