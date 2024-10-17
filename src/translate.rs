// #![allow(unused_imports)]
use ast::ast::StringTable;
use crate::reporting::ErrList;
use std::sync::Arc;
use crate::vm::FuncData;
use crate::value::VarTable;
use crate::value::Value as IRValue;
use crate::vm::{Operation,Context};

pub struct Code {
	pub names: Vec<u32>,
	pub func : Vec<FuncHolder>,
	pub main_id : usize,
}

#[derive(Clone,PartialEq,Debug)]
pub struct FuncHolder {
    pub mut_vars_template: VarTable<'static>,
    pub vars: VarTable<'static>,
    pub code: Box<[Operation]>,
}


impl Code {
	pub fn get_global<'code>(&'code self) -> VarTable<'code>{
		let mut data = Vec::with_capacity(self.names.len());
		for f in self.func.iter() {
			let function = Arc::new(FuncData::new(
            &f.vars,f.mut_vars_template.clone(),&f.code //very happy this works not sure why it works tho...
            ));
			data.push(Some(IRValue::Func(function)))
		}

		VarTable{data,names:self.names.clone()}
	}

	pub fn run_main(&self,table:&StringTable) -> Result<(),ErrList> {
		let global = self.get_global();
		let Some(IRValue::Func(main)) = global.get(self.main_id) else { todo!() };
		let mut context = Context::new(main,&global,table);
		let _ = context.run()?;
		Ok(())
	}

	pub fn run_compare_main(&self,table:&StringTable,value:IRValue) -> Result<bool,ErrList> {
		let global = self.get_global();
		let Some(IRValue::Func(main)) = global.get(self.main_id) else { todo!() };
		let mut context = Context::new(main,&global,table);
		context.run().map(|x| x==value)
	}

	pub fn run_map_main<T,F:FnOnce(IRValue) -> T>(&self,table:&StringTable,map:F) -> Result<T,ErrList> {
		let global = self.get_global();
		let Some(IRValue::Func(main)) = global.get(self.main_id) else { todo!() };
		let mut context = Context::new(main,&global,table);
		context.run().map(map)
	}
}