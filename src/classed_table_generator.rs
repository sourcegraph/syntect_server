use std::fmt::Write;
use syntect::{
    escape::Escape,
    html::ClassStyle,
    parsing::{
        BasicScopeStackOp, ParseState, Scope, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet,
        SCOPE_REPO,
    },
    util::LinesWithEndings,
};

/// The ClassedTableGenerator generates HTML tables of the following form:
/// <table>
///   <tbody>
///     <tr>
///       <td class="line" data-line="1">
///       <td class="code">
///         <span class="hl-source hl-go">
///           <span class="hl-keyword hl-control hl-go">package</span>
///           main
///         </span>
///       </td>
///     </tr>
///   </tbody>
/// </table
///
/// If max_line_len is not None, any lines with length greater than the
/// provided number will not be highlighted.
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
    pub fn new(
        ss: &'a SyntaxSet,
        sr: &SyntaxReference,
        code: &'a str,
        max_line_len: Option<usize>,
        style: ClassStyle,
    ) -> Self {
        ClassedTableGenerator {
            code,
            syntax_set: ss,
            parse_state: ParseState::new(sr),
            stack: ScopeStack::new(),
            html: String::with_capacity(code.len() * 8), // size is a best guess
            style,
            max_line_len,
        }
    }

    // generate takes ownership of self so that it can't be re-used
    pub fn generate(mut self) -> String {
        open_table(&mut self.html);

        for (i, line) in LinesWithEndings::from(self.code).enumerate() {
            open_row(&mut self.html, i);
            if self.max_line_len.map_or(false, |n| line.len() > n) {
                self.html.push_str(line.strip_suffix("\n").unwrap_or(line));
            } else {
                self.write_spans_for_line(&line);
            }
            close_row(&mut self.html);
        }

        close_table(&mut self.html);
        self.html
    }

    // open_current_scopes opens a span for every scope that was still
    // open from the last line
    fn open_current_scopes(&mut self) {
        for scope in self.stack.clone().as_slice() {
            self.open_scope(scope)
        }
    }

    fn close_current_scopes(&mut self) {
        for _ in 0..self.stack.len() {
            self.close_scope()
        }
    }

    fn open_scope(&mut self, scope: &Scope) {
        self.html.push_str("<span class=\"");
        self.write_classes_for_scope(scope);
        self.html.push_str("\">");
    }

    fn close_scope(&mut self) {
        self.html.push_str("</span>");
    }

    fn write_spans_for_line(&mut self, line: &str) {
        // Whenever we highlight a new line, the all scopes that are still open
        // from the last line must be created. Since scope spans can't cross table
        // row boundaries, we need to open and close scope spans that are shared
        // between lines on every line.
        //
        // For example, for a go file, every line should likely start with
        // <span class="hl-source hl-go">
        self.open_current_scopes();
        let parsed_line = self.parse_state.parse_line(line, self.syntax_set);
        self.write_spans_for_tokens(line, parsed_line.as_slice());
        self.close_current_scopes();
    }

    // write_spans_for_tokens creates spans for the list of tokens passed to it.
    // It modifies the stack of the ClassedTableGenerator, adding any scopes
    // that are unclosed at the end of the line.
    //
    // This is modified from highlight::tokens_to_classed_spans
    fn write_spans_for_tokens(&mut self, line: &str, ops: &[(usize, ScopeStackOp)]) {
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
            stack.apply_with_hook(op, |basic_op, _| match basic_op {
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
            });
            self.stack = stack;
        }
        write!(&mut self.html, "{}", Escape(&line[cur_index..line.len()])).unwrap();
    }

    // write_classes_for_scope is modified from highlight::scope_to_classes
    fn write_classes_for_scope(&mut self, scope: &Scope) {
        let repo = SCOPE_REPO.lock().unwrap();
        for i in 0..(scope.len()) {
            let atom = scope.atom_at(i as usize);
            let atom_s = repo.atom_str(atom);
            if i != 0 {
                self.html.push_str(" ")
            }
            if let ClassStyle::SpacedPrefixed { prefix } = self.style {
                self.html.push_str(&prefix)
            }
            self.html.push_str(atom_s);
        }
    }
}

fn open_table(s: &mut String) {
    s.push_str("<table><tbody>");
}

fn close_table(s: &mut String) {
    s.push_str("</tbody></table>");
}

fn open_row(s: &mut String, i: usize) {
    write!(
        s,
        "<tr><td class=\"line\" data-line=\"{}\"/><td class=\"code\">",
        i + 1
        )
        .unwrap();
}

fn close_row(s: &mut String) {
    s.push_str("</td></tr>");
}

