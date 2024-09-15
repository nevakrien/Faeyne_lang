use std::env;
use std::fs;
use std::process;

use faeyne_lang::lexer::Lexer;
use faeyne_lang::parser;
use faeyne_lang::ast::StringTable;

fn main() {
    // Get the command line arguments.
    let args: Vec<String> = env::args().collect();
    
    // If a file path is provided, use it; otherwise, use "sample.fay".
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "sample.fay"
    };

    // Read the file content, or exit on failure.
    let input = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read file: {}", file_path);
            process::exit(1);
        }
    };

    
    // Create the lexer and string table.
    let lexer = Lexer::new(&input);
    let mut table = StringTable::new();
    
    // Create the parser.
    let parser = parser::ProgramParser::new();
    
    // Parse the program.
    let result = parser.parse(&input, &mut table, lexer);
    
    // Print the parsed program or an error message.
    match result {
        Ok(program) => {
            for exp in program {
                println!("{:?}\n\n", exp);
            }
            
        }
        Err(err) => {
            eprintln!("Error parsing the program: {:?}", err);
            process::exit(1);
        }
    }
}
