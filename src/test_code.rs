#![cfg(test)]

use crate::runners::{run_str, clean_str_run};
use crate::ir::*;

#[test]
fn simple_parse_hello_world_function() {
    let input = "def main(system) { system(:println)('hello world'); }";
    let (_ans,junk) = run_str(input);
    unsafe{clean_str_run(junk);}
}

#[test]
fn simple_string_arith() {
    let input = "def main(system) { system(:println)('hello'+' world'); }";
    let  (_ans,junk) = run_str(input);
    unsafe{clean_str_run(junk);}
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

    let (ans, junk) = run_str(input);
    assert_eq!(ans, Value::String(GcPointer::new("hello world - 15".to_string())));

    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
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

    let (ans, junk) = run_str(input);
    assert_eq!(ans, Value::String(GcPointer::new("greater".to_string())));

    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
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

    let (ans, junk) = run_str(input);
    assert_eq!(ans, Value::Int(120));

    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
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

    let (ans, junk) = run_str(input);
    assert_eq!(ans, Value::Int(120));

    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
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

    let (ans, junk) = run_str(input);
    assert_eq!(ans, Value::Int(8));

    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
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

    let (ans, junk) = run_str(input);
    assert_eq!(ans, Value::Int(42));

    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
}
