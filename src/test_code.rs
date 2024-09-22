#![cfg(test)]

use crate::runners::{run_str, clean_str_run};

#[test]
fn simple_parse_hello_world_function() {
    let input = "def main(system) { system(:println)('hello world'); }";
    let junk = run_str(input);
    unsafe{clean_str_run(junk);}
}

#[test]
fn simple_string_arith() {
    let input = "def main(system) { system(:println)('hello'+' world'); }";
    let junk = run_str(input);
    unsafe{clean_str_run(junk);}
}
