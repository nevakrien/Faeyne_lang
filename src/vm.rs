
// use smallvec::SmallVec;

use std::sync::Arc;
use crate::reporting::RecursionError;
use crate::stack::StackOverflow;
use crate::value::Value;
use crate::value::VarTable;

use crate::basic_ops::handle_bin;
use crate::basic_ops::BinOp;
use crate::reporting::Error;
use crate::reporting::ErrList;
use crate::stack::ValueStack;
// use crate::value::Scope;
use ast::ast::StringTable;

use arrayvec::ArrayVec;

pub type Code<'a> = &'a [Operation];
// pub type DynFFI = dyn Fn(&mut FuncInputs) -> Result<(),ErrList>;
pub type StaticFunc = fn(&mut FuncInputs) -> Result<(),ErrList>;


#[derive(Clone,PartialEq,Debug)]
#[derive(Default)]
// #[repr(C)]
pub struct FuncData {
    pub vars: VarTable,
    pub code: Box<[Operation]>,
}

impl FuncData {
    pub fn new(vars: VarTable, code: Box<[Operation]>) -> Self {
        FuncData {
            vars,
            code,
        }
    }

}

#[test]
fn func_data() {
    let f = Arc::new(FuncData::new(VarTable::default(),Box::new([NoOp])));
    assert_eq!(f.code[0],NoOp);
    assert_eq!(f.vars,VarTable::default());
}


pub struct RetData {
    ret:usize,
    pos:usize,
    func:Arc<FuncData>,
    vars:Box<VarTable>,
}

pub struct FuncInputs<'code>{
    pub stack: ValueStack,    
    pub table: &'code StringTable<'code>,//for errors only
}

impl FuncInputs<'_, >{
    #[inline(always)]
    pub fn pop_value(&mut self) -> Option<Value> {
        self.stack.pop_value()
    }

    #[inline(always)]
    pub fn push_value(&mut self,value : Value) -> Result<(),StackOverflow> {
        self.stack.push_value(value)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
}

pub const MAX_LOCAL_SCOPES: usize = 1000;
pub const MAX_RECURSION :usize=2_500;

// #[repr(C)] //want to orgenize by importance
pub struct Context<'code> {
    // pub pos:usize,
    pos:usize,
    func:Arc<FuncData>,
    call_stack:  ArrayVec<RetData,MAX_RECURSION>,
    local_call_stack: ArrayVec<RetData,MAX_LOCAL_SCOPES>,
    vars:Box<VarTable>,
    global_vars:&'code VarTable,

    pub inputs: FuncInputs<'code>,


    
}

impl<'code> Context<'code> {
    pub fn new(
        
        table:&'code StringTable<'code>,
        func:Arc<FuncData>,
        
        // stack: &'ctx mut ValueStack,
        global_vars:&'code VarTable,
        // call_stack:  &'ctx mut ArrayVec<RetData,MAX_RECURSION>,
        // local_call_stack: &'ctx mut ArrayVec<RetData,MAX_LOCAL_SCOPES>,
        
    ) -> Self{
        let inputs = FuncInputs{stack:ValueStack::default(),table};
        Context{
            pos:0,vars:Box::new(VarTable::default()),
            func,global_vars,
            inputs,call_stack:ArrayVec::new(),local_call_stack:ArrayVec::new(),
        }
    }


    fn pop_to(&mut self,id:u32) -> Result<(),ErrList>{
        match self.inputs.pop_value(){
            Some(x) => self.vars.set(id as usize,x)
                .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
            None  => Err(Error::Bug("over poping").to_list()),
        }
    }

    fn push_from(&mut self,id:u32) -> Result<(),ErrList>{
        let value = self.vars.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.inputs.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    fn push_const(&mut self,id:u32) -> Result<(),ErrList>{
        let value = self.global_vars.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.inputs.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.vars.names.iter()
        .map(|id| self.inputs.table.get_raw_str(*id))
        .collect()
    } 

    fn big_ret(&mut self) -> Result<(),ErrList> {
        let ret_data = self.call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
        let value = self.inputs.pop_value().ok_or_else(|| Error::Bug("over pop value stack").to_list())?;

        assert!(self.inputs.stack.len()>=ret_data.ret);
        while self.inputs.stack.len()>ret_data.ret {
            self.inputs.pop_value().ok_or_else(|| Error::Bug("impossible").to_list())?;
        }
        assert!(self.inputs.stack.len()==ret_data.ret);


        self.inputs.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        
        self.func = ret_data.func.clone();
        self.pos = ret_data.pos;

        self.local_call_stack.clear();
        self.vars=ret_data.vars;

        Ok(())
    }

    fn small_ret(&mut self) -> Result<(),ErrList> {
        let ret_data = self.local_call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
        let value = self.inputs.pop_value().ok_or_else(|| Error::Bug("over pop value stack").to_list())?;

        assert!(self.inputs.stack.len()>=ret_data.ret);
        while self.inputs.stack.len()>ret_data.ret {
            self.inputs.pop_value().ok_or_else(|| Error::Bug("impossible").to_list())?;
        }
        assert!(self.inputs.stack.len()==ret_data.ret);
        
        self.func = ret_data.func.clone();
        self.pos = ret_data.pos;

        self.inputs.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        self.vars=ret_data.vars;

        Ok(())
    }

    fn call(&mut self) -> Result<(),ErrList> {
        let called = self.inputs.pop_value()
            .ok_or_else(||{Error::Bug("over poping").to_list()}
            )?;

        let func = match called {
            Value::Func(f) => f,
            Value::WeakFunc(wf) => wf.upgrade().ok_or_else(||{Error::Bug("weak function failed to upgrade").to_list()}
            )?,
            _ => todo!()

        };

        let mut new_vars = Box::new(func.vars.clone());

        std::mem::swap(&mut self.vars,&mut new_vars);

        let ret = RetData{
            ret:self.inputs.stack.len(),
            func:self.func.clone(),
            pos:self.pos,
            vars:new_vars,
        };

        self.call_stack.try_push(ret)
            .map_err(|_| Error::Recursion(
                RecursionError{depth:MAX_RECURSION}
            ).to_list())?;

        todo!()
    }
    
    fn handle_op(&mut self,op:Operation) -> Result<(),ErrList> {
        match op {
            BinOp(b) => handle_bin(&mut self.inputs,b),
            PopTo(id) => self.pop_to(id),
            PushFrom(id) => self.push_from(id),
            PushConst(id) => self.push_const(id),

            RetSmall => self.small_ret(),
            RetBig => self.big_ret(),
            Call => self.call(),

            _ => todo!(),
        }
    }

    //returns true if we should keep going
    pub fn next_op(&mut self) -> Result<bool,ErrList>{
        if self.pos>=self.func.code.len() {return Ok(false);}
        let op = self.func.code[self.pos];
        self.pos+=1;
        self.handle_op(op).map(|_| true)
    }

}



#[derive(Debug,PartialEq,Clone,Copy)]
#[repr(u32)]
pub enum Operation {
    //type ids end at 8 so we take a safe distance from them to maje the try_from fail on most UB

    Call = 16,//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
    RetBig,//returns out of the function scope. 
    RetSmall,//returns out of a match block

    PopTo(u32),
    PushFrom(u32),
    PushConst(u32),

    BinOp(BinOp),

    Match,//pops a value to match aginst then returns the result (not quite sure about the detais)
    CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
    NoOp,
}

use Operation::*;


#[test]
fn test_vm_push_pop() {
    // Step 1: Setup the StringTable
    let mut string_table = StringTable::new();
    let atom_a_id = string_table.get_id(":a");
    let atom_b_id = string_table.get_id(":b");

    let a_id = string_table.get_id("var_a");
    let b_id = string_table.get_id("var_b");

    //make function
    let vars = VarTable::default();
    let code = vec![
        Operation::PushConst(0),
        Operation::PushConst(1),
    ]
    .into_boxed_slice(); // Box the slice for FuncData

    let func_data = Arc::new(FuncData::new(vars, code));

    //global vars
    let mut global_vars = VarTable::default();
    global_vars.add_ids(&[a_id, b_id]);
    global_vars.set(0, Value::Atom(atom_a_id)).unwrap(); 
    global_vars.set(1, Value::Atom(atom_b_id)).unwrap();

    let mut context = Context::new(&string_table, func_data.clone(), &global_vars);

    // Step 5: Execute operations using next_op
    let mut keep_running = true;
    while keep_running {
        keep_running = context.next_op().unwrap();
    }

    // Step 6: Assert the results
    // After executing PushConst(atom_a_id), PushConst(atom_b_id), BinOp::Add, the result should be 15 on the stack
    let result = context.inputs.stack.pop_value().unwrap();
    assert_eq!(result, Value::Atom(atom_b_id));
}


