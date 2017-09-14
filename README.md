# Syntect Server [![Build Status](https://travis-ci.org/sourcegraph/syntect_server.svg?branch=master)](https://travis-ci.org/sourcegraph/syntect_server)

This is an HTTP server that exposes the Rust [Syntect](https://github.com/trishume/syntect) syntax highlighting library for use by other services. Send it some code, and it'll send you syntax-highlighted code in response.

Technologies:

- [Syntect](https://github.com/trishume/syntect) -> Syntax highlighting of code.
- [Rocket.rs](https://rocket.rs) -> Web framework.
- [Serde](https://serde.rs/) -> JSON serialization / deserialization .
- [Rayon](https://github.com/nikomatsakis/rayon) -> data parallelism for `SyntaxSet` across Rocket server threads.
- [lazy_static](https://crates.io/crates/lazy_static) -> lazily evaluated static `ThemeSet` (like a global).

## API

- `POST` to `/` with `Content-Type: application/json`. The following fields are required:
  - `extension` string, e.g. `go`, see "Supported file extensions" section below.
  - `theme` string, e.g. `Solarized (dark)`, see "Embedded themes" section below.
  - `code` string, i.e. the literal code to highlight.
- The response is a JSON object (as long as request was a JSON object with all required fields present) containing:
  - `data` string with syntax highlighted response. The input `code` string [is properly escaped](https://github.com/sourcegraph/syntect_server/blob/ee3810f70e5701b961b7249393dbac8914c162ce/syntect/src/html.rs#L6) and as such can be directly rendered in the browser safely.
  - Otherwise, an `error` string which describes the problem.
- `GET` to `/health` to receive an `OK` health check response / ensure the service is alive.

## Client

[gosyntect](https://github.com/sourcegraph/gosyntect) is a Go package + CLI program to make requests against syntect_server.

## Configuration

By default on startup, `syntect_server` will list all features (themes + file types) it supports. This can be disabled by setting `QUIET=true` in the environment.

## Development

1. [Install Rust **nightly**](https://rocket.rs/guide/getting-started/#installing-rust).
2. `git clone` this repository anywhere on your filesystem.
3. Use `cargo run` to download dependencies + compile + run the server.

## Building

Invoke `cargo build --release` and an optimized binary will be built (e.g. to `./target/debug/syntect_server`).

## Building docker image

Note: Docker images are automatically published for the `master` branch by Travis. The below steps are for building / publishing manually.

- Mac: `brew install filosottile/musl-cross/musl-cross`
- `rustup target add x86_64-unknown-linux-musl`
- `./build.sh` -> then `./publish.sh` to push the docker image.

You can then run it via `docker run -it syntect_server`.

## Code hygiene

- Use `cargo fmt` or an editor extension to format code.

## Adding themes

- Copy a `.tmTheme` file anywhere under `./syntect/testdata` (make a new dir if needed).
- `cd syntect && make assets`
- Build a new binary.

## Adding languages:

- With a `.tmLanguage` file:
  - example: https://github.com/Microsoft/TypeScript-TmLanguage/blob/master/TypeScript.tmLanguage
  - Ensure it has exact `.tmLanguage` suffix, or else command will not be available.
  - Open the file with Sublime Text 3, press <kbd>Cmd+Shift+P</kbd>.
  - Search for `Plugin Development: Convert Syntax to .sublime-syntax` command.
  - Continue with steps below.
- With a `.sublime-syntax` file:
  - Save the file anywhere under `./syntect/testdata/Packages` (make a new dir if needed).
  - `cd syntect && make assets`
  - Build a new binary.

## Embedded themes:

- `InspiredGitHub`
- `Monokai`
- `Solarized (dark)`
- `Solarized (light)`
- `Visual Studio`
- `Visual Studio Dark`
- `base16-eighties.dark`
- `base16-mocha.dark`
- `base16-ocean.dark`
- `base16-ocean.light`

## Supported file extensions:

- Plain Text (`txt`)
- ASP (`asa`)
- HTML (ASP) (`asp`)
- ActionScript (`as`)
- AppleScript (`applescript`, `script editor`)
- Batch File (`bat`, `cmd`)
- NAnt Build File (`build`)
- C# (`cs`, `csx`)
- C++ (`cpp`, `cc`, `cp`, `cxx`, `c++`, `C`, `h`, `hh`, `hpp`, `hxx`, `h++`, `inl`, `ipp`)
- C (`c`, `h`)
- CSS (`css`, `css.erb`, `css.liquid`)
- Clojure (`clj`)
- D (`d`, `di`)
- Diff (`diff`, `patch`)
- Erlang (`erl`, `hrl`, `Emakefile`, `emakefile`)
- HTML (Erlang) (`yaws`)
- Go (`go`)
- Graphviz (DOT) (`dot`, `DOT`)
- Groovy (`groovy`, `gvy`, `gradle`)
- HTML (`html`, `htm`, `shtml`, `xhtml`, `inc`, `tmpl`, `tpl`)
- Haskell (`hs`)
- Literate Haskell (`lhs`)
- Java Server Page (JSP) (`jsp`)
- Java (`java`, `bsh`)
- JavaDoc (``)
- Java Properties (`properties`)
- JSON (`json`, `sublime-settings`, `sublime-menu`, `sublime-keymap`, `sublime-mousemap`, `sublime-theme`, `sublime-build`, `sublime-project`, `sublime-completions`, `sublime-commands`, `sublime-macro`)
- JavaScript (Babel) (`js`, `jsx`, `babel`, `es6`)
- Regular Expressions (Javascript) (``)
- BibTeX (`bib`)
- LaTeX Log (``)
- LaTeX (`tex`, `ltx`)
- TeX (`sty`, `cls`)
- Lisp (`lisp`, `cl`, `l`, `mud`, `el`, `scm`, `ss`, `lsp`, `fasl`)
- Lua (`lua`)
- Make Output (``)
- Makefile (`make`, `GNUmakefile`, `makefile`, `Makefile`, `OCamlMakefile`, `mak`, `mk`)
- Markdown (`md`, `mdown`, `markdown`, `markdn`)
- MultiMarkdown (``)
- MATLAB (`matlab`)
- OCaml (`ml`, `mli`)
- OCamllex (`mll`)
- OCamlyacc (`mly`)
- camlp4 (``)
- Objective-C++ (`mm`, `M`, `h`)
- Objective-C (`m`, `h`)
- PHP Source (``)
- PHP (`php`, `php3`, `php4`, `php5`, `php7`, `phps`, `phpt`, `phtml`)
- Pascal (`pas`, `p`, `dpr`)
- Perl (`pl`, `pm`, `pod`, `t`, `PL`)
- Python (`py`, `py3`, `pyw`, `pyi`, `rpy`, `cpy`, `SConstruct`, `Sconstruct`, `sconstruct`, `SConscript`, `gyp`, `gypi`, `Snakefile`, `wscript`)
- Regular Expressions (Python) (``)
- R Console (``)
- R (`R`, `r`, `s`, `S`, `Rprofile`)
- Rd (R Documentation) (`rd`)
- HTML (Rails) (`rails`, `rhtml`, `erb`, `html.erb`)
- JavaScript (Rails) (`js.erb`)
- Ruby Haml (`haml`, `sass`)
- Ruby on Rails (`rxml`, `builder`)
- SQL (Rails) (`erbsql`, `sql.erb`)
- Regular Expression (`re`)
- reStructuredText (`rst`, `rest`)
- Ruby (`rb`, `Appfile`, `Appraisals`, `Berksfile`, `Brewfile`, `capfile`, `cgi`, `Cheffile`, `config.ru`, `Deliverfile`, `Fastfile`, `fcgi`, `Gemfile`, `gemspec`, `Guardfile`, `irbrc`, `jbuilder`, `podspec`, `prawn`, `rabl`, `rake`, `Rakefile`, `Rantfile`, `rbx`, `rjs`, `ruby.rail`, `Scanfile`, `simplecov`, `Snapfile`, `thor`, `Thorfile`, `Vagrantfile`)
- Cargo Build Results (``)
- Rust (`rs`)
- SQL (`sql`, `ddl`, `dml`)
- Scala (`scala`, `sbt`)
- Shell Script (Bash) (`sh`, `bash`, `zsh`, `.bash_aliases`, `.bash_functions`, `.bash_login`, `.bash_logout`, `.bash_profile`, `.bash_variables`, `.bashrc`, `.profile`, `.textmate_init`)
- Swift (`swift`)
- HTML (Tcl) (`adp`)
- Tcl (`tcl`)
- Textile (`textile`)
- TypeScript (`ts`)
- TypeScriptReact (`tsx`)
- XML (`xml`, `xsd`, `xslt`, `tld`, `dtml`, `rss`, `opml`, `svg`)
- YAML (`yaml`, `yml`, `sublime-syntax`)
