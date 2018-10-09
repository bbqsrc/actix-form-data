extern crate actix_web;
extern crate form_data;
extern crate futures;
extern crate mime;

use std::path::PathBuf;

use actix_web::{http, server, App, AsyncResponder, HttpMessage, HttpRequest, HttpResponse, State};
use form_data::{handle_multipart, Error, Field, FilenameGenerator, Form};
use futures::Future;

struct Gen;

impl FilenameGenerator for Gen {
    fn next_filename(&self, _: &mime::Mime) -> Option<PathBuf> {
        let mut p = PathBuf::new();
        p.push("examples/filename.png");
        Some(p)
    }
}

fn upload(
    (req, state): (HttpRequest<Form>, State<Form>),
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    handle_multipart(req.multipart(), state.clone())
        .map(|uploaded_content| {
            println!("Uploaded Content: {:?}", uploaded_content);
            HttpResponse::Created().finish()
        })
        .responder()
}

fn main() {
    let form = Form::new()
        .field("Hey", Field::text())
        .field(
            "Hi",
            Field::map()
                .field("One", Field::int())
                .field("Two", Field::float())
                .finalize(),
        )
        .field("files", Field::array(Field::file(Gen)));

    println!("{:?}", form);

    server::new(move || {
        App::with_state(form.clone())
            .resource("/upload", |r| r.method(http::Method::POST).with(upload))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}
