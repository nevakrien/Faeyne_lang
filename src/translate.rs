
use ast::ast::Literal;
use crate::reporting::stacked_error;
use crate::reporting::sig_error;
use ast::ast::MatchOut;
use ast::ast::MatchPattern;
use ast::ast::Value as AstValue;
use ast::lexer::Lexer;
use ast::parser::ProgramParser;

use crate::reporting::{report_parse_error,report_err_list,ErrList};
use crate::value::VarTable;
use crate::value::Value as IRValue;
use crate::vm::{Operation,StaticMatch};
use std::collections::HashMap;

use ast::ast::{StringTable,FuncDec,OuterExp,Ret,Statment,FValue,FunctionCall,BuildIn,MatchStatment};
use std::sync::{RwLock,Arc};
use crate::runtime::{Code,FuncHolder};

#[derive(Debug,PartialEq,Clone,Copy)]
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
			Statment::Match(m) =>{
				translate_match(m,&mut handle,FullCall)?;
				handle.code.push(Operation::PopDump);
				handle.code.push(Operation::PushNil);
			},
			Statment::Assign(_, _) => todo!(),
			Statment::Call(call) => {
				translate_call_raw(call,&mut handle,FullCall)?;
				handle.code.push(Operation::PopDump);
				handle.code.push(Operation::PushNil);
			}
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
	x:&Option<Ret>,tail:CallType,
	
	handle:&mut TransHandle

) -> Result<(),ErrList> {
	match x {
	    None => {
	    	handle.code.push(Operation::PushNil);
	    },
	    Some(r) => {
	    	translate_value(r.get_value(),handle,tail)?;
	    }
	}
	handle.code.push(Operation::Return);
	Ok(())
}

fn translate_value(v:&AstValue,handle: &mut TransHandle,tail:CallType) -> Result<(),ErrList> {
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
		AstValue::FuncCall(call) => translate_call_raw(call,handle,tail)?,
		AstValue::Match(m) => translate_match(m,handle,tail)?,
	    _ => todo!(),
	};
	Ok(())
}

fn translate_call_raw(call:&FunctionCall,handle: &mut TransHandle,tail:CallType) -> Result<(),ErrList> {
	if !matches!(call.name,FValue::BuildIn(_)) {
		handle.code.push(Operation::PushTerminator);
	}
	else {
    	#[cfg(feature = "debug_terminators")]
    	handle.code.push(Operation::PushTerminator);
	}
	
	for a in call.args.iter() {
		translate_value(a,handle,FullCall)?;
	}

	match call.name {
		FValue::SelfRef(_) => match tail {
			TailCall => handle.code.push(Operation::CallThis),
			FullCall => {
				handle.code.push(Operation::PushThis);
				handle.code.push(Operation::Call(call.debug_span));
			}
		},		
		FValue::BuildIn(op) => match op {
			BuildIn::Add => handle.code.push(Operation::Add(call.debug_span)),
			BuildIn::Sub => handle.code.push(Operation::Sub(call.debug_span)),
			BuildIn::Mul => handle.code.push(Operation::Mul(call.debug_span)),
			BuildIn::Div => handle.code.push(Operation::Div(call.debug_span)),
			BuildIn::IntDiv => handle.code.push(Operation::IntDiv(call.debug_span)),

			BuildIn::Modulo => handle.code.push(Operation::Modulo(call.debug_span)),


			BuildIn::Pow => handle.code.push(Operation::Pow(call.debug_span)),
			BuildIn::Equal => handle.code.push(Operation::Equal(call.debug_span)),
			BuildIn::NotEqual => handle.code.push(Operation::NotEqual(call.debug_span)),
			
			BuildIn::Bigger => handle.code.push(Operation::Bigger(call.debug_span)),
			BuildIn::BiggerEq => handle.code.push(Operation::BiggerEq(call.debug_span)),
			BuildIn::Smaller => handle.code.push(Operation::Smaller(call.debug_span)),
			BuildIn::SmallerEq => handle.code.push(Operation::SmallerEq(call.debug_span)),
			
			BuildIn::Xor => handle.code.push(Operation::Xor(call.debug_span)),
			BuildIn::DoubleXor => handle.code.push(Operation::DoubleXor(call.debug_span)),
			BuildIn::And => handle.code.push(Operation::And(call.debug_span)),
			BuildIn::DoubleAnd => handle.code.push(Operation::DoubleAnd(call.debug_span)),
			BuildIn::Or => handle.code.push(Operation::Or(call.debug_span)),
			BuildIn::DoubleOr => handle.code.push(Operation::DoubleOr(call.debug_span)),

		},
		_ => todo!(),
	}

	Ok(())	
}

fn translate_match(m:&MatchStatment,handle:&mut TransHandle,tail:CallType) -> Result<(),ErrList> {
	translate_value(&m.val,handle,tail)?;
	let mut ans = Box::new(StaticMatch::default());
	ans.span = m.debug_span;

	let in_id = handle.code.len();
	handle.code.push(Operation::PopTo(100000000));//trap instraction

	let mut return_spots = Vec::new();

	if let Some((last, rest)) = m.arms.split_last() {
	    for c in rest {
	    	match &c.pattern {
	    	    MatchPattern::Literal(v) => {
	    	    	let val = literal_to_IRValue(v,handle.table);
	    	    	ans.map.insert(val,handle.code.len());
	    	    	match &c.result {
		    			MatchOut::Value(v) =>translate_value(v,handle,tail)?,
		    			MatchOut::Block(_) => todo!(),
		    		}

	    	    	return_spots.push(handle.code.len());
	    			handle.code.push(Operation::PopTo(100000000));//trap instraction 
	    	    }
	    	    MatchPattern::Variable(_) => todo!(),
	    	    MatchPattern::Wildcard => {
	    	    	//todo fix this up to be a proper error type
	    	    	return Err(stacked_error("while defining match",sig_error(),m.debug_span));
	    	    }
	    	}
	    }
	    
	    match &last.pattern {
	    	 MatchPattern::Literal(v) => {
	    	    	let val = literal_to_IRValue(&v,handle.table);
	    	    	ans.map.insert(val,handle.code.len());
	    	    	match &last.result {
		    			MatchOut::Value(v) =>translate_value(v,handle,tail)?,
		    			MatchOut::Block(_) => todo!(),
		    		}
		    },
	    	MatchPattern::Variable(_) => todo!(),
	    	MatchPattern::Wildcard => {
	    		ans.default=Some(handle.code.len());
	    	
	    		match &last.result {
	    			MatchOut::Value(v) =>translate_value(v,handle,tail)?,
	    			MatchOut::Block(_) => todo!(),
	    		}
	    	}
	    }
	}

	for r in return_spots {
		handle.code[r] = Operation::Jump(handle.code.len());
	}


	handle.code[in_id]=Operation::MatchJump(ans);
	Ok(())
}

#[allow(non_snake_case)]
fn literal_to_IRValue(l: &Literal,table:&StringTable) -> IRValue<'static> {
	match l {
		Literal::Int(i) => IRValue::Int(*i),
        Literal::Float(f) => IRValue::Float(*f),
        Literal::Atom(a) => IRValue::Atom(*a),
        Literal::String(s) => IRValue::String(Arc::new(table.get_escaped_string(*s))),
        Literal::Bool(b) => IRValue::Bool(*b),
        Literal::Nil => IRValue::Nil,
	}
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

