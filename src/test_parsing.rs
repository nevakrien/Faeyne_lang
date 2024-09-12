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
    
    assert!(func_dec.body.body.len() == 2, "Expected two statement in function body");
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
fn test_logical_as_func_call() {
    let input = "1 == 1 && 2 != 3 || 4 < 5";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::ValueParser::new();  // Assuming you have this parser set up
    let result = parser.parse(input, &mut table, lexer);

    //assert!(result.is_ok(), "Failed to parse logical expression");

    let parsed_value = result.unwrap();
    //println!("\n\nparsed:\n\n{:?}\n\n",parsed_value );

    // Check the parsed value is an OR `FuncCall`
    if let Value::FuncCall(or_call) = parsed_value {
        if let FValue::BuildIn(BuildIn::DoubleOr) = or_call.name {
            // Check the left operand is an AND `FuncCall`
            if let Value::FuncCall(and_call) = &or_call.args[0] {
                if let FValue::BuildIn(BuildIn::DoubleAnd) = and_call.name {
                    // Check the left operand of AND is an equality `FuncCall`
                    if let Value::FuncCall(eq_call) = &and_call.args[0] {
                        if let FValue::BuildIn(BuildIn::Equal) = eq_call.name {
                            assert_eq!(eq_call.args[0], Value::Int(Ok(1))); // Equality left operand is 1
                            assert_eq!(eq_call.args[1], Value::Int(Ok(1))); // Equality right operand is 1
                        } else {
                            panic!("Expected equality function");
                        }
                    } else {
                        panic!("Expected equality as the left operand of AND");
                    }

                    // Check the right operand of AND is a not-equal `FuncCall`
                    if let Value::FuncCall(neq_call) = &and_call.args[1] {
                        if let FValue::BuildIn(BuildIn::NotEqual) = neq_call.name {
                            assert_eq!(neq_call.args[0], Value::Int(Ok(2))); // NotEqual left operand is 2
                            assert_eq!(neq_call.args[1], Value::Int(Ok(3))); // NotEqual right operand is 3
                        } else {
                            panic!("Expected not-equal function");
                        }
                    } else {
                        panic!("Expected not-equal as the right operand of AND");
                    }
                } else {
                    panic!("Expected AND function");
                }
            } else {
                panic!("Expected AND as the left operand of OR");
            }

            // Check the right operand of OR is a smaller-than `FuncCall`
            if let Value::FuncCall(smaller_call) = &or_call.args[1] {
                if let FValue::BuildIn(BuildIn::Smaller) = smaller_call.name {
                    assert_eq!(smaller_call.args[0], Value::Int(Ok(4))); // Smaller left operand is 4
                    assert_eq!(smaller_call.args[1], Value::Int(Ok(5))); // Smaller right operand is 5
                } else {
                    panic!("Expected smaller-than function");
                }
            } else {
                panic!("Expected smaller-than as the right operand of OR");
            }
        } else {
            panic!("Expected OR function");
        }
    } else {
        panic!("Expected a function call for logical expression");
    }
}

#[test]
fn test_comparison_with_logical_in_parentheses() {
    let input = "x > (a && b)";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::ValueParser::new();  // Assuming you have this parser set up
    let result = parser.parse(input, &mut table, lexer);

    // Uncomment when debugging to see the parsed result
    // println!("\n\nparsed:\n\n{:?}\n\n", parsed_value);

    //assert!(result.is_ok(), "Failed to parse expression with comparison and logical operators in parentheses");

    let parsed_value = result.unwrap();

    // Check the parsed value is a `FuncCall` for `>`
    if let Value::FuncCall(gt_call) = parsed_value {
        if let FValue::BuildIn(BuildIn::Bigger) = gt_call.name {
            // Fetch the index for variable "x" from the string table
            let x_var_index = table.get_id("x");

            // Check that the left operand is the variable `x` (index in string table)
            assert_eq!(gt_call.args[0], Value::Variable(x_var_index));

            // Check that the right operand is the `&&` logical operation inside parentheses
            if let Value::FuncCall(and_call) = &gt_call.args[1] {
                if let FValue::BuildIn(BuildIn::DoubleAnd) = and_call.name {
                    // Fetch the index for variable "a" from the string table
                    let a_var_index = table.get_id("a");

                    // Check that the left operand of `&&` is the variable `a` (index in string table)
                    assert_eq!(and_call.args[0], Value::Variable(a_var_index));

                    // Fetch the index for variable "b" from the string table
                    let b_var_index = table.get_id("b");

                    // Check that the right operand of `&&` is the variable `b` (index in string table)
                    assert_eq!(and_call.args[1], Value::Variable(b_var_index));
                } else {
                    panic!("Expected logical AND function inside parentheses");
                }
            } else {
                panic!("Expected a function call (AND) inside parentheses as the right operand of the comparison");
            }
        } else {
            panic!("Expected a greater-than comparison (>) function");
        }
    } else {
        panic!("Expected a function call for the comparison expression");
    }
}

#[test]
fn test_complex_expression_with_nested_parentheses() {
    let input = "(x + y) * (a && (b > c || d))";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::ValueParser::new();  // Assuming you have this parser set up
    let result = parser.parse(input, &mut table, lexer);

    // Unwrap the result to get a better error message if parsing fails
    let parsed_value = result.unwrap();

    // Check the parsed value is a `FuncCall` for `*`
    if let Value::FuncCall(mul_call) = parsed_value {
        if let FValue::BuildIn(BuildIn::Mul) = mul_call.name {
            // Fetch the index for variable "x" and "y" from the string table
            let x_var_index = table.get_id("x");
            let y_var_index = table.get_id("y");

            // Check that the left operand is the `(x + y)` operation
            if let Value::FuncCall(add_call) = &mul_call.args[0] {
                if let FValue::BuildIn(BuildIn::Add) = add_call.name {
                    // Check that the left operand of `+` is `x`
                    assert_eq!(add_call.args[0], Value::Variable(x_var_index));

                    // Check that the right operand of `+` is `y`
                    assert_eq!(add_call.args[1], Value::Variable(y_var_index));
                } else {
                    panic!("Expected addition (+) function for `(x + y)`");
                }
            } else {
                panic!("Expected a function call (addition) as the left operand of the multiplication");
            }

            // Check that the right operand is the logical operation `(a && (b > c || d))`
            if let Value::FuncCall(and_call) = &mul_call.args[1] {
                if let FValue::BuildIn(BuildIn::DoubleAnd) = and_call.name {
                    let a_var_index = table.get_id("a");

                    // Check that the left operand of `&&` is `a`
                    assert_eq!(and_call.args[0], Value::Variable(a_var_index));

                    // Check that the right operand of `&&` is `(b > c || d)`
                    if let Value::FuncCall(or_call) = &and_call.args[1] {
                        if let FValue::BuildIn(BuildIn::DoubleOr) = or_call.name {
                            let b_var_index = table.get_id("b");
                            let c_var_index = table.get_id("c");
                            let d_var_index = table.get_id("d");

                            // Check that the left operand of `||` is `(b > c)`
                            if let Value::FuncCall(gt_call) = &or_call.args[0] {
                                if let FValue::BuildIn(BuildIn::Bigger) = gt_call.name {
                                    assert_eq!(gt_call.args[0], Value::Variable(b_var_index));
                                    assert_eq!(gt_call.args[1], Value::Variable(c_var_index));
                                } else {
                                    panic!("Expected greater-than (>) function for `(b > c)`");
                                }
                            } else {
                                panic!("Expected a function call (>) as the left operand of `||`");
                            }

                            // Check that the right operand of `||` is `d`
                            assert_eq!(or_call.args[1], Value::Variable(d_var_index));
                        } else {
                            panic!("Expected logical OR (||) function for `(b > c || d)`");
                        }
                    } else {
                        panic!("Expected a function call (OR) as the right operand of the AND operation");
                    }
                } else {
                    panic!("Expected logical AND (&&) function for `(a && (b > c || d))`");
                }
            } else {
                panic!("Expected a function call (AND) as the right operand of the multiplication");
            }
        } else {
            panic!("Expected a multiplication (*) function for the entire expression");
        }
    } else {
        panic!("Expected a function call for the multiplication expression");
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

#[test]
fn test_piping_with_function_call_as_function() {
    let input = "a() |> c()()";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncCallParser::new();  // Assuming you have this parser set up for function calls
    let result = parser.parse(input, &mut table, lexer);

    let pipe_call = result.unwrap();

    // Check that the outermost call is a `FuncCall` representing the invocation of `c()()`
    if let FValue::FuncCall(c_call) = pipe_call.name {
        // Verify the name of the inner function call is `c`
        if let FValue::Name(c_name) = c_call.name {
            assert_eq!(table.get_string(c_name).unwrap(), "c");

            // Check that `c()` has no arguments in its first invocation
            assert_eq!(c_call.args.len(), 0);

            // Check the argument to the result of `c()` is a function call for `a()`
            if let Value::FuncCall(a_call) = &pipe_call.args[0] {
                if let FValue::Name(a_name) = a_call.name {
                    assert_eq!(table.get_string(a_name).unwrap(), "a");

                    // Verify `a()` has no arguments
                    assert_eq!(a_call.args.len(), 0);
                } else {
                    panic!("Expected function name 'a'");
                }
            } else {
                panic!("Expected 'a()' as the argument to `c()()`");
            }
        } else {
            panic!("Expected function name 'c'");
        }
    } else {
        panic!("Expected outer function call to be a result of `c()()`");
    }
}

#[test]
fn test_complex_piping_with_double_function_call() {
    let input = "a() |> b(x) |> c(d(e()), f())()";
    let lexer = Lexer::new(input);
    let mut table = StringTable::new();

    let parser = parser::FuncCallParser::new();
    let result = parser.parse(input, &mut table, lexer);
    let pipe_call = result.unwrap();

    // Check that the outermost call is to `c(d(e()), f())(...)`
    if let FValue::FuncCall(c_call) = pipe_call.name {
        // Verify the name of the function is `c`
        if let FValue::Name(c_name) = c_call.name {
            assert_eq!(table.get_string(c_name).unwrap(), "c");

            // Check that `c()` has two arguments: `d(e())` and `f()`
            assert_eq!(c_call.args.len(), 2);

            // First argument is `d(e())`
            if let Value::FuncCall(d_call) = &c_call.args[0] {
                if let FValue::Name(d_name) = d_call.name {
                    assert_eq!(table.get_string(d_name).unwrap(), "d");

                    // Check `d()` has one argument: `e()`
                    if let Value::FuncCall(e_call) = &d_call.args[0] {
                        if let FValue::Name(e_name) = e_call.name {
                            assert_eq!(table.get_string(e_name).unwrap(), "e");
                        } else {
                            panic!("Expected function name 'e'");
                        }
                    } else {
                        panic!("Expected `e()` as the argument to `d()`");
                    }
                } else {
                    panic!("Expected function name 'd'");
                }
            } else {
                panic!("Expected `d(e())` as the first argument of `c()`");
            }

            // Second argument is `f()`
            if let Value::FuncCall(f_call) = &c_call.args[1] {
                if let FValue::Name(f_name) = f_call.name {
                    assert_eq!(table.get_string(f_name).unwrap(), "f");

                    // Verify `f()` has no arguments
                    assert_eq!(f_call.args.len(), 0);
                } else {
                    panic!("Expected function name 'f'");
                }
            } else {
                panic!("Expected `f()` as the second argument of `c()`");
            }
        } else {
            panic!("Expected function name 'c'");
        }

        // Check the argument passed to `c()` is `b(a(), x)`
        if let Value::FuncCall(b_call) = &pipe_call.args[0] {
            if let FValue::Name(b_name) = b_call.name {
                assert_eq!(table.get_string(b_name).unwrap(), "b");

                // First argument to `b()` is `a()`
                if let Value::FuncCall(a_call) = &b_call.args[0] {
                    if let FValue::Name(a_name) = a_call.name {
                        assert_eq!(table.get_string(a_name).unwrap(), "a");

                        // Verify `a()` has no arguments
                        assert_eq!(a_call.args.len(), 0);
                    } else {
                        panic!("Expected function name 'a'");
                    }
                } else {
                    panic!("Expected `a()` as the first argument to `b()`");
                }

                // Second argument to `b()` is `x`
                assert_eq!(b_call.args[1], Value::Variable(table.get_id("x")));
            } else {
                panic!("Expected function name 'b'");
            }
        } else {
            panic!("Expected `b(a(), x)` as the argument to `c()`");
        }
    } else {
        panic!("Expected outer function call to be `c(d(e()), f())(b(a(), x))`");
    }
}

#[test]
fn parse_pipe_hello_world_function() {
    let input = "def main(system) { 'hello world'|> system(:println)(); }";
    
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
fn parse_pipe_nil() {
    let input = "def main(system) { nil|> system(nil)(); }";
    
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

            assert_eq!(outer_call.args[0],Value::Nil);
        } else {
            panic!("Expected system(:println) call as the outer function");
        }

        // Validate the argument to `system(:println)` is `"hello world"`
        assert_eq!(func_call.args.len(), 1, "Expected one argument to system(:println)");
        assert_eq!(func_call.args[0],Value::Nil);
        
    } else {
        panic!("Expected a function call in the body of main");
    }
}
