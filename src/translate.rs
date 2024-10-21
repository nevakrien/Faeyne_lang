
use ast::ast::MatchLambda;
use crate::vm::FuncMaker;
use codespan::Span;
use ast::ast::Value as AstValue;
use ast::lexer::Lexer;
use ast::parser::ProgramParser;
use ast::ast::{
	StringTable,FuncBlock,OuterExp,Ret,Statment,
	FValue,FunctionCall,BuildIn,
	MatchStatment,MatchArm,MatchOut,MatchPattern,Literal,
	Lambda
};


use crate::reporting::{report_parse_error,report_err_list,ErrList,stacked_error,sig_error,missing_error,unreachable_func_error};
use crate::value::VarTable;
use crate::value::Value as IRValue;
use crate::vm::{Operation,StaticMatch};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::sync::{RwLock,Arc};
use crate::runtime::{Code,FuncHolder};

#[derive(Debug,PartialEq,Clone,Copy)]
enum CallType{
	TailCall,
	FullCall,
}
use CallType::*;

struct TransHandle<'a> {
	code:&'a mut Vec<Operation>,
	mut_vars:&'a mut VarTable<'static>,
	vars:&'a mut VarTable<'static>,
	table:&'a StringTable<'a>,
}

trait NameSpace {
	fn set(&mut self,handle:&mut TransHandle,name:u32);

	//called by child which DOES mutate the parent scope. but does so in a safe way
	fn get(&mut self,handle:&mut TransHandle,name:u32) -> Result<(),ErrList>;

	//called by lambda and does NOT mutate anything. does not rely on handle in any way 
	fn capture(&self,name:u32) -> Result<CaptureType,ErrList>;
}

enum CaptureType {
	Global(usize),
	Local
}

struct FuncScope<'a> {
	global_vars: &'a HashMap<u32,usize>,
	assigns: HashMap<u32,usize>,
}

impl<'a> FuncScope<'a> {
	fn start(global_vars: &'a HashMap<u32,usize>,args:&[u32]) -> Self {
		let mut assigns = HashMap::new();
		for (i,name) in args.iter().enumerate() {
			assigns.insert(*name,i);
		}
		FuncScope{
			global_vars,
			assigns
		}
	}
}

impl NameSpace for FuncScope<'_> {
	fn get(&mut self,handle: &mut TransHandle, name: u32) -> Result<(),ErrList> {
		let op = match self.assigns.get(&name) {
		    Some(id) => Operation::PushFrom(*id),
		    None => {
		    	let id = self.global_vars.get(&name).ok_or_else(|| missing_error(name))?;
		    	Operation::PushGlobal(*id)
		    }
		};
		handle.code.push(op);
		Ok(())
	}
	fn set(&mut self, handle: &mut TransHandle, name: u32) {
	    let op = match self.assigns.entry(name) {
	        Entry::Occupied(entry) => {
	            Operation::PopTo(*entry.get())
	        }
	        Entry::Vacant(spot) => {

	            let id = handle.mut_vars.len();
	            handle.mut_vars.add_ids(&[name]);
	            spot.insert(id);
	            Operation::PopTo(id)
	        }
	    };
	    handle.code.push(op);
	}

	fn capture(&self,name:u32) -> Result<CaptureType,ErrList> {
		match self.assigns.get(&name) {
		    Some(_id) => Ok(CaptureType::Local),
		    None => {
		    	let id = self.global_vars.get(&name).ok_or_else(|| missing_error(name))?;
		    	Ok(CaptureType::Global(*id))
		    }
		}
	}
}

struct ChildScope<'a> {
	parent: &'a mut dyn NameSpace,
	assigns: HashMap<u32,usize>,
}

impl<'a> ChildScope<'a> {
	fn new(parent: &'a mut dyn NameSpace,) -> Self {
		ChildScope{
			parent,
			assigns:HashMap::new()
		}
	}
}

impl NameSpace for ChildScope<'_> {
	fn get(&mut self,handle: &mut TransHandle, name: u32) -> Result<(),ErrList> {
		match self.assigns.get(&name) {
		    Some(id) => handle.code.push(Operation::PushFrom(*id)),
		    None => self.parent.get(handle,name)?,
		};
		Ok(())
	}
	fn set(&mut self, handle: &mut TransHandle, name: u32) {
	    let op = match self.assigns.entry(name) {
	        Entry::Occupied(entry) => {
	            Operation::PopTo(*entry.get())
	        }
	        Entry::Vacant(spot) => {

	            let id = handle.mut_vars.len();
	            handle.mut_vars.add_ids(&[name]);
	            spot.insert(id); 
	            Operation::PopTo(id)
	        }
	    };
	    handle.code.push(op);
	}

	fn capture(&self,name:u32) -> Result<CaptureType,ErrList> {
		match self.assigns.get(&name) {
		    Some(_id) => Ok(CaptureType::Local),
		    None => self.parent.capture(name),
		}
	}
}

enum LambdaVar {
    Mut(usize),         
    Captured(usize),
    Global(usize),        
}

impl LambdaVar {		
	fn get_op(&self) -> Operation{
		match self {
	    	LambdaVar::Global(id) => Operation::PushGlobal(*id),
	    	LambdaVar::Mut(id) => Operation::PushFrom(*id),
	    	LambdaVar::Captured(id) =>  Operation::PushLocal(*id),
	    }
	}
}

struct LambdaScope<'a> {
    parent: &'a mut dyn NameSpace,
    map: HashMap<u32, LambdaVar>,
}

impl<'a> LambdaScope<'a> {
    fn start(parent: &'a mut dyn NameSpace,args:&[u32]) -> Self {
        let mut map = HashMap::new();
		for (i,name) in args.iter().enumerate() {
			map.insert(*name,LambdaVar::Mut(i));
		}
        LambdaScope {
            parent,
            map
        }
    }
}

impl NameSpace for LambdaScope<'_> {
	fn get(&mut self,handle:&mut TransHandle,name:u32) -> Result<(),ErrList> {
		let op = match self.map.entry(name) {
		    Entry::Occupied(entry) => entry.get().get_op(),
		    Entry::Vacant(spot) => {
		    	let value = match self.parent.capture(name)?{
		    		CaptureType::Global(id) => LambdaVar::Global(id),
		    		CaptureType::Local => {
		    			let id = handle.vars.names.len();
		    			handle.vars.add_ids(&[name]);
		    			LambdaVar::Captured(id)
		    		}
		    	};
		    	let ans = value.get_op();
		    	spot.insert(value);

		    	ans
		    }
		};

		handle.code.push(op);
		Ok(())
	}

    fn set(&mut self,handle:&mut TransHandle,name:u32)  {
    	let op = match self.map.entry(name) {
    	   Entry::Occupied(mut entry) => match entry.get() {
    	       LambdaVar::Mut(id) => Operation::PopTo(*id),
    	       _ => {
    	       	 let id = handle.mut_vars.len();
	             handle.mut_vars.add_ids(&[name]);
    	       	 entry.insert(LambdaVar::Mut(id));
    	       	 Operation::PopTo(id)
    	       }
    	   },
    	   Entry::Vacant(spot) => {
	            // If the key doesn't exist, insert the new id
	            let id = handle.mut_vars.len();
	            handle.mut_vars.add_ids(&[name]);
	            spot.insert(LambdaVar::Mut(id)); 
	            Operation::PopTo(id)
	        }
    	};

    	handle.code.push(op);

    }
	fn capture(&self,name:u32) -> Result<CaptureType,ErrList> {
		match self.map.get(&name) {
		    None => self.parent.capture(name), 
		    Some(v) => match v {
		    	LambdaVar::Global(id) => Ok(CaptureType::Global(*id)),
		    	LambdaVar::Mut(_) | LambdaVar::Captured(_) => Ok(CaptureType::Local),
		    }
		}
	}
}



pub fn translate_program<'a>(outer:&[OuterExp],table:Arc<RwLock<StringTable<'a>>>) ->Result<Code<'a>,ErrList> {
	let mut funcs = Vec::with_capacity(outer.len());
	let mut names = Vec::with_capacity(outer.len());
	let mut name_map = HashMap::new();

	let table_ref = table.read().unwrap();

	//set up allow vars
	let mut global_vars = HashMap::<u32,usize>::new();
	for (i,exp) in outer.iter().enumerate() {
		match exp {
			OuterExp::ImportFunc(_imp) => todo!(),
			OuterExp::FuncDec(func) => {
				match global_vars.entry(func.sig.name) {
				    Entry::Occupied(_) => {
				    	return Err(unreachable_func_error(func.sig.clone()));
				    },
				    Entry::Vacant(v) => {
				    	v.insert(i);
				    },
				}
			}
		};
	}

	for exp in outer {
		match exp {
			OuterExp::ImportFunc(_) => todo!(),
			OuterExp::FuncDec(func) => {
				let index = funcs.len();

				let name = func.sig.name;
				names.push(name);
				let s = table_ref.get_raw_str(name);
				name_map.insert(s.into(),index);

				let mut scope = FuncScope::start(&global_vars,&func.sig.args);
				funcs.push(translate_func(&func.sig.args,&func.body,&mut scope,&table_ref)?);
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


fn translate_func<'a>(args:&[u32],body:&FuncBlock,name_space:&mut dyn NameSpace,table:&StringTable<'a>) -> Result<FuncHolder<'static>,ErrList> {
	let mut vars = VarTable::default();
	let mut mut_vars = VarTable::default();
	mut_vars.add_ids(&args);

	let mut code = Vec::default();

	let mut handle = TransHandle{
		code:&mut code,
		vars:&mut vars,
		mut_vars:&mut mut_vars,
		table
	};

	for x in body.body.iter() {
		match x{
			Statment::Match(m) =>{
				translate_match(m,name_space,&mut handle,FullCall)?;
				handle.code.push(Operation::PopDump);
				handle.code.push(Operation::PushNil);
			},
			Statment::Assign(id, val) => {
				translate_value(val,name_space,&mut handle,FullCall)?;
				name_space.set(&mut handle,*id);
			},
			Statment::Call(call) => {
				translate_call_raw(call,name_space,&mut handle,FullCall)?;
				handle.code.push(Operation::PopDump);
				handle.code.push(Operation::PushNil);
			}
		}
	}

	translate_ret_func(&body.ret,TailCall,name_space,&mut handle)?;
	
	Ok(FuncHolder{
		code:code.into(),
		mut_vars_template:mut_vars,
		vars,
		num_args:args.len()
	})
	
}



#[inline]
fn translate_ret_func(
	x:&Option<Ret>,tail:CallType,
	
	name_space:&mut dyn NameSpace,handle:&mut TransHandle

) -> Result<(),ErrList> {
	match x {
	    None => {
	    	handle.code.push(Operation::PushNil);
	    },
	    Some(r) => {
	    	translate_value(r.get_value(),name_space,handle,tail)?;
	    }
	}
	handle.code.push(Operation::Return);
	Ok(())
}

fn translate_value(v:&AstValue,name_space:&mut dyn NameSpace,handle: &mut TransHandle,tail:CallType) -> Result<(),ErrList> {
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
		AstValue::FuncCall(call) => translate_call_raw(call,name_space,handle,tail)?,
		AstValue::Match(m) => translate_match(m,name_space,handle,tail)?,

		AstValue::Variable(id) => name_space.get(handle,*id)?,
		AstValue::BuildIn(_) => unreachable!("build in op should never be made as a value in the ast"),
		AstValue::Lambda(l) => translate_lambda(l,name_space,handle,tail)?, 
		AstValue::MatchLambda(ml) => translate_match_lambda(ml,name_space,handle,tail)?, 
	};
	Ok(())
}

fn translate_call_raw(call:&FunctionCall,name_space:&mut dyn NameSpace,handle: &mut TransHandle,tail:CallType) -> Result<(),ErrList> {
	if !matches!(call.name,FValue::BuildIn(_)) {
		handle.code.push(Operation::PushTerminator);
	}
	else {
    	#[cfg(feature = "debug_terminators")]
    	handle.code.push(Operation::PushTerminator);
	}
	
	for a in call.args.iter() {
		translate_value(a,name_space,handle,FullCall)?;
	}

	match &call.name {
		FValue::SelfRef(_) => match tail {
			TailCall => handle.code.push(Operation::CallThis),
			FullCall => {
				handle.code.push(Operation::PushThis);
				handle.code.push(Operation::Call(call.debug_span));
			}
		},	
		FValue::Name(id) => {
			name_space.get(handle,*id)?;
			match tail {
				TailCall => handle.code.push(Operation::TailCall(call.debug_span)),
				CallType::FullCall => handle.code.push(Operation::Call(call.debug_span)),
			}
		},

		FValue::FuncCall(call2) => {
			translate_call_raw(call2,name_space,handle,FullCall)?;
			match tail {
				TailCall => handle.code.push(Operation::TailCall(call.debug_span)),
				CallType::FullCall => handle.code.push(Operation::Call(call.debug_span)),
			}
		},

		FValue::Lambda(l) => {
			translate_lambda(l,name_space,handle,FullCall)?;
			match tail {
				TailCall => handle.code.push(Operation::TailCall(call.debug_span)),
				CallType::FullCall => handle.code.push(Operation::Call(call.debug_span)),
			}
		},   
		FValue::MatchLambda(ml)  => {
			translate_match_lambda(ml,name_space,handle,FullCall)?;
			match tail {
				TailCall => handle.code.push(Operation::TailCall(call.debug_span)),
				CallType::FullCall => handle.code.push(Operation::Call(call.debug_span)),
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
	}

	Ok(())	
}

fn translate_match(m:&MatchStatment,name_space:&mut dyn NameSpace,handle:&mut TransHandle,tail:CallType) -> Result<(),ErrList> {
	//first in the existing name space calculate the match value
	translate_value(&m.val,name_space,handle,FullCall)?;
	translate_match_internal(&m.arms,m.debug_span,name_space,handle,tail)
}

fn translate_match_internal(arms:&[MatchArm],span:Span,name_space:&mut dyn NameSpace,handle:&mut TransHandle,tail:CallType) -> Result<(),ErrList> {

	//then in the match scope do the rest
	let mut scope = ChildScope::new(name_space);

	let mut ans = Box::new(StaticMatch::default());
	ans.span = span;

	let in_id = handle.code.len();
	handle.code.push(Operation::PushFrom(100000000));//trap instraction

	let mut return_spots = Vec::new();

	if let Some((last, rest)) = arms.split_last() {
	    for c in rest {
	    	match &c.pattern {
	    	    MatchPattern::Literal(v) => {
	    	    	let val = literal_to_ir_value(v,handle.table);
	    	    	ans.map.insert(val,handle.code.len());
	    	    	match &c.result {
		    			MatchOut::Value(v) =>translate_value(v,&mut scope,handle,tail)?,
		    			MatchOut::Block(block) => translate_block(block,&mut scope,handle,tail)?,
		    		}

	    	    	return_spots.push(handle.code.len());
	    			handle.code.push(Operation::PushFrom(100000000));//trap instraction 
	    	    }

	    	    MatchPattern::Variable(_) => todo!(),
	    	    MatchPattern::Wildcard => {
	    	    	//todo fix this up to be a proper error type
	    	    	return Err(stacked_error("while defining match",sig_error(),span));
	    	    }
	    	}
	    }
	    
	    match &last.pattern {
	    	 MatchPattern::Literal(v) => {
	    	    	let val = literal_to_ir_value(v,handle.table);
	    	    	ans.map.insert(val,handle.code.len());
	    	    	match &last.result {
		    			MatchOut::Value(v) =>translate_value(v,&mut scope,handle,tail)?,
		    			MatchOut::Block(block) => translate_block(block,&mut scope,handle,tail)?,
		    		}
		    },
	    	MatchPattern::Variable(_) => todo!(),
	    	MatchPattern::Wildcard => {
	    		ans.default=Some(handle.code.len());
	    	
	    		match &last.result {
	    			MatchOut::Value(v) =>translate_value(v,&mut scope,handle,tail)?,
	    			MatchOut::Block(block) => translate_block(block,&mut scope,handle,tail)?,
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

fn translate_block(block:&FuncBlock,parent_name_space:&mut dyn NameSpace,handle:&mut TransHandle,tail:CallType) -> Result<(),ErrList> {
	let mut scope = ChildScope::new(parent_name_space);
	let name_space = &mut scope;

	for x in block.body.iter() {
		match &x{
			Statment::Match(m) =>{
				translate_match(m,name_space,handle,FullCall)?;
				handle.code.push(Operation::PopDump);
				handle.code.push(Operation::PushNil);
			},
			Statment::Assign(id, val) => {
				translate_value(val,name_space,handle,FullCall)?;
				name_space.set(handle,*id);
			},
			Statment::Call(call) => {
				translate_call_raw(call,name_space,handle,FullCall)?;
				handle.code.push(Operation::PopDump);
				handle.code.push(Operation::PushNil);
			}
		}
	}

	match &block.ret {
	    None =>  handle.code.push(Operation::PushNil),
	    Some(ret) => match ret{
	    	Ret::Exp(val) => {
	    		translate_value(val,name_space,handle,TailCall)?;
	    		handle.code.push(Operation::Return);

	    	},
	    	Ret::Imp(val) => translate_value(val,name_space,handle,tail)?
	    },
	}

	Ok(())
}


fn literal_to_ir_value(l: &Literal,table:&StringTable) -> IRValue<'static> {
	match l {
		Literal::Int(i) => IRValue::Int(*i),
        Literal::Float(f) => IRValue::Float(*f),
        Literal::Atom(a) => IRValue::Atom(*a),
        Literal::String(s) => IRValue::String(Arc::new(table.get_escaped_string(*s))),
        Literal::Bool(b) => IRValue::Bool(*b),
        Literal::Nil => IRValue::Nil,
	}
}

fn translate_lambda(l:&Lambda,name_space:&mut dyn NameSpace,handle:&mut TransHandle,_tail:CallType) -> Result<(),ErrList> {
	let mut scope = LambdaScope::start(name_space,&l.sig);
	let holder = translate_func(&l.sig,&l.body,&mut scope,handle.table)
		.map_err(|e| stacked_error("while defining lambda",e,l.debug_span))?;
	std::mem::drop(scope);
        
    #[cfg(feature = "debug_terminators")]
    handle.code.push(Operation::PushTerminator);

	for name in &holder.vars.names{
		name_space.get(handle,*name)?;
	}

	let maker = Box::new(FuncMaker{
		holder,
		span:l.debug_span
	});
	
	handle.code.push(Operation::CaptureClosure(maker));
	Ok(())
}

fn translate_match_lambda(ml:&MatchLambda,name_space:&mut dyn NameSpace,handle:&mut TransHandle,_tail:CallType) -> Result<(),ErrList> {
	//use the hack that 0 would NEVER be a name used by an identifier
	let mut scope = LambdaScope::start(name_space,&[0]);

	let mut vars = VarTable::default();
	let mut mut_vars = VarTable::default();
	mut_vars.add_ids(&[0]);

	let mut code = vec![Operation::PushFrom(0)];

	let mut child_handle = TransHandle{
		code:&mut code,
		vars:&mut vars,
		mut_vars:&mut mut_vars,
		table: handle.table
	};


	translate_match_internal(&ml.arms,ml.debug_span,&mut scope,&mut child_handle,TailCall)?;
	code.push(Operation::Return);
	std::mem::drop(scope);
    
    let holder = FuncHolder{
    	code: code.into(),vars,mut_vars_template:mut_vars,num_args:1
    };

    #[cfg(feature = "debug_terminators")]
    handle.code.push(Operation::PushTerminator);

	for name in &holder.vars.names{
		name_space.get(handle,*name)?;
	}

	let maker = Box::new(FuncMaker{
		holder,
		span:ml.debug_span
	});
	
	handle.code.push(Operation::CaptureClosure(maker));
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


