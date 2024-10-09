use faeyne_lang::value::StringRegistry;
use std::time::Instant;

pub fn main() {
    let mut registry = StringRegistry::new(1000);

    // Advanced stress test with alternating insertions, deletions, and retrievals
    let start = Instant::now();
    let mut operation_count = 0;
    for i in 2001..=5000 {
        registry.insert(format!("advanced_value_{}", i));
        operation_count += 1;
        if i % 3 == 0 {
            registry.del(i - 1500);
            operation_count += 1;
        }
        if i % 4 == 0 {
            registry.get(i - 1000);
            operation_count += 1;
        }
        if i % 5 == 0 {
            registry.insert(format!("extra_value_{}", i));
            operation_count += 1;
        }
    }
    let duration = start.elapsed();
    let avg_time_per_operation = duration.as_secs_f64() / operation_count as f64 * 1_000_000.0; // Average time per operation in microseconds
    println!("Time taken for stress test: {:?}", duration);
    println!("Total operations: {}", operation_count);
    println!("Average time per operation: {:.3}µs", avg_time_per_operation);
}
