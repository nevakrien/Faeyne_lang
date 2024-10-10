// Main function for performance testing the Value stack

use faeyne_lang::stack::{ValueStack};
use faeyne_lang::value::Value;
use std::time::Instant;

fn main() {
    let mut stack = Box::new(ValueStack::new());

    let start = Instant::now();

    for _ in 0..100 {
        // Start the timer

        // Initial push of a few values
        for i in 0..100 {
            stack.push_value(Value::Int(i as i64)).unwrap();
            stack.push_value(Value::Bool(i % 2 == 0)).unwrap();
            stack.push_value(Value::Atom(i as u32)).unwrap();
        }



        // Simulate program behavior with alternating push/pop
        for _ in 0..1_000_000 {
            stack.push_value(Value::Int(42)).unwrap();
            stack.push_value(Value::Bool(true)).unwrap();
            stack.push_value(Value::Atom(7)).unwrap();
            // stack.pop_value().unwrap();
            // stack.pop_value().unwrap();
            // stack.push_value(&Value::String(99)).unwrap();
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
        }


        // Pop the original values
        for _ in 0..100 {
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
        }

        std::mem::drop(stack);
        stack = Box::new(ValueStack::new());
    }

     // Stop the timer and print the elapsed time
    let duration = start.elapsed();
    println!("Performance test completed in: {:?}", duration);
}
