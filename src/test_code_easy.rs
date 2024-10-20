#![cfg(test)]


use crate::translate::compile_source_to_code;
use std::sync::Arc;
use crate::value::Value;

#[test]
fn test_atom_passing() {
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
fn test_string_escaping() {
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
fn test_return_self() {
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