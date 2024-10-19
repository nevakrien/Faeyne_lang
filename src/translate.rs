
use ast::ast::Value as AstValue;
use ast::lexer::Lexer;
use ast::parser::ProgramParser;

use crate::reporting::{report_parse_error,report_err_list};
use crate::value::VarTable;
#[cfg(test)]
use crate::value::Value as IRValue;
use crate::vm::Operation;
use crate::reporting::ErrList;
use std::collections::HashMap;

use ast::ast::{StringTable,FuncDec,OuterExp,Ret,Statment};
use std::sync::{RwLock,Arc};
use crate::runtime::{Code,FuncHolder};

enum RetRef<'a> {
	Func(&'a AstValue),
	Block(&'a AstValue),
}

enum CallType{
	TailCall,
	FullCall,
}
use CallType::*;

impl<'a> From<&'a Ret> for RetRef<'a> {

	fn from(r: &'a Ret) -> Self {
		match r  {
			Ret::Exp(x) => RetRef::Func(x),
			Ret::Imp(x) => RetRef::Block(x),
		}
	}
}

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
	let mut vars = VarTable::default();
	let mut mut_vars = VarTable::default();
	mut_vars.add_ids(&func.sig.args);

	let mut code = Vec::default();

	for x in func.body.body.iter() {
		match x{
			Statment::Match(_) => todo!(),
			Statment::Assign(_, _) => todo!(),
			Statment::Call(_) => todo!(),
		}
	}

	let ret = func.body.ret.as_ref().map(|x| RetRef::Func(x.get_value()));
	translate_ret(ret,TailCall,&mut code,&mut mut_vars,&mut vars)?;
	
	Ok(FuncHolder{
		code:code.into(),
		mut_vars_template:mut_vars,
		vars,
		num_args:func.sig.args.len()
	})
	
}

#[inline]
fn translate_ret(
	x:Option<RetRef>,_tail:CallType,
	
	code:&mut Vec<Operation>,
	_mut_vars:&mut VarTable,
	_vars:&mut VarTable,

) -> Result<(),ErrList> {
	match x {
	    None => {
	    	code.push(Operation::PushNil);
	    	code.push(Operation::Return);
	    },
	    Some(_) => todo!(),
	}
	Ok(())
}

// This function handles the process of taking source code and returning a `Code` object.
pub fn compile_source_to_code(source_code: &str) -> Code<'_> {
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
    code.run("main", args).unwrap();
}
