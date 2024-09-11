#![cfg(test)]

use crate::lexer::Lexer;
use crate::parser;
use crate::ast::*;

#[test]
fn func_dec_single_arg() {
    let input = "def main(system)";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncSigParser::new();
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
fn func_dec_multiple_args() {
    let input = "def foo(bar, baz, qux)";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncSigParser::new();
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
fn func_dec_no_args() {
    let input = "def noop()";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncSigParser::new();
    let result = parser.parse(input, &mut table, lexer);

    // Assert that parsing was successful
    assert!(result.is_ok(), "Failed to parse function with no arguments");

    let func_dec = result.unwrap();

    // Assert function name and empty argument list
    assert_eq!(table.get_string(func_dec.name).unwrap(), "noop");
    assert_eq!(func_dec.args.len(), 0);
}

#[test]
fn function_calls_and_expressions() {
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

#[test]
fn function_call_no_args() {
    let input = "foo()";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::ValueParser::new();
    let result = parser.parse(input, &mut table, lexer);
    
    assert!(result.is_ok(), "Failed to parse function call with no arguments");
    
    let value = result.unwrap();
    if let Value::FuncCall(FunctionCall { name, args }) = value {
        assert_eq!(table.get_string(name).unwrap(), "foo");
        assert_eq!(args.len(), 0, "Expected no arguments");
    } else {
        panic!("Expected a function call");
    }
}



#[test]
fn func_block_with_statements_and_return() {
    let input = "{ a = b; s(); c }";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncBlockParser::new();
    let result = parser.parse(input, &mut table, lexer);

    // assert!(result.is_ok(), "Failed to parse function block with statements and return");

    let func_block = result.unwrap();

    // Check body length and return value
    assert_eq!(func_block.body.len(), 2);
    match &func_block.body[1] {
        Statment::Call(fc) => {
            assert_eq!("s",table.get_string(fc.name).unwrap());
        },
        _ => unreachable!()
    };
    assert!(matches!(func_block.ret, Some(Value::Variable(_))));
}

#[test]
fn func_block_with_statements_no_return() {
    let input = "{ a = b; c = d; }";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncBlockParser::new();
    let result = parser.parse(input, &mut table, lexer);

    // assert!(result.is_ok(), "Failed to parse function block with statements and no return");

    let func_block = result.unwrap();

    // Check body length and ensure no return
    assert_eq!(func_block.body.len(), 2);
    assert!(func_block.ret.is_none());
}

#[test]
fn func_block_only_return() {
    let input = "{ return x; }";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncBlockParser::new();
    let result = parser.parse(input, &mut table, lexer);

    assert!(result.is_ok(), "Failed to parse function block with only return");

    let func_block = result.unwrap();

    // Check empty body and return value
    assert_eq!(func_block.body.len(), 0);
    assert!(matches!(func_block.ret, Some(Value::Variable(_))));
}

#[test]
fn func_block_empty() {
    let input = "{}";

    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncBlockParser::new();
    let result = parser.parse(input, &mut table, lexer);

    assert!(result.is_ok(), "Failed to parse empty function block");

    let func_block = result.unwrap();

    // Check empty body and no return
    assert_eq!(func_block.body.len(), 0);
    assert!(func_block.ret.is_none());
}
