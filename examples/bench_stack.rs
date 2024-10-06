// Main function for performance testing the Value stack

use faeyne_lang::value::{InterpretedValue, ValueStack};
use faeyne_lang::stack::Stack;
use std::time::Instant;

fn main() {
    const STACK_SIZE: usize = 1_000_000;
    let mut stack: Stack<STACK_SIZE> = Stack::new();

    let start = Instant::now();

    for _ in 0..100 {
        // Start the timer

        // Initial push of a few values
        for i in 0..100 {
            stack.push_value(&InterpretedValue::Int(i as i64)).unwrap();
            stack.push_value(&InterpretedValue::Bool(i % 2 == 0)).unwrap();
            stack.push_value(&InterpretedValue::Atom(i as u32)).unwrap();
        }

        // Simulate program behavior with alternating push/pop
        for _ in 0..1_000_000 {
            stack.push_value(&InterpretedValue::Int(42)).unwrap();
            stack.push_value(&InterpretedValue::Bool(true)).unwrap();
            stack.push_value(&InterpretedValue::Atom(7)).unwrap();
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
            stack.push_value(&InterpretedValue::String(99)).unwrap();
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
        }

        // Pop the original values
        for _ in 0..100 {
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
            stack.pop_value().unwrap();
        }
    }

     // Stop the timer and print the elapsed time
    let duration = start.elapsed();
    println!("Performance test completed in: {:?}", duration);
}
