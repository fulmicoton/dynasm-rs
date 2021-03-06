#![feature(plugin_registrar, rustc_private)]
#![feature(const_fn)]

extern crate syntax;
extern crate rustc_plugin;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

use rustc_plugin::registry::Registry;
use syntax::ext::base::{SyntaxExtension, ExtCtxt, MacResult, DummyResult};
use syntax::fold::Folder;
use syntax::codemap::Span;
use syntax::ast;
use syntax::util::small_vector::SmallVector;
use syntax::parse::token::intern;
use syntax::tokenstream::TokenTree;

mod parser;
mod compiler;
mod x64data;
mod serialize;
mod debug;


#[plugin_registrar]
pub fn registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        intern("dynasm"), 
        SyntaxExtension::NormalTT(
            Box::new(main),
            None,
            false
        )
    );
}

fn main<'cx>(ecx: &'cx mut ExtCtxt, span: Span, token_tree: &[TokenTree])
-> Box<MacResult + 'cx> {
    // expand all macros in our token tree first. This enables the use of rust macros
    // within dynasm
    let token_tree = ecx.expander().fold_tts(token_tree);

    let mut parser = ecx.new_parser_from_tts(&token_tree);

    // construct an ast of assembly nodes
    let (name, ast) = match parser::parse(ecx, &mut parser) {
        Ok(ast) => ast,
        Err(mut e) => {e.emit(); return DummyResult::any(span)}
    };

    // println!("{:?}", ast);

    let stmts = if let Ok(stmts) = compiler::compile(ecx, ast) {
        stmts
    } else {
        return DummyResult::any(span)
    };

    let stmts = serialize::serialize(ecx, name, stmts);

    Box::new(DynAsm {stmts: SmallVector::many(stmts)})
}

struct DynAsm {
    stmts: SmallVector<ast::Stmt>
}

impl MacResult for DynAsm {
    fn make_stmts(self: Box<Self>) -> Option<SmallVector<ast::Stmt>> {
        Some(self.stmts)
    }
}
