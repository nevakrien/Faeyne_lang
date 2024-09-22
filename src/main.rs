use std::env;
use std::fs;
use std::process;

use faeyne_lang::lexer::Lexer;
use faeyne_lang::parser;
use faeyne_lang::ast::StringTable;
use faeyne_lang::ir;
        
use faeyne_lang::translate::translate_program;
use faeyne_lang::system::get_system;
// use faeyne_lang::reporting::ErrList;

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

    let input_ref = input.leak();
    
    // Create the lexer and string table.
    let lexer = Lexer::new(input_ref);
    let mut table = Box::new(StringTable::new());
    
    // Create the parser.
    let parser = parser::ProgramParser::new();
    
    // Parse the program.
    let result = parser.parse(input_ref, &mut table, lexer);
    
    // // Print the parsed program or an error message.
    // match result {
    //     Ok(program) => {
    //         for exp in program {
    //             println!("{:?}\n\n", exp);
    //         }
            
    //     }
    //     Err(err) => {
    //         eprintln!("Error parsing the program: {:?}", err);
    //         process::exit(1);
    //     }
    // }

    let global = Box::leak(translate_program(result.unwrap(),&table).unwrap());
    let ir::Value::Func(main_func) = global.get(table.get_id("main")).expect("we need a main function") else {unreachable!()};

    let system = get_system(Box::leak(table));

    let _ans = main_func.eval(vec![system]).unwrap(); 

}
