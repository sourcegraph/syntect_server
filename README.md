# Syntect Server

This is an HTTP server that exposes the Rust [Syntect](https://github.com/trishume/syntect) syntax highlighting library for use by other services. Send it some code, and it'll send you syntax-highlighted code in response.

Technologies:

- [Syntect](https://github.com/trishume/syntect) -> Syntax highlighting of code.
- [Rocket.rs](https://rocket.rs) -> Web framework.
- [Serde](https://serde.rs/) -> JSON serialization / deserialization .
- [Rayon](https://github.com/nikomatsakis/rayon) -> data parallelism for `SyntaxSet` across Rocket server threads.
- [lazy_static](https://crates.io/crates/lazy_static) -> lazily evaluated static `ThemeSet` (like a global).

## Usage

```bash
docker run --detach --name=syntect_server -p 9238:9238 sourcegraph/syntect_server
```

You can then e.g. `GET` http://localhost:9238/health or http://host.docker.internal:9238/health to confirm it is working.

## API

- `POST` to `/` with `Content-Type: application/json`. The following fields are required:
  - `filepath` string, e.g. `the/file.go` or `file.go` or `Dockerfile`, see "Supported file extensions" section below.
  - `theme` string, e.g. `Solarized (dark)`, see "Embedded themes" section below.
  - `code` string, i.e. the literal code to highlight.
- The response is a JSON object of either:
  - A successful response (`data` field):
    - `data` string with syntax highlighted response. The input `code` string [is properly escaped](https://github.com/sourcegraph/syntect_server/blob/ee3810f70e5701b961b7249393dbac8914c162ce/syntect/src/html.rs#L6) and as such can be directly rendered in the browser safely.
    - `plaintext` boolean indicating whether a syntax could not be found for the file and instead it was rendered as plain text.
  - An error response (`error` field), one of:
    - `{"error": "invalid theme", "code": "invalid_theme"}`
    - `{"error": "resource not found", "code": "resource_not_found"}`
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

## Building Docker image

The Docker image is itself build using Docker (a multistage build).

Run `build.sh`, you can then run it via `docker run -it syntect_server`.

## Publishing docker image

Run `./publish.sh` after merging your changes.

## Code hygiene

- Use `cargo fmt` or an editor extension to format code.

## Adding themes

- Copy a `.tmTheme` file anywhere under `./syntect/testdata` (make a new dir if needed) [in our fork](https://github.com/slimsag/syntect).
- `cd syntect && make assets`
- In this repo, `cargo update -p syntect`.
- Build a new binary.

## Adding languages:

- With a `.tmLanguage` file:
  - example: https://github.com/Microsoft/TypeScript-TmLanguage/blob/master/TypeScript.tmLanguage
  - Ensure it has exact `.tmLanguage` suffix, or else command will not be available.
  - Open the file with Sublime Text 3, press <kbd>Cmd+Shift+P</kbd>.
  - Search for `Plugin Development: Convert Syntax to .sublime-syntax` command.
  - Continue with steps below.
- With a `.sublime-syntax` file:
  - Save the file anywhere under `Packages/MySyntax` [in our fork of sublimehq/Packages](https://github.com/slimsag/Packages).
  - In our fork of syntect
    - update the git submodule
    - run `make assets`
    - commit those changes and submit a PR to the syntect fork
  - Build a new binary.

## Embedded themes:

- `InspiredGitHub`
- `Monokai`
- `Solarized (dark)`
- `Solarized (light)`
- `Sourcegraph`
- `TypeScript`
- `TypeScriptReact`
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
- CMake Cache (`CMakeCache.txt`)
- CMake Listfile (`CMakeLists.txt`, `cmake`)
- CSS (`css`, `css.erb`, `css.liquid`)
- Cap’n Proto (`capnp`)
- Cg (`cg`)
- Clojure (`clj`, `cljs`, `cljc`, `cljx`)
- Crontab (`crontab`)
- D (`d`, `di`)
- Diff (`diff`, `patch`)
- Dockerfile (`Dockerfile`)
- Erlang (`erl`, `hrl`, `Emakefile`, `emakefile`)
- HTML (Erlang) (`yaws`)
- F Sharp (`fs`)
- friendly interactive shell (fish) (`fish`)
- Forth (`frt`, `fs`)
- ESSL (`essl`, `f.essl`, `v.essl`, `_v.essl`, `_f.essl`, `_vs.essl`, `_fs.essl`)
- GLSL (`vs`, `fs`, `gs`, `vsh`, `fsh`, `gsh`, `vshader`, `fshader`, `gshader`, `vert`, `frag`, `geom`, `tesc`, `tese`, `comp`, `glsl`)
- Git Attributes (`attributes`, `gitattributes`, `.gitattributes`)
- Git Commit (`COMMIT_EDITMSG`, `MERGE_MSG`, `TAG_EDITMSG`)
- Git Common (``)
- Git Config (`gitconfig`, `.gitconfig`, `.gitmodules`)
- Git Ignore (`exclude`, `gitignore`, `.gitignore`)
- Git Link (`.git`)
- Git Log (`gitlog`)
- Git Rebase Todo (`git-rebase-todo`)
- Go (`go`)
- Graphviz (DOT) (`dot`, `DOT`, `gv`)
- Groovy (`groovy`, `gvy`, `gradle`)
- HLSL (`fx`, `fxh`, `hlsl`, `hlsli`, `usf`)
- HTML (`html`, `htm`, `shtml`, `xhtml`, `tmpl`, `tpl`)
- Haskell (`hs`)
- Literate Haskell (`lhs`)
- INI (`ini`, `INI`, `INF`, `reg`, `REG`, `lng`, `cfg`, `CFG`, `url`, `URL`, `.editorconfig`)
- Java Server Page (JSP) (`jsp`)
- Java (`java`, `bsh`)
- Javadoc (``)
- Java Properties (`properties`)
- JSON (`json`, `sublime-settings`, `sublime-menu`, `sublime-keymap`, `sublime-mousemap`, `sublime-theme`, `sublime-build`, `sublime-project`, `sublime-completions`, `sublime-commands`, `sublime-macro`, `sublime-color-scheme`)
- JavaScript (Babel) (`js`, `jsx`, `babel`, `es6`)
- Regular Expressions (Javascript) (``)
- LESS (`less`)
- BibTeX (`bib`)
- LaTeX Log (``)
- LaTeX (`tex`, `ltx`)
- TeX (`sty`, `cls`)
- Lisp (`lisp`, `cl`, `clisp`, `l`, `mud`, `el`, `scm`, `ss`, `lsp`, `fasl`)
- Lua (`lua`)
- MSBuild (`proj`, `targets`, `msbuild`, `csproj`, `vbproj`, `fsproj`, `vcxproj`)
- Make Output (``)
- Makefile (`make`, `GNUmakefile`, `makefile`, `Makefile`, `OCamlMakefile`, `mak`, `mk`)
- Man (`man`)
- Markdown (`md`, `mdown`, `markdown`, `markdn`)
- MultiMarkdown (``)
- MATLAB (`matlab`)
- Maven POM (`pom.xml`)
- Mediawiki (`mediawiki`, `wikipedia`, `wiki`)
- Ninja (`ninja`)
- OCaml (`ml`, `mli`)
- OCamllex (`mll`)
- OCamlyacc (`mly`)
- camlp4 (``)
- Objective-C++ (`mm`, `M`, `h`)
- Objective-C (`m`, `h`)
- PHP Source (``)
- PHP (`php`, `php3`, `php4`, `php5`, `php7`, `phps`, `phpt`, `phtml`)
- Regular Expressions (PHP) (``)
- Pascal (`pas`, `p`, `dpr`)
- Perforce Client Specification (`spec`, `client`)
- Perl (`pl`, `pm`, `pod`, `t`, `PL`)
- Property List (XML) (``)
- Postscript (`ps`, `eps`)
- PowerShell (`ps1`, `psm1`, `psd1`)
- Python (`py`, `py3`, `pyw`, `pyi`, `pyx`, `pyx.in`, `pxd`, `pxd.in`, `pxi`, `pxi.in`, `rpy`, `cpy`, `SConstruct`, `Sconstruct`, `sconstruct`, `SConscript`, `gyp`, `gypi`, `Snakefile`, `wscript`)
- Regular Expressions (Python) (``)
- R Console (``)
- R (`R`, `r`, `s`, `S`, `Rprofile`)
- Rd (R Documentation) (`rd`)
- HTML (Rails) (`rails`, `rhtml`, `erb`, `html.erb`)
- JavaScript (Rails) (`js.erb`)
- Ruby Haml (`haml`)
- Ruby on Rails (`rxml`, `builder`)
- SQL (Rails) (`erbsql`, `sql.erb`)
- Regular Expression (`re`)
- reStructuredText (`rst`, `rest`)
- Ruby (`rb`, `Appfile`, `Appraisals`, `Berksfile`, `Brewfile`, `capfile`, `cgi`, `Cheffile`, `config.ru`, `Deliverfile`, `Fastfile`, `fcgi`, `Gemfile`, `gemspec`, `Guardfile`, `irbrc`, `jbuilder`, `podspec`, `prawn`, `rabl`, `rake`, `Rakefile`, `Rantfile`, `rbx`, `rjs`, `ruby.rail`, `Scanfile`, `simplecov`, `Snapfile`, `thor`, `Thorfile`, `Vagrantfile`)
- Cargo Build Results (``)
- Rust Enhanced (`rs`)
- Sass (`sass`, `scss`)
- SQL (`sql`, `ddl`, `dml`)
- Scala (`scala`, `sbt`)
- Bourne Again Shell (bash) (`sh`, `bash`, `zsh`, `fish`, `.bash_aliases`, `.bash_completions`, `.bash_functions`, `.bash_login`, `.bash_logout`, `.bash_profile`, `.bash_variables`, `.bashrc`, `.profile`, `.textmate_init`)
- Shell-Unix-Generic (``)
- commands-builtin-shell-bash (``)
- Smalltalk (`st`)
- Swift (`swift`)
- HTML (Tcl) (`adp`)
- Tcl (`tcl`)
- TOML (`toml`, `tml`, `lock`)
- Textile (`textile`)
- Thrift (`thrift`)
- TypeScript (`ts`)
- TypeScriptReact (`tsx`)
- XML (`xml`, `xsd`, `xslt`, `tld`, `dtml`, `rss`, `opml`, `svg`)
- YAML (`yaml`, `yml`, `sublime-syntax`)
