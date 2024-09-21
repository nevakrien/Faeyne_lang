pub mod lexer;
pub mod ast;
pub mod ir;
pub mod basic_ops;
pub mod reporting;

// pub mod translate;

mod test_parsing;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);