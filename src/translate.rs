#![allow(unused_variables)]

use std::collections::HashMap;
use crate::vm::Operation;
use std::sync::Arc;
use crate::value::VarTable;
use crate::reporting::ErrList;
use ast::ast::{OuterExp,StringTable,FuncDec};

pub enum Var {
	Global(usize),
	Local(usize),
	Mut(usize),
}

impl Var {
	pub fn get_push(&self) -> Operation {
		match self{
			Var::Global(id) => Operation::PushGlobal(*id),
			Var::Local(id) => Operation::PushLocal(*id),
			Var::Mut(id) => Operation::PushFrom(*id),
		}
	} 
}

pub struct GlobalData<'code> {
	pub vars : Arc<VarTable<'code>>,
	pub code: Vec<Operation<'code>>,
}

pub fn translate_program<'code>(code:&[OuterExp],table: &StringTable<'code>) -> Result<GlobalData<'code>,ErrList>{	
	//find all avilble global names first

	let mut func_decs_ids = Vec::new();
	let mut impport_decs_ids = Vec::new();

	for exp in code {
		match exp {
			OuterExp::FuncDec(func) => func_decs_ids.push(func.sig.name),
			OuterExp::ImportFunc(func) => impport_decs_ids.push(func.name),
		}
	}

	let mut global_scope_base = Arc::new(VarTable::default());
	let global_scope = Arc::get_mut(&mut global_scope_base).unwrap();

	global_scope.add_ids(&func_decs_ids);
	global_scope.add_ids(&impport_decs_ids);

	let mut resolve_table :HashMap<u32,Var> = HashMap::new();
	for (id,name) in global_scope.names.iter().enumerate() {
		resolve_table.insert(*name,Var::Global(id));
	}

	let mut byte_code = Vec::new();

	for exp in code {
		match exp {
			OuterExp::FuncDec(func) => translate_function(func,table,
				global_scope,&resolve_table,&mut byte_code)?,
			OuterExp::ImportFunc(_) => todo!(),
		};
	}

	Ok(GlobalData{
		vars:global_scope_base,
		code: byte_code
	})
}

pub fn translate_function<'code>(
	func_ast:&FuncDec,
	table: &StringTable<'code>,
	global_scope:&mut VarTable<'code>,
	resolve_table: &HashMap<u32,Var>,
	write_spot:&mut Vec<Operation<'code>>
) -> Result<(),ErrList> {
	let mut mut_vars = VarTable::default();
	let mut vars = VarTable::default();

	let my_id = resolve_table.get(&func_ast.sig.name).unwrap();

	simple_load_args(
		&func_ast.sig.args,table,write_spot,
		&mut mut_vars,
		&mut vars,
	);

	todo!()
}


fn simple_load_args<'code>(
	args: &[u32],
	_table: &StringTable<'code>,
	write_spot:&mut Vec<Operation<'code>>,

	mut_vars:&mut VarTable<'code>,
	vars:&mut VarTable<'code>,
){
	let last_arg_id = mut_vars.len();
	mut_vars.add_ids(args);

	for i in (last_arg_id..mut_vars.len()).rev() {
		write_spot.push(Operation::PopArgTo(i));
	}

}

// pub fn translate_statment<'code> {

// } 