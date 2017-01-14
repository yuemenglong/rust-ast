extern crate syntex_syntax as syntax;
extern crate syntex_errors as errors;
extern crate regex;

mod formatter;
mod visitor;
mod anno;
mod meta;
mod types;

use formatter::Formatter;
use visitor::Visitor;
use meta::*;

use syntax::codemap::CodeMap;
use syntax::parse::{self, ParseSess};
use errors::emitter::ColorConfig;
use errors::Handler;

use std::rc::Rc;

fn create_parse_session() -> ParseSess {
    let codemap = Rc::new(CodeMap::new());
    let tty_handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(codemap.clone()));
    let parse_session = ParseSess::with_span_handler(tty_handler, codemap.clone());
    parse_session
}

static SRC: &'static str = "
struct Name {
    // #[derive(Debug, asdf=\"123\")]
    field: Option<i32>,
    // invalid:Option<RefCell<i64>>,
    #[id(auto)]
    id:i64,
}
";

struct T {
    a: i32,
    b: i32,
}

fn main() {
    let parse_session = create_parse_session();
    let krate =
        parse::parse_crate_from_source_str("stdin".to_string(), SRC.to_string(), &parse_session)
            .unwrap();
    // println!("{:?}", krate.module.items);
    // println!("{:?}", krate.module.items.len());
    let mut visitor = Visitor::new();
    visitor.visit_krate(&krate);
    let formatter = Formatter::new();
    let ret = formatter.format_krate(&krate);
    // println!("{:?}", visitor.meta);
    // println!("{}", ret);
}
