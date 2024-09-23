use crate::ast::StringTable;
use crate::ir;
use crate::ir::Value;

use crate::translate::translate_program;
use crate::system::get_system;
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


pub unsafe fn clean_string_run(junk:(*mut ir::GlobalScope,*mut StringTable<'static>,*mut str)){
    let (global_raw,table_raw,raw_str) = junk;
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

pub unsafe fn clean_str_run(junk: (*mut ir::GlobalScope,*mut StringTable<'static>)){
    let (global_raw,table_raw) = junk;
    
    if !global_raw.is_null(){
        _ = Box::from_raw(&mut *global_raw);
    }
    {
        _ = Box::from_raw(&mut *table_raw);
    }
}

pub fn run_str(input_ref: &'static str) ->(Value<'static>,(*mut ir::GlobalScope,*mut StringTable<'static>)) {
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

    let ir::Value::Func(main_func) = global.get(table.get_id("main")).expect("We need a main function") else {unreachable!()};

    let system = get_system(table);

    let ans = main_func.eval(vec![system]).unwrap();

    (ans,(global_raw, table_raw))
}


pub fn run_string(code: String) -> (Value<'static>,(*mut ir::GlobalScope<'static>,*mut StringTable<'static>,*mut str)) {
    let input_ref = code.leak();
    let raw_str = input_ref as *mut str;

    let (ans,(global_raw, table_raw)) = run_str(input_ref);

    (ans,(global_raw, table_raw, raw_str))
}
