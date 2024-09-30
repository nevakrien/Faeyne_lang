pub mod lexer;
pub mod ast;
pub mod id;
mod test_parsing;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub parser);