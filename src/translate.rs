use crate::vm::Operation;
use std::sync::Arc;
use crate::value::VarTable;
use crate::reporting::ErrList;
use ast::ast::{OuterExp,StringTable,FuncDec};

pub fn translate_program<'code>(_v:&[OuterExp],_table: &StringTable<'code>) -> Result<Arc<VarTable<'code>>,ErrList>{
	todo!()
}

pub fn translate_function<'code>(func_ast:&FuncDec,table: &StringTable<'code>,global_scope:&mut VarTable<'code>,write_spot:&mut Vec<Operation<'code>>) -> Result<(),ErrList> {
	let mut mut_vars = VarTable::default();
	let mut vars = VarTable::default();

	let my_id = global_scope.len();
	global_scope.add_ids(&[func_ast.sig.name]);
	
	let func_inners = simple_load_args(
		&func_ast.sig.args,table,write_spot,
		&mut mut_vars,
		&mut vars,
	)?;
	todo!()
}


pub fn simple_load_args<'code>(
	_args: &[u32],
	_table: &StringTable<'code>,
	_write_spot:&mut Vec<Operation<'code>>,

	mut_vars:&mut VarTable<'code>,
	vars:&mut VarTable<'code>,

)-> Result<(),ErrList> {
	

	todo!()
}