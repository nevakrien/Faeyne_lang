// Main function for performance testing the Value stack

use faeyne_lang::value::{Value, ValueStack};
use faeyne_lang::stack::Stack;
use std::time::Instant;

fn main() {
    const STACK_SIZE: usize = 1_000_000;
    let mut stack: Stack<STACK_SIZE> = Stack::new();

    // Start the timer
    let start = Instant::now();

    // Initial push of a few values
    for i in 0..100 {
        stack.push_value(&Value::new_int(i as i32));
        stack.push_value(&Value::new_bool(i % 2 == 0));
        stack.push_value(&Value::new_atom(i as u32));
    }

    // Simulate program behavior with alternating push/pop
    for _ in 0..1_000_000 {
        stack.push_value(&Value::new_int(42));
        stack.push_value(&Value::new_bool(true));
        stack.push_value(&Value::new_atom(7));
        let _ = stack.pop_value().expect("Failed to pop value from stack");
        let _ = stack.pop_value().expect("Failed to pop value from stack");
        stack.push_value(&Value::new_string(99));
        let _ = stack.pop_value().expect("Failed to pop value from stack");
        let _ = stack.pop_value().expect("Failed to pop value from stack");
    }

    // Pop the original values
    for _ in 0..100 {
        let _ = stack.pop_value().expect("Failed to pop value from stack");
        let _ = stack.pop_value().expect("Failed to pop value from stack");
        let _ = stack.pop_value().expect("Failed to pop value from stack");
    }

    // Stop the timer and print the elapsed time
    let duration = start.elapsed();
    println!("Performance test completed in: {:?}", duration);
}
