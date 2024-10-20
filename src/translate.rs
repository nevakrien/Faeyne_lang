
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

enum CallType{
	TailCall,
	FullCall,
}
use CallType::*;

// enum RetRef<'a> {
// 	Func(Option<&'a AstValue>),
// 	Block(Option<&'a AstValue>),
// }


// impl<'a> From<&'a Ret> for RetRef<'a> {

// 	fn from(r: &'a Ret) -> Self {
// 		match r  {
// 			Ret::Exp(x) => RetRef::Func(Some(x)),
// 			Ret::Imp(x) => RetRef::Block(Some(x)),
// 		}
// 	}
// }

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

pub fn translate_func<'a>(func:&FuncDec,table:&StringTable<'a>) -> Result<FuncHolder<'a>,ErrList> {
	let mut vars = VarTable::default();
	let mut mut_vars = VarTable::default();
	mut_vars.add_ids(&func.sig.args);

	let mut code = Vec::default();

	let mut handle = TransHandle{
		code:&mut code,
		vars:&mut vars,
		mut_vars:&mut mut_vars,
		table
	};

	for x in func.body.body.iter() {
		match x{
			Statment::Match(_) => todo!(),
			Statment::Assign(_, _) => todo!(),
			Statment::Call(_) => todo!(),
		}
	}

	translate_ret_func(&func.body.ret,TailCall,&mut handle)?;
	
	Ok(FuncHolder{
		code:code.into(),
		mut_vars_template:mut_vars,
		vars,
		num_args:func.sig.args.len()
	})
	
}

struct TransHandle<'a> {
	code:&'a mut Vec<Operation>,
	mut_vars:&'a mut VarTable<'static>,
	vars:&'a mut VarTable<'static>,
	table:&'a StringTable<'a>,
}

#[inline]
fn translate_ret_func(
	x:&Option<Ret>,_tail:CallType,
	
	handle:&mut TransHandle

) -> Result<(),ErrList> {
	match x {
	    None => {
	    	handle.code.push(Operation::PushNil);
	    },
	    Some(r) => {
	    	translate_value(r.get_value(),handle)?;
	    }
	}
	handle.code.push(Operation::Return);
	Ok(())
}

fn translate_value(v:&AstValue,handle: &mut TransHandle ) -> Result<(),ErrList> {
	match v {
		AstValue::Nil => handle.code.push(Operation::PushNil),
		AstValue::Bool(b) => handle.code.push(Operation::PushBool(*b)),
		AstValue::Atom(a) => handle.code.push(Operation::PushAtom(*a)),

		AstValue::Float(f) => handle.code.push(Operation::PushFloat(*f)),
		AstValue::Int(i) => handle.code.push(Operation::PushInt(*i)),
		AstValue::String(id) => {
			let s = Arc::new(handle.table.get_escaped_string(*id));
			handle.code.push(Operation::PushString(s))
		},

		AstValue::SelfRef(_) => handle.code.push(Operation::PushThis),
	    _ => todo!(),
	};
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

#[test]
fn test_return_true() {
    // Step 1: Define the source code (a function that does nothing)
    let source_code = "
        def main() {
            true
        }
    ";

    // Step 2: Compile the source code to a `Code` object
    let code = compile_source_to_code(source_code);

    // Step 3: Run the translated code and call the "main" function with the arguments
    assert!(code.run_compare("main", vec![],IRValue::Bool(true)).unwrap());
}

