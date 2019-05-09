# Actix Form Data
A library for retrieving form data from Actix Web's multipart streams. It can stream uploaded files
onto the filesystem (its main purpose), but it can also parse associated form data.

[documentation](https://docs.rs/actix-form-data)

### Usage

Add it to your dependencies.
```toml
# Cargo.toml

[dependencies]
actix-web = "1.0.0-beta.3"
actix-multipart = "0.1.0-beta.1"
actix-form-data = "0.4.0-beta.2"
```

Require it in your project.
```rust
// src/lib.rs or src/main.rs

use form_data::{Field, Form, Value};
```

#### Overview
First, you'd create a form structure you want to parse from the multipart stream.
```rust
let form = Form::new().field("field-name", Field::text());
```
This creates a form with one required field named "field-name" that will be parsed as text.

Then, pass it to `handle_multipart` in your request handler.
```rust
fn request_handler(mp: Multipart, state: Data<State>) -> ... {
    let future = form_data::handle_multipart(mp, state.form);

    ...
}
```

This returns a `Future<Item = Value, Error = form_data::Error>`, which can be used to
fetch your data.

```rust
let field_value = match value {
    Value::Map(mut hashmap) => {
        hashmap.remove("field-name")?
    }
    _ => return None,
};
```

#### Example
```rust
/// examples/simple.rs

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
}
```

### Contributing
Feel free to open issues for anything you find an issue with. Please note that any contributed code will be licensed under the GPLv3.

### License

Copyright © 2018 Riley Trautman

Actix Form Data is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Actix Form Data is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details. This file is part of Actix Form Data.

You should have received a copy of the GNU General Public License along with Actix Form Data. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
