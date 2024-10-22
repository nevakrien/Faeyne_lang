use clap::{Arg, Command};
use faeyne_lang::value::Value;
use faeyne_lang::system::system;
use faeyne_lang::reporting::report_err_list;
use std::fs;
use std::process;

use faeyne_lang::translate::compile_source_to_code;

fn main() {
    // Define the command-line argument structure using clap
    let matches = Command::new("Faeyne_lang Runner")
        .version("0.1")
        .author("Your Name <your.email@example.com>")
        .about("Runs Faeyne_lang scripts with optional repetition")
        .arg(Arg::new("file")
            .help("The Faeyne_lang script file to run")
            .default_value("sample.fay")
            .index(1))
        .arg(Arg::new("repeat")
            .short('r')
            .long("repeat")
            .help("Number of times to repeat the execution")
            .default_value("1"))
        .get_matches();

    // Get the file path and repeat count
    let file_path = matches.get_one::<String>("file").unwrap();
    let repeat_count: usize = matches.get_one::<String>("repeat")
        .unwrap()
        .parse()
        .unwrap_or_else(|_| {
            eprintln!("Invalid repeat count");
            process::exit(1);
        });

    // Read the source code from the file
    let source_code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read file: {}", file_path);
            process::exit(1);
        }
    };

    // Compile the source code
    let code = compile_source_to_code(&source_code);

    // Run the code multiple times based on the repeat count
    for _ in 0..repeat_count {
        match code.run("main", vec![Value::StaticFunc(system)]) {
            Ok(()) => {},
            Err(e) => report_err_list(&e, &source_code, &code.table.try_read().unwrap()),
        }
    }
}
