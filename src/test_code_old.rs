#![cfg(test)]

use crate::runners::{safe_run, safe_run_compare};  // Assuming you have both safe_run and safe_run_compare
use crate::ir::*;
use ast::get_id;
use ast::id::*;

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

#[test]
fn test_string_rev() {
    let input = r#"
        def rev(id,ag,source) {
            match id>=0{
                true => {
                    ag=ag+source(id);
                    rev(id-1,ag,source)
                },
                false => ag,
            }
        }

        def reverse_string(s) {
            #type check
            match ''+s == s {
                true => rev(s(:len) -1,'',s),
                false => :err
            }
        }

        def main(system) {
            reverse_string('1234567')
        }
    "#;

    safe_run_compare(input, Value::String(GcPointer::new("7654321".to_string())));
}




#[test]
fn test_atom_str() {
    let input = r#"
        def main(system) {
            system(:to_string)(:hi)
        }
    "#;

    safe_run_compare(input, Value::String(GcPointer::new(":hi".to_string())));
}

#[test]
fn test_recursive_lambda_string_accumulation() {
    let input = r#"
        def main(system) {
            f = fn(x, acc) {
                match x {
                    0 => acc,
                    _ => { acc + ''+x + self(x - 1, acc) }
                }
            };
            result = f(5, "");
            system(:println)(result);
            result
        }
    "#;

    safe_run_compare(input, Value::String(GcPointer::new("54321".to_string())));
}

#[test]
fn test_lambda_returning_itself() {
    let input = r#"
        def main(system) {
            f = fn() {
                self
            };
            result = f();
            result == f  # Check if the returned function is equal to the original lambda
        }
    "#;

    safe_run_compare(input, Value::Bool(true));
}

#[test]
fn test_mutual_recursive_lambdas() {
    let s = r#"
        
        
        def main(system) {
            a = fn(x) {
                match x {
                    0 => self,
                    _ => self(x - 1)
                }
            };
            
            b = fn(x) {
                match x {
                    0 => 3,
                    _ => fn(x) {self(x - 1)|>a()}
                }
            };

            a == b(10)
        }
    "#.to_string();
    let raw_str: *mut str = Box::into_raw(s.into_boxed_str()) as *mut str;

    // Run the code with a static reference and test for UB
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
fn recursive_lambda_complex_ownership_ub() {
    let s = r#"
        def main(system) {
            f = fn(x, acc) {
                match x {
                    0 => acc,
                    _ => { acc + ' ' + self(x - 1, acc) }
                }
            };
            result = f(5, 'start');
            system(:println)(result);
        }
    "#.to_string();
    
    // Leak the string, extract its inner str, and get a mutable raw pointer to it
    let raw_str: *mut str = Box::into_raw(s.into_boxed_str()) as *mut str;

    // Run the code with a static reference and test for UB
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
fn closure_capture_test() {
    //note the code may not be correct but it sure as hell stretches the type system
    let input = r#"
        def make_matrix(a,b,c,d) {
            match fn {
                0 => match fn {
                    0=> a,
                    1=> b,
                },
                1 => match fn {
                    0=> c,
                    1=> d,
                },

                :n=>2,
                :m=>2,
            }
        }

        def matrix_mul(A,B) {
            match A(:n)==B(:m){
                false => {return :err;},
                true => {}
            };

            make_entry = fn(n,m) -> {
                fn(i,agg) -> {
                    match i==A(:n){
                        true =>{return  agg;},
                        false =>{}
                    };

                    c = A(i)(m)*B(n)(i);
                    self(i-1,c+agg)
                }(A(:n),0)
            };

            make_entry(1,1);
            make_entry(0,0);
            
            _make_row = fn(i,m,row) {
                match i==A(:m){
                    true =>{return  row;},
                    false =>{}
                };

                row = fn(x) {
                    match x==i {
                        true => make_entry(m,i),
                        false=> row(x) 
                    }
                };

                self(i+1,m,row)
            };

            empty_row= match fn {_ => :err_matrix};

            make_row = fn(m) -> {
                _make_row(0,m,empty_row)
            };

            fn(x) -> {
                match x {
                    :m => A(:m),
                    :n => B(:n),
                    _ => make_row(x)
                }
            }
        }

        #catches bugs related to closures fairly well
        def main(system) {
            a = 1;
            b = 1;
            c = 1;
            d = 1;

            matrix = make_matrix(a,b,c,d);

            matrix_mul(matrix,matrix)(1)(1) |> system(:println)()
        }
    "#;

    safe_run(input);
}
