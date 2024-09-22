#![cfg(test)]

use crate::runners::{safe_run, safe_run_compare};  // Assuming you have both safe_run and safe_run_compare
use crate::ir::*;

#[test]
fn simple_parse_hello_world_function() {
    safe_run("def main(system) { system(:println)('hello world'); }");
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
