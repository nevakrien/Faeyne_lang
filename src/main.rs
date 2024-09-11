use faeyne_lang::lexer::Lexer;
use faeyne_lang::parser;
use faeyne_lang::ast::StringTable;

#[test]
fn integration_test() {
    let input = r#"
	"hello"
    "world"
    "foo"
    :atom
    "poison
    "bar" 
    "baz"
    "#;

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::StringListParser::new();
    // Feed lexer tokens into parser
    let result = parser.parse(input,&mut table,lexer);

    println!("input '''{}'''", input);
    match result {
        Ok(parsed_strings) => println!("Parsed: {:?}", parsed_strings),
        Err(err) => println!("Error: {:?}", err),
    }
}


fn main() {
    println!("Hello, world!");
}
