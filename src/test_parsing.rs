#![cfg(test)]

use crate::lexer::Lexer;
use crate::parser;
use crate::ast::{StringTable,Value,FunctionCall};

#[test]
fn test_func_dec_single_arg() {
    let input = "def main(system)";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncDecParser::new();
    let result = parser.parse(input, &mut table, lexer);

    // Assert that parsing was successful
    assert!(result.is_ok(), "Failed to parse function with single argument");

    let func_dec = result.unwrap();

    // Assert function name and argument
    assert_eq!(table.get_string(func_dec.name).unwrap(), "main");
    assert_eq!(func_dec.args.len(), 1);
    assert_eq!(table.get_string(func_dec.args[0]).unwrap(), "system");
}

#[test]
fn test_func_dec_multiple_args() {
    let input = "def foo(bar, baz, qux)";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncDecParser::new();
    let result = parser.parse(input, &mut table, lexer);

    // Assert that parsing was successful
    assert!(result.is_ok(), "Failed to parse function with multiple arguments");

    let func_dec = result.unwrap();

    // Assert function name and argument list
    assert_eq!(table.get_string(func_dec.name).unwrap(), "foo");
    assert_eq!(func_dec.args.len(), 3);
    assert_eq!(table.get_string(func_dec.args[0]).unwrap(), "bar");
    assert_eq!(table.get_string(func_dec.args[1]).unwrap(), "baz");
    assert_eq!(table.get_string(func_dec.args[2]).unwrap(), "qux");
}

#[test]
fn test_func_dec_no_args() {
    let input = "def noop()";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncDecParser::new();
    let result = parser.parse(input, &mut table, lexer);

    // Assert that parsing was successful
    assert!(result.is_ok(), "Failed to parse function with no arguments");

    let func_dec = result.unwrap();

    // Assert function name and empty argument list
    assert_eq!(table.get_string(func_dec.name).unwrap(), "noop");
    assert_eq!(func_dec.args.len(), 0);
}

#[test]
fn test_function_calls_and_expressions() {
    let input = "foo(1, 2.5, x)";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::ValueParser::new();
    let result = parser.parse(input, &mut table, lexer);
    
    assert!(result.is_ok(), "Failed to parse function call with mixed expressions");
    
    let value = result.unwrap();
    if let Value::FuncCall (FunctionCall {name, args }) = value {
        assert_eq!(table.get_string(name).unwrap(), "foo");
        assert_eq!(args.len(), 3);
        
        match args[0] {
            Value::Int(Ok(1)) => (),
            _ => panic!("Expected first argument to be Int(1)"),
        }
        match args[1] {
            Value::Float(2.5) => (),
            _ => panic!("Expected second argument to be Float(2.5)"),
        }
        match args[2] {
            Value::Variable(id) => assert_eq!(table.get_string(id).unwrap(), "x"),
            _ => panic!("Expected third argument to be Variable 'x'"),
        }
    } else {
        panic!("Expected a function call");
    }
}