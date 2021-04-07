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
use std::panic;
use syntect::{
    highlighting::ThemeSet,
    parsing::{Scope, ScopeStack, SCOPE_REPO, ScopeStackOp, SyntaxSet, BasicScopeStackOp, SyntaxReference, ParseState},
    util::LinesWithEndings,
    html::{highlighted_html_for_string, ClassStyle},
};
use std::fmt::{self, Write};

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

#[derive(Deserialize)]
struct CSSTableQuery {
    filepath: String,
    code: String,
    line_length_limit: Option<usize>,
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

#[post("/css_table", format = "application/json", data = "<q>")]
fn css_table_index(q: Json<CSSTableQuery>) -> JsonValue {
    // TODO(slimsag): In an ideal world we wouldn't be relying on catch_unwind
    // and instead Syntect would return Result types when failures occur. This
    // will require some non-trivial work upstream:
    // https://github.com/trishume/syntect/issues/98
    match panic::catch_unwind(|| css_table_highlight(q)) {
        Ok(v) => v,
        Err(_) => json!({"error": "panic while highlighting code", "code": "panic"}),
    }
}

fn css_table_highlight(q: Json<CSSTableQuery>) -> JsonValue {
    SYNTAX_SET.with(|syntax_set| {

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
        let syntax_def = syntax_set.find_syntax_by_extension(file_name)
            .or_else(|| syntax_set.find_syntax_by_extension(extension))
            .or_else(|| syntax_set.find_syntax_by_first_line(&q.code))
            .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

        let output = ClassedTableGenerator::new(
            &syntax_set,
            &syntax_def,
            &q.code,
            q.line_length_limit
        ).generate();

        json!({
            "data": output,
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
        .mount("/css_table", routes![css_table_index, health])
        .register(catchers![not_found])
}

pub struct ClassedTableGenerator<'a> {
    syntax_set: &'a SyntaxSet,
    parse_state: ParseState,
    stack: ScopeStack,
    html: String,
    style: ClassStyle,
    code: &'a str,
    max_line_len: Option<usize>,
}


impl<'a> ClassedTableGenerator<'a> {
    fn new(ss: &'a SyntaxSet, sr: &SyntaxReference, code: &'a str, max_line_len: Option<usize>) -> Self {
        ClassedTableGenerator{
            code,
            syntax_set: ss,
            parse_state: ParseState::new(sr),
            stack: ScopeStack::new(),
            html: String::with_capacity(code.len() * 8),
            style: ClassStyle::Spaced,
            max_line_len,
        }
    }

    // generate takes ownership of self so that it can't be re-used
    fn generate(mut self) -> String {
        self.open_table();

        for (i, line) in LinesWithEndings::from(self.code).enumerate() {
            self.open_row(i);
            if self.max_line_len.map_or(false, |n| line.len() > n) {
                self.html.push_str(line.strip_suffix("\n").unwrap_or(line));
            } else {
                self.write_spans_for_line(&line);
            }
            self.close_row();
        }

        self.close_table();
        self.html
    }

    fn open_table(&mut self) {
        self.html.push_str("<table><tbody>");
    }

    fn close_table(&mut self) {
        self.html.push_str("</tbody></table>");
    }

    fn open_row(&mut self, i: usize) {
        write!(&mut self.html, "<tr><td class=\"line\" data-line=\"{}\"/><td class=\"code\">", i+1).unwrap();
    }

    fn close_row(&mut self) {
        self.html.push_str("</td></tr>");
    }

    fn open_current_scopes(&mut self) {
        for scope in self.stack.clone().as_slice() {
            self.open_scope(scope)
        }
    }

    fn close_current_scopes(&mut self) {
        for _ in 0..self.stack.len() {
            self.html.push_str("</span>")
        }
    }

    fn open_scope(&mut self, scope: &Scope) {
        self.html.push_str("<span class=\"");
        ClassedTableGenerator::scope_to_classes(&mut self.html, *scope, self.style);
        self.html.push_str("\">");
    }

    fn close_scope(&mut self) {
        self.html.push_str("</span>");
    }


    fn write_spans_for_line(&mut self, line: &str) {
        self.open_current_scopes();
        let parsed_line = self.parse_state.parse_line(line, self.syntax_set);
        self.tokens_to_classed_spans(line, parsed_line.as_slice());
        self.close_current_scopes();
    }

    fn tokens_to_classed_spans(&mut self, line: &str, ops: &[(usize, ScopeStackOp)]) {
        let mut cur_index = 0;

        // check and skip empty inner <span> tags
        let mut span_empty = false;
        let mut span_start = 0;

        for &(i, ref op) in ops {
            if i > cur_index {
                span_empty = false;
                write!(&mut self.html, "{}", Escape(&line[cur_index..i])).unwrap();
                cur_index = i
            }
            let mut stack = self.stack.clone();
            stack.apply_with_hook(op, |basic_op, _| {
                match basic_op {
                    BasicScopeStackOp::Push(scope) => {
                        span_start = self.html.len();
                        span_empty = true;
                        self.open_scope(&scope);
                    }
                    BasicScopeStackOp::Pop => {
                        if span_empty {
                            self.html.truncate(span_start);
                        } else {
                            self.close_scope();
                        }
                        span_empty = false;
                    }
                }
            });
            self.stack = stack;
        }
        write!(&mut self.html, "{}", Escape(&line[cur_index..line.len()])).unwrap();
    }

    fn scope_to_classes(s: &mut String, scope: Scope, style: ClassStyle) {
        let repo = SCOPE_REPO.lock().unwrap();
        for i in 0..(scope.len()) {
            let atom = scope.atom_at(i as usize);
            let atom_s = repo.atom_str(atom);
            if i != 0 {
                s.push_str(" ")
            }
            match style {
                ClassStyle::Spaced => {},
                ClassStyle::SpacedPrefixed{prefix} => s.push_str(&prefix),
                _ => unreachable!(),
            }
            s.push_str(atom_s);
        }
    }
}




/// Wrapper struct which will emit the HTML-escaped version of the contained
/// string when passed to a format string.
pub struct Escape<'a>(pub &'a str);

impl<'a> fmt::Display for Escape<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Because the internet is always right, turns out there's not that many
        // characters to escape: http://stackoverflow.com/questions/7381974
        let Escape(s) = *self;
        let pile_o_bits = s;
        let mut last = 0;
        for (i, ch) in s.bytes().enumerate() {
            match ch as char {
                '<' | '>' | '&' | '\'' | '"' => {
                    fmt.write_str(&pile_o_bits[last..i])?;
                    let s = match ch as char {
                        '>' => "&gt;",
                        '<' => "&lt;",
                        '&' => "&amp;",
                        '\'' => "&#39;",
                        '"' => "&quot;",
                        _ => unreachable!(),
                    };
                    fmt.write_str(s)?;
                    last = i + 1;
                }
                _ => {}
            }
        }

        if last < s.len() {
            fmt.write_str(&pile_o_bits[last..])?;
        }
        Ok(())
    }
}
