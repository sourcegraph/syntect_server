#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]

#[macro_use] extern crate lazy_static;
extern crate rayon;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate syntect;

use rocket_contrib::json::{Json, JsonValue};
use std::env;
use std::path::Path;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use std::panic;
use syntect::html::{highlighted_html_for_string};

thread_local! {
    static SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
}

lazy_static! {
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

#[derive(Deserialize)]
struct Query {
    // Deprecated field with a default empty string value, kept for backwards
    // compatability with old clients.
    #[serde(default)]
    extension: String,

    // default empty string value for backwards compat with clients who do not specify this field.
    #[serde(default)]
    filepath: String,

    theme: String,
    code: String,
}

#[post("/", format = "application/json", data = "<q>")]
fn index(q: Json<Query>) -> JsonValue {
    // TODO(slimsag): In an ideal world we wouldn't be relying on catch_unwind
    // and instead Syntect would return Result types when failures occur. This
    // will require some non-trivial work upstream:
    // https://github.com/trishume/syntect/issues/98
    let result = panic::catch_unwind(|| {
        highlight(q)
    });
    match result {
        Ok(v) => v,
        Err(_) => json!({"error": "panic while highlighting code", "code": "panic"}),
    }
}

fn highlight(q: Json<Query>) -> JsonValue {
    SYNTAX_SET.with(|syntax_set| {
        // Determine theme to use.
        //
        // TODO(slimsag): We could let the query specify the theme file's actual
        // bytes? e.g. via `load_from_reader`.
        let theme = match THEME_SET.themes.get(&q.theme) {
            Some(v) => v,
            None => return json!({"error": "invalid theme", "code": "invalid_theme"}),
        };

        // Determine syntax definition by extension.
        let mut is_plaintext = false;
        let syntax_def = if q.filepath == "" {
            // Legacy codepath, kept for backwards-compatability with old clients.
            match syntax_set.find_syntax_by_extension(&q.extension) {
                Some(v) => v,
                None =>
                    // Fall back: Determine syntax definition by first line.
                    match syntax_set.find_syntax_by_first_line(&q.code) {
                        Some(v) => v,
                        None => return json!({"error": "invalid extension"}),
                },
            }
        } else {
            // Split the input path ("foo/myfile.go") into file name
            // ("myfile.go") and extension ("go").
            let path = Path::new(&q.filepath);
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let extension = path.extension().and_then(|x| x.to_str()).unwrap_or("");

            // To determine the syntax definition, we must first check using the
            // filename as some syntaxes match an "extension" that is actually a
            // whole file name (e.g. "Dockerfile" or "CMakeLists.txt"); see e.g. https://github.com/trishume/syntect/pull/170
            //
            // After that, if we do not find any syntax, we can actually check by
            // extension and lastly via the first line of the code.

            // First try to find a syntax whose "extension" matches our file
            // name. This is done due to some syntaxes matching an "extension"
            // that is actually a whole file name (e.g. "Dockerfile" or "CMakeLists.txt")
            // see https://github.com/trishume/syntect/pull/170
            match syntax_set.find_syntax_by_extension(file_name) {
                Some(v) => v,
                None => 
                    // Now try to find the syntax by the actual file extension.
                    match syntax_set.find_syntax_by_extension(extension) {
                        Some(v) => v,
                        None =>
                            // Fall back: Determine syntax definition by first line.
                            match syntax_set.find_syntax_by_first_line(&q.code) {
                                Some(v) => v,
                                None => {
                                    is_plaintext = true;

                                    // Render plain text, so the user gets the same HTML
                                    // output structure.
                                    syntax_set.find_syntax_plain_text()
                                }
                        },
                    }
            }
        };

        // TODO(slimsag): return the theme's background color (and other info??) to caller?
        // https://github.com/trishume/syntect/blob/c8b47758a3872d478c7fc740782cd468b2c0a96b/examples/synhtml.rs#L24

        json!({
            "data": highlighted_html_for_string(&q.code, &syntax_set, &syntax_def, theme),
            "plaintext": is_plaintext,
        })
    })
}

#[get("/health")]
fn health() -> &'static str {
    "OK"
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({"error": "resource not found", "code": "resource_not_found"})
}

fn list_features() {
    // List embedded themes.
    println!("## Embedded themes:");
    println!("");
    for t in THEME_SET.themes.keys() {
        println!("- `{}`", t);
    }
    println!("");

    // List supported file extensions.
    SYNTAX_SET.with(|syntax_set| {
        println!("## Supported file extensions:");
        println!("");
        for sd in syntax_set.syntaxes() {
            println!("- {} (`{}`)", sd.name, sd.file_extensions.join("`, `"));
        }
        println!("");
    });
}

#[launch]
fn rocket() -> rocket::Rocket {
    // Only list features if QUIET != "true"
    match env::var("QUIET") {
        Ok(v) => if v != "true" {
            list_features()
        },
        Err(_) => list_features(),
    };

    rocket::ignite()
        .mount("/", routes![index, health])
        .register(catchers![not_found])
}