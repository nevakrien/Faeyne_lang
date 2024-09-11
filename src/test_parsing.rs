#![cfg(test)]

use crate::lexer::Lexer;
use crate::parser;
use crate::ast::*;

#[test]
fn simple_parse_hello_world_function() {
    let input = "def main(system) { system(:println)('hello world'); }";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::FuncDecParser::new();  // Assuming you create this parser
    let result = parser.parse(input, &mut table, lexer);
    
    assert!(result.is_ok(), "Failed to parse function declaration");
    
    let func_dec = result.unwrap();
    
    assert_eq!(table.get_string(func_dec.sig.name), Some("main"));
    assert!(func_dec.body.body.len() == 1, "Expected one statement in function body");
}

#[test]
fn parse_hello_world_function() {
    let input = "def main(system) { system(:println)('hello world'); }";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::FuncDecParser::new();  
    let result = parser.parse(input, &mut table, lexer);
    
    assert!(result.is_ok(), "Failed to parse function declaration");
    
    let func_dec = result.unwrap();
    
    // Validate the function signature (name "main" and one argument "system")
    assert_eq!(table.get_string(func_dec.sig.name), Some("main"));
    assert_eq!(func_dec.sig.args.len(), 1, "Expected one argument in function signature");
    assert_eq!(table.get_string(func_dec.sig.args[0]), Some("system"));

    // Validate the function body has one statement (a function call)
    assert_eq!(func_dec.body.body.len(), 1, "Expected one statement in function body");
    
    // Unwrap the single statement and ensure it's a function call
    if let Statment::Call(func_call) = &func_dec.body.body[0] {
        // Check the outer function call is `system(:println)`
        if let FValue::FuncCall(outer_call) = &func_call.name {
            // Ensure the outer function is `system`
            if let FValue::Name(system_name) = outer_call.name {
                assert_eq!(table.get_string(system_name), Some("system"));
            } else {
                panic!("Expected 'system' as the outer function name");
            }

            // The first argument of the outer call is `:println` (an atom)
            if let Value::Atom(atom_id) = &outer_call.args[0] {
                assert_eq!(table.get_string(*atom_id), Some(":println"));
            } else {
                panic!("Expected :println as the argument to system");
            }
        } else {
            panic!("Expected system(:println) call as the outer function");
        }

        // Validate the argument to `system(:println)` is `"hello world"`
        assert_eq!(func_call.args.len(), 1, "Expected one argument to system(:println)");
        if let Value::String(str_id) = &func_call.args[0] {
            assert_eq!(table.get_string(*str_id), Some("'hello world'"));
        } else {
            panic!("Expected 'hello world' as the argument to system(:println)");
        }
    } else {
        panic!("Expected a function call in the body of main");
    }
}



#[test]
fn func_sig_single_arg() {
    let input = "main(system)";

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
fn func_sig_multiple_args() {
    let input = "foo(bar, baz, qux)";

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
fn func_sig_no_args() {
    let input = "noop()";

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
        let name = match name {
            FValue::Name(n) => n,
            _ => unreachable!()
        };
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
fn nested_calls() {
    let input = "system(:println)(format('hello world'))";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::ValueParser::new();
    let result = parser.parse(input, &mut table, lexer);
    
    assert!(result.is_ok(), "Failed to parse function call with mixed expressions");
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
        let name = match name {
            FValue::Name(n) => n,
            _ => unreachable!()
        };
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
            let name = match fc.name {
                FValue::Name(n) => n,
                _ => unreachable!()
            };
            assert_eq!("s",table.get_string(name).unwrap());
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
