use crate::ast::StringTable;
use crate::ir;
use crate::translate::translate_program;
use crate::system::get_system;
use crate::lexer::Lexer;
use crate::parser;

use std::process;

pub unsafe fn clean_string_run(junk:(*mut ir::GlobalScope,*mut StringTable<'static>,*mut str)){
    let (global_raw,table_raw,raw_str) = junk;
    {
        _ = Box::from_raw(&mut *global_raw);
    }
    {
        _ = Box::from_raw(&mut *table_raw);
    }
    {
        _ = Box::from_raw(&mut *raw_str);
    }
}

pub unsafe fn clean_str_run(junk: (*mut ir::GlobalScope,*mut StringTable<'static>)){
    let (global_raw,table_raw) = junk;
    
    {
        _ = Box::from_raw(&mut *global_raw);
    }
    {
        _ = Box::from_raw(&mut *table_raw);
    }
}

pub fn run_str(input_ref: &'static str) -> (*mut ir::GlobalScope,*mut StringTable<'static>) {
    let lexer = Lexer::new(input_ref);
    let table = Box::leak(Box::new(StringTable::new()));
    let table_raw = table as *mut StringTable;

    let parser = parser::ProgramParser::new();
    let result = parser.parse(input_ref, table, lexer);

    let global = Box::leak(translate_program(result.unwrap(), table).unwrap());
    let global_raw = global as *mut ir::GlobalScope;

    let ir::Value::Func(main_func) = global.get(table.get_id("main")).expect("We need a main function") else {unreachable!()};

    let system = get_system(table);

    let _ans = main_func.eval(vec![system]).unwrap();

    (global_raw, table_raw)
}


pub fn run_string(code: String) -> (*mut ir::GlobalScope,*mut StringTable<'static>,*mut str) {
    let input_ref = code.leak();
    let raw_str = input_ref as *mut str;

    let (global_raw, table_raw) = run_str(input_ref);

    (global_raw, table_raw, raw_str)
}
