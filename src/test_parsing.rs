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
fn simple_parse_blocky_function() {
    let input = "def main(system) { x=system(:println)('hello world'); x(w(z)); return z; }";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::FuncDecParser::new();  // Assuming you create this parser
    let result = parser.parse(input, &mut table, lexer);
    
    assert!(result.is_ok(), "Failed to parse function declaration");
    
    let func_dec = result.unwrap();
    
    assert!(func_dec.body.body.len() == 2, "Expected one statement in function body");
    assert!(func_dec.body.ret.is_some(), "Expected return statment");
}
#[test]
fn simple_parse_lammda_function() {
    let input = "def main(system) { f = fn (x,y) -> {x}; fn (x) {} }";
    
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();
    
    let parser = parser::FuncDecParser::new();  // Assuming you create this parser
    let result = parser.parse(input, &mut table, lexer);
    
    // assert!(result.is_ok(), "Failed to parse function declaration");
    
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
    if let Value::FuncCall (FunctionCall {name, args,.. }) = value {
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
    if let Value::FuncCall(FunctionCall { name, args,.. }) = value {
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

#[test]
fn test_arithmetic_as_func_call() {
    let input = "1 + 2 * 3 - 4 / 2";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::ValueParser::new();  // Assuming you have this parser set up
    let result = parser.parse(input, &mut table, lexer);

    assert!(result.is_ok(), "Failed to parse arithmetic expression");

    let parsed_value = result.unwrap();

    // Check the parsed value is a subtraction `FuncCall`
    if let Value::FuncCall(subtract_call) = parsed_value {
        // Handle FValue::Name variant for function names
        if let FValue::BuildIn(BuildIn::Sub) = subtract_call.name {
            // Check the left operand is an addition `FuncCall`
            if let Value::FuncCall(add_call) = &subtract_call.args[0] {
                if let FValue::BuildIn(BuildIn::Add) = add_call.name {
                    assert_eq!(add_call.args[0], Value::Int(Ok(1))); // Check the first argument of addition is 1

                    // Check the right operand of addition is multiplication `FuncCall`
                    if let Value::FuncCall(mul_call) = &add_call.args[1] {
                        if let FValue::BuildIn(BuildIn::Mul) = mul_call.name {
                            assert_eq!(mul_call.args[0], Value::Int(Ok(2))); // Multiplication left operand is 2
                            assert_eq!(mul_call.args[1], Value::Int(Ok(3))); // Multiplication right operand is 3
                        } else {
                            panic!("Expected multiplication function");
                        }
                    } else {
                        panic!("Expected multiplication as the right operand of addition");
                    }
                } else {
                    panic!("Expected addition function");
                }
            } else {
                panic!("Expected addition as the left operand of subtraction");
            }

            // Check the right operand of subtraction is division `FuncCall`
            if let Value::FuncCall(div_call) = &subtract_call.args[1] {
                if let FValue::BuildIn(BuildIn::Div) = div_call.name {
                    assert_eq!(div_call.args[0], Value::Int(Ok(4))); // Division left operand is 4
                    assert_eq!(div_call.args[1], Value::Int(Ok(2))); // Division right operand is 2
                } else {
                    panic!("Expected division function");
                }
            } else {
                panic!("Expected division as the right operand of subtraction");
            }
        } else {
            panic!("Expected subtraction function");
        }
    } else {
        panic!("Expected a function call for arithmetic expression");
    }
}

#[test]
fn test_basic_piping() {
    let input = "a() |> b() |> c()";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncCallParser::new();  // Assuming you have this parser set up for function calls
    let result = parser.parse(input, &mut table, lexer);

    let pipe_call = result.unwrap();

    // Check the last function called in the chain is `c()`
    if let FValue::Name(c_name) = pipe_call.name {
        assert_eq!(table.get_string(c_name).unwrap(), "c");

        // Check the argument to `c()` is a `FuncCall` for `b()`
        if let Value::FuncCall(b_call) = &pipe_call.args[0] {
            if let FValue::Name(b_name) = b_call.name {
                assert_eq!(table.get_string(b_name).unwrap(), "b");

                // Check the argument to `b()` is a `FuncCall` for `a()`
                if let Value::FuncCall(a_call) = &b_call.args[0] {
                    if let FValue::Name(a_name) = a_call.name {
                        assert_eq!(table.get_string(a_name).unwrap(), "a");
                    } else {
                        panic!("Expected function name 'a'");
                    }
                } else {
                    panic!("Expected 'a()' as the argument to 'b()'");
                }
            } else {
                panic!("Expected function name 'b'");
            }
        } else {
            panic!("Expected 'b()' as the argument to 'c()'");
        }
    } else {
        panic!("Expected function name 'c'");
    }
}

#[test]
fn test_piping_with_third_order_nesting() {
    let input = "a(1 + 2) |> b(y, n(m(x))) |> c(3 * 4, z)";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncCallParser::new();
    let pipe_call = parser.parse(input, &mut table, lexer).unwrap();  // Unwrap to get full error report on failure

    // Check the last function called in the chain is `c()`
    if let FValue::Name(c_name) = pipe_call.name {
        assert_eq!(table.get_string(c_name).unwrap(), "c");

        // Check the arguments to `c()` (total 3 arguments, reverse order)
        assert_eq!(pipe_call.args.len(), 3);

        // Third argument is `z` (as a variable)
        assert_eq!(pipe_call.args[2], Value::Variable(table.get_id("z")));

        // Second argument is `3 * 4`
        if let Value::FuncCall(mul_call) = &pipe_call.args[1] {
            if let FValue::BuildIn(BuildIn::Mul) = mul_call.name {
                assert_eq!(mul_call.args[0], Value::Int(Ok(3)));
                assert_eq!(mul_call.args[1], Value::Int(Ok(4)));
            } else {
                panic!("Expected multiplication function");
            }
        } else {
            panic!("Expected multiplication as the second argument of `c()`");
        }

        // First argument is `b(a(1 + 2), y, n(m(x)))` (as a function call)
        if let Value::FuncCall(b_call) = &pipe_call.args[0] {
            if let FValue::Name(b_name) = b_call.name {
                assert_eq!(table.get_string(b_name).unwrap(), "b");
            } else {
                panic!("Expected function name 'b'");
            }

            // Check the arguments to `b()` (total 3 arguments)
            assert_eq!(b_call.args.len(), 3);

            // First argument to `b()` is `a(1 + 2)` (as a function call)
            if let Value::FuncCall(a_call) = &b_call.args[0] {
                if let FValue::Name(a_name) = a_call.name {
                    assert_eq!(table.get_string(a_name).unwrap(), "a");
                } else {
                    panic!("Expected function name 'a'");
                }

                // Check `a()` has one argument: `1 + 2`
                if let Value::FuncCall(add_call) = &a_call.args[0] {
                    if let FValue::BuildIn(BuildIn::Add) = add_call.name {
                        assert_eq!(add_call.args[0], Value::Int(Ok(1)));
                        assert_eq!(add_call.args[1], Value::Int(Ok(2)));
                    } else {
                        panic!("Expected addition function");
                    }
                } else {
                    panic!("Expected `1 + 2` as the argument to `a()`");
                }
            } else {
                panic!("Expected `a(1 + 2)` as the first argument of `b()`");
            }

            // Second argument to `b()` is `y` (as a variable)
            assert_eq!(b_call.args[1], Value::Variable(table.get_id("y")));

            // Third argument to `b()` is `n(m(x))`
            if let Value::FuncCall(n_call) = &b_call.args[2] {
                if let FValue::Name(n_name) = n_call.name {
                    assert_eq!(table.get_string(n_name).unwrap(), "n");
                } else {
                    panic!("Expected function name 'n'");
                }

                // Check `n()` has one argument: `m(x)`
                if let Value::FuncCall(m_call) = &n_call.args[0] {
                    if let FValue::Name(m_name) = m_call.name {
                        assert_eq!(table.get_string(m_name).unwrap(), "m");
                    } else {
                        panic!("Expected function name 'm'");
                    }

                    // Check `m()` has one argument: `x`
                    assert_eq!(m_call.args[0], Value::Variable(table.get_id("x")));
                } else {
                    panic!("Expected `m(x)` as the argument to `n()`");
                }
            } else {
                panic!("Expected `n(m(x))` as the third argument to `b()`");
            }
        } else {
            panic!("Expected `b()` as the first argument of `c()`");
        }
    } else {
        panic!("Expected function name 'c'");
    }
}
