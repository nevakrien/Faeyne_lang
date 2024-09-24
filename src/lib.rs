pub mod lexer;
pub mod ast;
pub mod ir;
pub mod basic_ops;
pub mod reporting;

pub mod translate;
pub mod system;
pub mod runners;

mod test_parsing;
mod test_code;



use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);