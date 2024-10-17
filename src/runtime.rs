// #![allow(unused_imports)]
use crate::reporting::overflow_error;
use std::sync::RwLock;
use crate::reporting::missing_func_error;
use std::collections::HashMap;
use ast::ast::StringTable;
use crate::reporting::ErrList;
use std::sync::Arc;
use crate::vm::FuncData;
use crate::value::VarTable;
use crate::value::Value as IRValue;
use crate::vm::{Operation,Context};

pub struct Code<'a> {
	pub names: Vec<u32>,
	pub func : Vec<FuncHolder<'a>>,
	pub name_map: HashMap<Box<str>,usize>,
    pub table:Arc<RwLock<StringTable<'a>>>,

}

#[derive(Clone,PartialEq,Debug)]
pub struct FuncHolder<'a> {
    pub mut_vars_template: VarTable<'a>,
    pub vars: VarTable<'a>,
    pub code: Box<[Operation]>,
}

#[derive(Debug,PartialEq)]
pub enum RunError {
	Exc(ErrList),
	Sync
}

impl From<ErrList> for RunError {

	fn from(e: ErrList) -> Self { RunError::Exc(e) }
}

impl Code<'_> {
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

	pub fn run<'a>(&self,name:&str,values:impl IntoIterator<Item = IRValue<'a>>) -> Result<(),RunError> {
		let func = *self.name_map.get(name).ok_or_else(|| missing_func_error(name.to_string()))?;
		let table = &*self.table.try_read().map_err(|_| {RunError::Sync})?;


		let global = self.get_global();
		let Some(IRValue::Func(main)) = global.get(func) else { todo!() };

		let mut context = Context::new(main,&global,table);
		for v in values {
			context.stack.push_value(v).map_err(|_| overflow_error())?;
		}
		let _ = context.run()?;
		Ok(())
	}

	pub fn run_compare<'a>(&self,name:&str,values:impl IntoIterator<Item = IRValue<'a>>,value:IRValue) -> Result<bool,RunError> {
		let func = *self.name_map.get(name).ok_or_else(|| missing_func_error(name.to_string()))?;
		let table = &*self.table.try_read().map_err(|_| {RunError::Sync})?;

		
		let global = self.get_global();
		let Some(IRValue::Func(main)) = global.get(func) else { todo!() };
		let mut context = Context::new(main,&global,table);
		for v in values {
			context.stack.push_value(v).map_err(|_| overflow_error())?;
		}
		let x = context.run()?;
		Ok(x==value)
	}

	pub fn run_map<'a, T,F:FnOnce(IRValue) -> T>(&self,name:&str,values:impl IntoIterator<Item = IRValue<'a>>,map:F) -> Result<T,RunError> {
		let func = *self.name_map.get(name).ok_or_else(|| missing_func_error(name.to_string()))?;
		let table = &*self.table.try_read().map_err(|_| {RunError::Sync})?;
		

		let global = self.get_global();
		let Some(IRValue::Func(main)) = global.get(func) else { todo!() };
		let mut context = Context::new(main,&global,table);
		for v in values {
			context.stack.push_value(v).map_err(|_| overflow_error())?;
		}
		let x = context.run()?;

		Ok(map(x))
	}
}

#[test]
fn test_unified_code_runs() {

    // Step 1: Setup the StringTable
    let mut string_table = StringTable::new();
    let var_a_id = string_table.get_id("var_a");

    // Step 2: Setup the VarTable for the function
    let mut mut_vars = VarTable::default();
    mut_vars.add_ids(&[var_a_id]);

    // Step 3: Create the code for the function
    let code = vec![
        Operation::PushBool(true), // Push true before returning
        Operation::Return,         // Return from the function
    ].into_boxed_slice();

    // Step 4: Create the FuncHolder and Code structs
    let func_holder = FuncHolder {
        mut_vars_template: mut_vars.clone(),
        vars: VarTable::default(),
        code,
    };

    let mut name_map = HashMap::new();
    name_map.insert(Box::from("bool_func"), 0); // Function for all tests

    let table = Arc::new(RwLock::new(string_table));

    let code_struct = Code {
        names: vec![var_a_id],
        func: vec![func_holder],
        name_map,
        table,
    };

    // Test 1: Using `run` method
    let result = code_struct.run("bool_func",vec![]);
    assert!(result.is_ok(), "Expected successful run with no errors");

    // Test 2: Using `run_compare` method to compare with true (should pass)
    let result = code_struct.run_compare("bool_func",vec![], IRValue::Bool(true));
    assert_eq!(result.unwrap(), true, "Expected true from run_compare");

    // Test 3: Using `run_compare` method to compare with false (should fail)
    let result = code_struct.run_compare("bool_func",vec![], IRValue::Bool(false));
    assert_eq!(result.unwrap(), false, "Expected false from run_compare");

    // Test 4: Using `run_map` method to map output to a string
    let result = code_struct.run_map("bool_func",vec![], |val| match val {
        IRValue::Bool(true) => "True",
        _ => "False",
    });
    assert_eq!(result.unwrap(), "True", "Expected result to be 'True' from run_map");
}

#[test]
fn test_all_errors_in_one_code() {
    // Step 1: Setup the StringTable
    let mut string_table = StringTable::new();
    let var_a_id = string_table.get_id("var_a");

    // Step 2: Setup the VarTable for the function that will trigger an internal error
    let mut mut_vars = VarTable::default();
    mut_vars.add_ids(&[var_a_id]);

    // Step 3: Create two sets of operation codes
    // Function 1: PopTo will trigger an internal error since the stack is empty
    let internal_error_code = vec![
        Operation::PopTo(0),  // Will cause an error as the stack is empty
        Operation::Return,
    ].into_boxed_slice();

    // Function 2: A simple return function that should work fine
    let simple_code = vec![
        Operation::PushBool(true),
        Operation::Return,
    ].into_boxed_slice();

    // Step 4: Create FuncHolders for the two functions
    let internal_error_func = FuncHolder {
        mut_vars_template: mut_vars.clone(),
        vars: VarTable::default(),
        code: internal_error_code,
    };

    let simple_func = FuncHolder {
        mut_vars_template: mut_vars.clone(),
        vars: VarTable::default(),
        code: simple_code,
    };

    // Step 5: Create a name map and add two functions to it
    let mut name_map = HashMap::new();
    name_map.insert(Box::from("simple_func"), 0);
    name_map.insert(Box::from("internal_error_func"), 1);

    // Step 6: Create the Code struct with both functions
    let table = Arc::new(RwLock::new(string_table));

    let code_struct = Code {
        names: vec![var_a_id],
        func: vec![simple_func, internal_error_func],
        name_map,
        table: table.clone(),
    };

    // Test 1: Missing Function Error
    let result = code_struct.run("missing_func",vec![]);
    assert!(matches!(result, Err(RunError::Exc(_))), "Expected missing function error");

    // Test 2: Internal Function Error (PopTo on empty stack)
    let result = code_struct.run("internal_error_func",vec![]);
    assert!(matches!(result, Err(RunError::Exc(_))), "Expected internal function error due to empty stack");

    // Test 3: Sync Error - Lock the table in write mode and try running a function
    let write_lock = table.write().unwrap();  // Hold the lock to simulate a sync error
    let result = code_struct.run("simple_func",vec![]);
    assert!(matches!(result, Err(RunError::Sync)), "Expected sync error due to locked table");

    // Ensure the lock is released after the test
    drop(write_lock);
}
