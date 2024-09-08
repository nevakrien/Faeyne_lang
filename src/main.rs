use Faeyne_lang::lexer::Lexer;
use Faeyne_lang::parser;

#[test]
fn integration_test() {
    let input = r#"
    "hello"
    "world"
    "foo"
    "bar"
    "#;

    let lexer = Lexer::new(input);
    let parser = parser::StringListParser::new();
    // Feed lexer tokens into parser
    let result = parser.parse(input,lexer);

    match result {
        Ok(parsed_strings) => println!("Parsed: {:?}", parsed_strings),
        Err(err) => println!("Error: {:?}", err),
    }
}


fn main() {
    println!("Hello, world!");
}
