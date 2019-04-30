extern crate hyper;
extern crate futures;

use futures::future;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::rt::Future;
use hyper::service::service_fn;

const PHRASE: &str = "Hello, World!";

trait Component {
    fn render(&self) -> String;
}

struct Text<'a> {
    content: &'a str,
}

impl <'a> Text<'a> {
    fn new(content: &'a str) -> Self {
        Self {
            content,
        }
    }
}

impl <'a> Component for Text<'a> {
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

    fn render(&self) -> String {
        let mut buf = String::new();
        buf.push_str("<!DOCTYPE html>\n");
        for c in &self.components {
            buf.push_str(&c.render());
        }

        buf
    }
}

fn homepage() -> Document {
    let text = Text::new(PHRASE);
    let mut doc = Document::new();
    doc.insert_element(Box::new(text));

    doc
}

impl From<Document> for String {
    fn from(doc: Document) -> String {
        doc.render()
    }
}

// Just a simple type alias
type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

fn handler(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let doc = homepage();
            *response.body_mut() = Body::from(String::from(doc));
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        },
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