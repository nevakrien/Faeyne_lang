use std::env;
use std::fs;
use std::process;

use faeyne_lang::runners::{run_string, clean_string_run};

fn main() {
    //input handeling

    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "sample.fay"
    };

    
    let input = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read file: {}", file_path);
            process::exit(1);
        }
    };

    
    
    let (global_raw, table_raw, raw_str) = run_string(input);

    // This is here to test if we can manually free memory.
    //probably should have been handled via lifetimes but after half an hour of cleanup meh
    unsafe {
        clean_string_run(global_raw, table_raw, raw_str);
    }
}

