use std::path::PathBuf;

use actix_multipart::Multipart;
use actix_web::{
    web::{post, resource, Data},
    App, HttpResponse, HttpServer,
};
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

fn upload((mp, state): (Multipart, Data<Form>)) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        handle_multipart(mp, state.get_ref().clone()).map(|uploaded_content| {
            println!("Uploaded Content: {:?}", uploaded_content);
            HttpResponse::Created().finish()
        }),
    )
}

fn main() -> Result<(), failure::Error> {
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

    HttpServer::new(move || {
        App::new()
            .data(form.clone())
            .service(resource("/upload").route(post().to(upload)))
    })
    .bind("127.0.0.1:8080")?
    .run()?;

    Ok(())
}
