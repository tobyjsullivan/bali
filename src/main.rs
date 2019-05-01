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

struct Image {
    src: String,
}

impl Image {
    fn new(src: &str) -> Self {
        Self {
            src: src.to_owned(),
        }
    }
}

impl Component for Image {
    fn render(&self) -> String {
        let mut buf = String::new();
        let content = format!("<img src=\"{}\" />", self.src);
        buf.push_str(content.as_ref());

        buf
    }
}

struct Document {
    components: Vec<Box<Component>>,
}

trait Resource {
    fn headers(&self) -> HeaderMap;
    fn body(&self) -> Body;
}

impl Document {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    fn insert_element(&mut self, el: Box<Component>) {
        self.components.push(el);
    }
}

impl Resource for Document {
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

enum DataSource {
    Static { content: &'static [u8] },
}

impl DataSource {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            DataSource::Static { content } => content.to_vec(),
        }
    }
}

struct File {
    content: DataSource,
    mime_type: String,
}

impl File {
    fn new(content: DataSource, mime_type: &str) -> Self {
        Self {
            content,
            mime_type: mime_type.to_owned(),
        }
    }
}

impl Resource for File {
    fn headers(&self) -> HeaderMap {
        let mut headers: HeaderMap = HeaderMap::default();
        headers.insert("content-type", self.mime_type.parse().unwrap());
        headers.insert("server", SERVER_NAME.parse().unwrap());

        headers
    }

    fn body(&self) -> Body {
        Body::from(self.content.as_bytes())
    }
}

fn homepage() -> Document {
    let text = Text::new(PHRASE);
    let img = Image::new("/cat.jpg");
    let mut doc = Document::new();
    doc.insert_element(Box::new(text));
    doc.insert_element(Box::new(img));

    doc
}

fn img_cow() -> File {
    let ds = DataSource::Static {
        content: include_bytes!("images/cow.jpg"),
    };
    File::new(ds, "image/jpeg")
}

fn img_cat() -> File {
    let ds = DataSource::Static {
        content: include_bytes!("images/cat.jpg"),
    };
    File::new(ds, "image/jpeg")
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
        (&Method::GET, "/cow.jpg") => {
            let img = img_cow();
            *response.headers_mut() = img.headers();
            *response.body_mut() = img.body();
        }
        (&Method::GET, "/cat.jpg") => {
            let img = img_cat();
            *response.headers_mut() = img.headers();
            *response.body_mut() = img.body();
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
