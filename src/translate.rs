// // #![allow(unused_variables)]

// use std::collections::HashMap;
// use crate::vm::Operation;
// use std::sync::Arc;
// use crate::value::VarTable;
// use crate::reporting::ErrList;
// use ast::ast::{OuterExp,StringTable,FuncDec};

// pub enum Var {
// 	Global(usize),
// 	Local(usize),
// 	Mut(usize),
// }

// impl Var {
// 	pub fn get_push(&self) -> Operation {
// 		match self{
// 			Var::Global(id) => Operation::PushGlobal(*id),
// 			Var::Local(id) => Operation::PushLocal(*id),
// 			Var::Mut(id) => Operation::PushFrom(*id),
// 		}
// 	} 
// }

// // struct VarScope<'a> {
// // 	map:HashMap<u32,Var>,
// // 	parent:Result<&'a VarScope<'a>,&'a HashMap<u32,Var>>,
// // }

// // impl<'a> VarScope<'a> {
// // 	fn new() -> Self{
// // 		VarScope{map:HashMap::new(),parent:None}
// // 	}

// // 	fn make_subscope(&'a self) -> Self {
// // 		VarScope{map:HashMap::new(),parent:Some(self)}

// // 	}

// // 	fn insert(&mut self,id:u32,var:Var) {
// // 		self.map.insert(id,var);
// // 	}

// // 	fn get(&self,id:&u32) -> Option<&Var> {
// // 		self.map.get(&id)
// // 	}
// // }

// pub struct GlobalData<'code> {
// 	pub global_vars : Arc<VarTable<'code>>,
// 	pub code: Vec<Operation<'code>>,
// 	pub vars_storage: Vec<VarTable<'code>>,
// }

// struct PreData<'code> {
// 	id: usize,
// 	vars : VarTable<'code>,
// 	mut_vars : VarTable<'code>,
// 	code_region : (usize,usize),
// }

// pub fn translate_program<'code>(code:&[OuterExp],table: &StringTable<'code>) -> Result<GlobalData<'code>,ErrList>{	
// 	//find all avilble global names first

// 	let mut func_decs_ids = Vec::new();
// 	let mut impport_decs_ids = Vec::new();

// 	for exp in code {
// 		match exp {
// 			OuterExp::FuncDec(func) => func_decs_ids.push(func.sig.name),
// 			OuterExp::ImportFunc(func) => impport_decs_ids.push(func.name),
// 		}
// 	}

// 	let mut global_scope_base = Arc::new(VarTable::default());
// 	let global_scope = Arc::get_mut(&mut global_scope_base).unwrap();

// 	global_scope.add_ids(&func_decs_ids);
// 	global_scope.add_ids(&impport_decs_ids);

// 	let mut resolve_table  = HashMap::new();
// 	for (id,name) in global_scope.names.iter().enumerate() {
// 		resolve_table.insert(*name,id);
// 	}

// 	let mut byte_code = Vec::new();
// 	let mut pre_data = Vec::new();

// 	for exp in code {
// 		match exp {
// 			OuterExp::FuncDec(func) => {
// 				let start = byte_code.len();

// 				let vars = translate_function(
// 					func,table,
// 					&resolve_table,
// 					&mut byte_code
// 				)?;

// 				let code_region = (start,byte_code.len());
// 				let id = resolve_table.get(&func.sig.name).unwrap();
// 				pre_data.push(PreData{code_region,vars,id: *id});
// 			},
// 			OuterExp::ImportFunc(_) => todo!(),
// 		};
// 	}

// 	for data in pre_data {
// 		let func = FuncData{

// 		}
// 		global_scope.set(data.id);
// 	}

// 	Ok(GlobalData{
// 		vars:global_scope_base,
// 		code: byte_code
// 	})
// }

// pub fn translate_function<'code>(
// 	func_ast:&FuncDec,
// 	table: &StringTable<'code>,
// 	// global_scope:&mut VarTable<'code>,
// 	resolve_table: &HashMap<u32,usize>,
// 	write_spot:&mut Vec<Operation<'code>>
// ) -> Result<VarTable<'code>,ErrList> {
// 	let mut mut_vars = VarTable::default();
// 	let mut vars = VarTable::default();

// 	// let my_id = resolve_table.get(&func_ast.sig.name).unwrap();

// 	simple_load_args(
// 		&func_ast.sig.args,table,write_spot,
// 		&mut mut_vars,
// 		// &mut vars,
// 	);



// 	Ok(vars)
// }


// fn simple_load_args<'code>(
// 	args: &[u32],
// 	_table: &StringTable<'code>,
// 	write_spot:&mut Vec<Operation<'code>>,

// 	mut_vars:&mut VarTable<'code>,
// 	// vars:&mut VarTable<'code>,
// ){
// 	let last_arg_id = mut_vars.len();
// 	mut_vars.add_ids(args);

// 	for i in (last_arg_id..mut_vars.len()).rev() {
// 		write_spot.push(Operation::PopArgTo(i));
// 	}

// }

// // pub fn translate_statment<'code> {

// // } 