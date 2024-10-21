use faeyne_lang::value::Value;
use faeyne_lang::system::system;
use faeyne_lang::reporting::report_err_list;
use std::env;
use std::fs;
use std::process;

use faeyne_lang::translate::compile_source_to_code;

fn main() {
    //input handeling

    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "sample.fay"
    };

    
    let source_code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Failed to read file: {}", file_path);
            process::exit(1);
        }
    };

    let code =  compile_source_to_code(&source_code);
    match code.run("main",vec![Value::StaticFunc(system)]){
        Ok(()) => {},
        Err(e) => report_err_list(&e, &source_code, &code.table.try_read().unwrap())
    }
}