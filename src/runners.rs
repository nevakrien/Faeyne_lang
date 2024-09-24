use crate::system::MAIN_ID;
use crate::get_id;
use crate::ast::StringTable;
use crate::ir;
use crate::ir::Value;

use crate::translate::translate_program;
use crate::system::{get_system,FreeHandle};
use crate::lexer::Lexer;
use crate::parser;

use crate::reporting::report_parse_error;

pub fn safe_run_compare(input: &'static str, expected: Value<'static>) {
    let (ans, junk) = run_str(input);
    assert_eq!(ans, expected);
    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
}

pub fn safe_run(input: &'static str) {
    let (ans, junk) = run_str(input);
    std::mem::drop(ans);
    unsafe { clean_str_run(junk); }
}


pub unsafe fn clean_string_run(junk:(FreeHandle<'static>,*mut ir::GlobalScope,*mut StringTable<'static>,*mut str)){
    let (handle,global_raw,table_raw,raw_str) = junk;
    {
    handle.free();

    }
    
    if !global_raw.is_null(){
        _ = Box::from_raw(&mut *global_raw);
    }
    {
        _ = Box::from_raw(&mut *table_raw);
    }
    {
        _ = Box::from_raw(&mut *raw_str);
    }
}

pub unsafe fn clean_str_run(junk: (FreeHandle<'static>,*mut ir::GlobalScope<'static>, *mut StringTable<'static>)){
    let (handle,global_raw,table_raw) = junk;
    {
        handle.free();

    }
    if !global_raw.is_null(){
        _ = Box::from_raw(&mut *global_raw);
    }
    {
        _ = Box::from_raw(&mut *table_raw);
    }
}

pub fn run_str(input_ref: &'static str) ->(Value<'static>,(FreeHandle<'static>,*mut ir::GlobalScope<'static>, *mut StringTable<'static>)) {
    let lexer = Lexer::new(input_ref);
    let table = Box::leak(Box::new(StringTable::new()));
    let table_raw = table as *mut StringTable;

    let parser = parser::ProgramParser::new();
    let result = match parser.parse(input_ref, table, lexer) {
        Ok(r) =>  r,
        Err(e) => {
            report_parse_error(e,input_ref); 
            panic!();
        }
    };



    let global = Box::leak(translate_program(result, table).unwrap());
    let global_raw = global as *mut ir::GlobalScope;

    let ir::Value::Func(main_func) = global.get(get_id!("main")).expect("We need a main function") else {unreachable!()};

    let (system,handle) = get_system(table);

    let ans = main_func.eval(vec![system]).unwrap();

    (ans,(handle,global_raw, table_raw))
}


pub fn run_string(code: String) -> (Value<'static>,(FreeHandle<'static>, *mut ir::GlobalScope<'static>,*mut StringTable<'static>,*mut str)) {
    let input_ref = code.leak();
    let raw_str = input_ref as *mut str;

    let (ans,(handle, global_raw, table_raw)) = run_str(input_ref);

    (ans,(handle, global_raw, table_raw, raw_str))
}
