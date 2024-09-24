#![cfg(test)]

use crate::runners::{safe_run, safe_run_compare};  // Assuming you have both safe_run and safe_run_compare
use crate::ir::*;
use crate::get_id;
use crate::system::*;

#[test]
fn simple_parse_hello_world_function() {
    safe_run("def main(system) { system(:println)('hello world'); }");
}

#[test]
fn lifetime_ub() {
    let s = "def main(system) { system(:println)('hello world'); }".to_string();
    
    // Leak the string, extract its inner str, and get a mutable raw pointer to it
    let raw_str: *mut str = Box::into_raw(s.into_boxed_str()) as *mut str;

    //run and drop the static ref
    {
        let static_ref_str: &'static str = unsafe { &*raw_str };
        safe_run(static_ref_str);
    }
    

    // Clean up: Convert the raw pointer back into a boxed str and drop it
    unsafe {
        let _ = Box::from_raw(raw_str);
    }
}


#[test]
fn simple_string_arith() {
    safe_run("def main(system) { system(:println)('hello'+' world'); }");
}

#[test]
fn test_string_and_number_addition() {
    let input = r#"
        def main(system) {
            result1 = 'hello' + ' world';
            result2 = 5 + 10;
            result1+' - '+result2
        }
    "#;
    
    safe_run_compare(input, Value::String(GcPointer::new("hello world - 15".to_string())));
}

#[test]
fn test_simple_conditional() {
    let input = r#"
        def main(system) {
            match 5 > 3 {
                true => 'greater',
                false => 'lesser',
            }
        }
    "#;

    safe_run_compare(input, Value::String(GcPointer::new("greater".to_string())));
}

#[test]
fn test_factorial_easy() {
    let input = r#"
        def base_factorial(i) {
            match i {
                0 => 1,
                _ => {a = base_factorial(i-1); return i*a;}
            }
        }

        def main(system) {
            base_factorial(5)
        }
    "#;

    safe_run_compare(input, Value::Int(120));
}

#[test]
fn test_factorial() {
    let input = r#"
        def base_factorial(i) {
            match i {
                0 => 1,
                _ => i * base_factorial(i - 1),
            }
        }

        def main(system) {
            base_factorial(5)
        }
    "#;

    safe_run_compare(input, Value::Int(120));
}

#[test]
fn test_fibonacci() {
    let input = r#"
        def fibonacci(n) {
            match n {
                0 => 0,
                1 => 1,
                _ => fibonacci(n - 1) + fibonacci(n - 2),
            }
        }

        def main(system) {
            fibonacci(6)
        }
    "#;

    safe_run_compare(input, Value::Int(8));
}

#[test]
fn test_boolean_logic() {
    let input = r#"
        def main(system) {
            match 10 > 5 && 3 < 5 {
                true => 42,
                false => 0,
            }
        }
    "#;

    safe_run_compare(input, Value::Int(42));
}

#[test]
fn test_type_checks() {
    let input = r#"
        def main(system) {
            system(:type)("hi")
        }
    "#;

    safe_run_compare(input, Value::Atom(get_id!(":string")));
}


// #[test]
// fn test_atom_str() {
//     let input = r#"
//         def main(system) {
//             system(:to_string)(:hi)
//         }
//     "#;

//     safe_run_compare(input, Value::String(GcPointer::new(":hi".to_string())));
// }