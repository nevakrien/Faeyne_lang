#![cfg(test)]


use crate::reporting::sig_error;
use crate::reporting::report_err_list;
use crate::translate::compile_source_to_code;
use std::sync::Arc;
use crate::value::Value;

#[test]
fn end_to_end_empty_function() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = "
        def main(a, b, c) {
            # This function does nothing and returns immediately
        }
    ";

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Setup the initial arguments for the "main" function (arbitrary values)
    let args = vec![
        Value::Nil,        // c = Nil (or None equivalent)
        Value::Int(1),     // a = 1
        Value::Bool(true), // b = true
    ];

    // Step 4: Run the translated code and call the "main" function with the arguments
    code.run("main", args).unwrap();
}

#[test]
fn return_true() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = "
        def main() {
            true
        }
    ";

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Bool(true)).unwrap());
}


#[test]
fn atom_passing() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = "
        def main() {
            :my_atom
        }
    ";

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    let atom_id = code.table.try_write().unwrap().get_id(":my_atom");
    let wrong_atom_id = code.table.try_write().unwrap().get_id(":wrong_atom");

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Atom(atom_id)).unwrap());
    assert!(!code.run_compare("main", vec![],Value::Atom(wrong_atom_id)).unwrap());
}

#[test]
fn string_escaping() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            "string\n"
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    let s = Arc::new("string\n".to_string());

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::String(s)).unwrap());
}

#[test]
fn return_self() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            self
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);


    // Step 3: Run the translated code and call the "main" function with the arguments
    code.run("main", vec![]).unwrap();
}

#[test]
fn test_1_plus_1() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            1+1
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Int(2)).unwrap());
}

#[test]
fn add_sub() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            1+1-2
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Int(0)).unwrap());
}

#[test]
fn parens() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            (1+1) < (2+3) || false
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Bool(true)).unwrap());
}

#[test]
fn add_err() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            1+:ok
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);
    let _error = sig_error();
    
    
    let res = code.run("main", vec![]);
    match res {
        Ok(()) => panic!("should have type errored"),
        Err(e) => {
            assert!(matches!(e,ref _error));//weird this works
            report_err_list(&e,source_code,&code.table.try_read().unwrap())
        }
    }

}

#[test]
fn match_err() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            match 2 {
                1 => 0,
            }
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);
    let _error = sig_error();
    
    
    let res = code.run("main", vec![]);
    match res {
        Ok(()) => panic!("should have type errored"),
        Err(e) => {
            assert!(matches!(e,ref _error));//weird this works
            report_err_list(&e,source_code,&code.table.try_read().unwrap())
        }
    }

}

#[test]
fn test_match() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            match 2 {
                :ok => 2,
                2 => true,
                _ => 0,
            }
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Bool(true)).unwrap());
}

#[test]
fn match_jumps() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            match 2 {
                :ok => 2,
                2 => true,
                _ => 0,
            };

            match :five {
                :ok => 2,
                2 => true,
                _ => false,
            }
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Bool(false)).unwrap());
}

#[test]
fn assign() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            a=match 2 {
                :ok => 2,
                2 => true,
                _ => 0,
            };

            a
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Bool(true)).unwrap());
}

#[test]
fn arg_reading() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main(a,b) {
            b
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![Value::Bool(false),Value::Bool(true)],Value::Bool(true)).unwrap());
}

#[test]
fn match_scope() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = r#"
        def main() {
            a=match 2 {
                2 => {
                    a=1;
                    a+1
                },
                _ => 0,
            };

            a
        }
    "#;

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],Value::Int(2)).unwrap());
}

#[test]
fn factorial_self() {
    let source_code = r#"
        def factorial(n) {
            match n {
                0 => 1,
                _ => (n)*self(n-1)
            }
        }
    "#;
    let code = compile_source_to_code(source_code);

    println!("{:?}",code.funcs[0].code);

    assert!(code.run_compare("factorial", vec![Value::Int(2)],Value::Int(2)).unwrap());
    assert!(code.run_compare("factorial", vec![Value::Int(1)],Value::Int(1)).unwrap());
    assert!(code.run_compare("factorial", vec![Value::Int(0)],Value::Int(1)).unwrap());

    assert!(code.run_compare("factorial", vec![Value::Int(4)],Value::Int(24)).unwrap());

}

#[test]
fn factorial() {
    let source_code = r#"
        def factorial(n) {
            match n {
                0 => 1,
                _ => (n)*factorial(n-1)
            }
        }
    "#;
    let code = compile_source_to_code(source_code);

    println!("{:?}",code.funcs[0].code);

    assert!(code.run_compare("factorial", vec![Value::Int(2)],Value::Int(2)).unwrap());
    assert!(code.run_compare("factorial", vec![Value::Int(1)],Value::Int(1)).unwrap());
    assert!(code.run_compare("factorial", vec![Value::Int(0)],Value::Int(1)).unwrap());

    assert!(code.run_compare("factorial", vec![Value::Int(4)],Value::Int(24)).unwrap());

}

#[test]
fn factorial_effishent() {
    let source_code = r#"
        def _factorial(ag,n) {
            match n {
                0 => ag,
                _ => {
                    ag = ag*n;
                    self(ag,n-1)
                }
            }
        }

        def factorial(n) {
            1 |> _factorial(n)
        }
    "#;
    let code = compile_source_to_code(source_code);

    println!("{:?}",code.funcs[0].code);
    // panic!("testing code");

    assert!(code.run_compare("factorial", vec![Value::Int(2)],Value::Int(2)).unwrap());
    assert!(code.run_compare("factorial", vec![Value::Int(1)],Value::Int(1)).unwrap());
    assert!(code.run_compare("factorial", vec![Value::Int(0)],Value::Int(1)).unwrap());

    assert!(code.run_compare("factorial", vec![Value::Int(4)],Value::Int(24)).unwrap());

}