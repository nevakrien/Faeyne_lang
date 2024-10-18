
use ast::lexer::Lexer;
use ast::parser::ProgramParser;

use crate::reporting::{report_parse_error,report_err_list};
use crate::value::VarTable;
#[cfg(test)]
use crate::value::Value as IRValue;
use crate::vm::Operation;
use crate::reporting::ErrList;
use std::collections::HashMap;

use ast::ast::{StringTable,FuncDec,OuterExp};
use std::sync::{RwLock,Arc};
use crate::runtime::{Code,FuncHolder};

pub fn translate_program<'a>(outer:&[OuterExp],table:Arc<RwLock<StringTable<'a>>>) ->Result<Code<'a>,ErrList> {
	let mut funcs = Vec::with_capacity(outer.len());
	let mut names = Vec::with_capacity(outer.len());
	let mut name_map = HashMap::new();

	let table_ref = table.read().unwrap();

	for exp in outer {
		match exp {
			OuterExp::ImportFunc(_) => todo!(),
			OuterExp::FuncDec(func) => {
				let index = funcs.len();

				let name = func.sig.name;
				names.push(name);
				let s = table_ref.get_raw_str(name);
				name_map.insert(s.into(),index);

				funcs.push(translate_func(func,&table_ref)?);
			},
		}
	}

	std::mem::drop(table_ref);

	Ok(Code{
		funcs,
		names,
		table,
		name_map,
	})
}

pub fn translate_func<'a>(func:&FuncDec,_table:&StringTable<'a>) -> Result<FuncHolder<'a>,ErrList> {
	let vars = VarTable::default();
	let mut mut_vars = VarTable::default();
	let mut code = Vec::default();

	simple_load_args(&func.sig.args,&mut code,&mut mut_vars);

	for _x in func.body.body.iter() {
		todo!()
	}
	match func.body.ret {
	    None => {
	    	code.push(Operation::PushNil);
	    	code.push(Operation::Return);
	    },
	    Some(_) => todo!(),
	}
	
	Ok(FuncHolder{
		code:code.into(),
		mut_vars_template:mut_vars,
		vars,
	})
	
}

fn simple_load_args<'code>(
	args: &[u32],
	write_spot:&mut Vec<Operation>,
	mut_vars:&mut VarTable<'code>,
){
	let last_arg_id = mut_vars.len();
	mut_vars.add_ids(args);
	println!("{:?}", mut_vars);

	for i in (last_arg_id..mut_vars.len()).rev() {
		println!("adding arg ({:?})",i );
		write_spot.push(Operation::PopArgTo(i));
	}
	write_spot.push(Operation::PopTerminator)
}

// This function handles the process of taking source code and returning a `Code` object.
pub fn compile_source_to_code<'a>(source_code: &'a str) -> Code<'a> {
    // Step 1: Setup the StringTable (we will use Arc<RwLock<StringTable>> as required)
    let string_table = Arc::new(RwLock::new(StringTable::new()));

    // Step 2: Parse the source code into an AST
    let lexer = Lexer::new(source_code);
    let parser = ProgramParser::new();
    let mut write_table = string_table.try_write().unwrap();

    let parsed_ast = match parser.parse(source_code, &mut write_table, lexer) {
        Ok(ast) => ast,
        Err(e) => {
            report_parse_error(e, source_code, &write_table);
            panic!("Failed to parse input");
        }
    };
    std::mem::drop(write_table);

    // Step 3: Translate the AST into the VM's bytecode using `translate_program`
    match translate_program(&parsed_ast, string_table.clone()) {
        Ok(code) => code,
        Err(e) => {
            report_err_list(&e, source_code, &string_table.try_read().unwrap());
            panic!("Failed to translate the program");
        }
    }
}

#[test]
fn test_end_to_end_empty_function() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = "
        def main(a, b, c) {
            # This function does nothing and returns immediately
        }
    ";

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Setup the initial arguments for the "main" function (arbitrary values)
    let args = vec![
        IRValue::Nil,        // c = Nil (or None equivalent)
        IRValue::Int(1),     // a = 1
        IRValue::Bool(true), // b = true
    ];

    // Step 4: Run the translated code and call the "main" function with the arguments
    let _result = code.run("main", args).unwrap();
}
