
// use smallvec::SmallVec;

use codespan::Span;
use std::collections::HashMap;
use std::sync::Arc;
use crate::reporting::RecursionError;
use crate::value::Value;
use crate::value::VarTable;

use crate::basic_ops::handle_bin;
use crate::basic_ops::BinOp;


use crate::reporting::{Error,MatchError,ErrList};
use crate::stack::ValueStack;
// use crate::value::Scope;
use ast::ast::StringTable;

use arrayvec::ArrayVec;


// pub type Code<'code> = &'code [Operation];
// pub type DynFFI = dyn Fn(&mut FuncInputs) -> Result<(),ErrList>;
pub type StaticFunc<'code> = fn(&mut ValueStack,&StringTable<'code>) -> Result<(),ErrList>;


#[derive(Clone,PartialEq,Debug)]
#[derive(Default)]
// #[repr(C)]
pub struct FuncData<'code> {
    pub mut_vars: VarTable<'code>,
    pub vars: VarTable<'code>,
    pub code: &'code [Operation<'code>],
    pub span: Span
}

impl<'code> FuncData<'code> {
    pub fn new(vars: VarTable<'code>,mut_vars: VarTable<'code>, code: &'code [Operation<'code>],span:Span) -> Self {
        FuncData {
            vars,
            mut_vars,
            code,
            span,
        }
    }

}



pub struct RetData<'code> {
    ret:usize,
    pos:usize,
    func:Arc<FuncData<'code>>,
    mut_vars:Box<VarTable<'code>>,
}

pub const MAX_RECURSION :usize=2_500;

// #[repr(C)] //want to orgenize by importance
pub struct Context<'code> {
    // pub pos:usize,
    pos:usize,
    func:Arc<FuncData<'code>>,
    call_stack:  ArrayVec<RetData<'code>,MAX_RECURSION>,
    mut_vars:Box<VarTable<'code>>,
    global_vars:&'code VarTable<'code>,

    // pub inputs: FuncInputs<'code>,
    pub stack: ValueStack<'code>,    
    pub table: &'code StringTable<'code>,//for errors only


    
}

impl<'code> Context<'code> {
    pub fn new(
        
        func:Arc<FuncData<'code>>,
        
        // stack: &'ctx mut ValueStack,
        global_vars:&'code VarTable<'code>,
        table: &'code StringTable<'code>,//for errors only

        // call_stack:  &'ctx mut ArrayVec<RetData,MAX_RECURSION>,
        // local_call_stack: &'ctx mut ArrayVec<RetData,MAX_LOCAL_SCOPES>,
        
    ) -> Self{
        // let inputs = FuncInputs{stack:ValueStack::default(),table};
        Context{
            pos:0,mut_vars:Box::new(VarTable::default()),
            stack:ValueStack::default(),
            func,global_vars,
            table,call_stack:ArrayVec::new(),
        }
    }


    fn pop_to(&mut self,id:usize) -> Result<(),ErrList>{
        match self.stack.pop_value(){
            Some(x) => self.mut_vars.set(id as usize,x)
                .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
            None  => Err(Error::Bug("over poping").to_list()),
        }
    }

    fn push_from(&mut self,id:usize) -> Result<(),ErrList>{
        let value = self.mut_vars.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    fn push_const(&mut self,id:usize) -> Result<(),ErrList>{
        let value = self.global_vars.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    fn push_closure(&mut self,id:usize) -> Result<(),ErrList>{
        let value = self.func.vars.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.mut_vars.names.iter()
        .chain(self.func.vars.names.iter())
        .chain(self.global_vars.names.iter())
        .map(|id| self.table.get_raw_str(*id))
        .collect()
    } 

    fn big_ret(&mut self) -> Result<(),ErrList> {
        let ret_data = self.call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
        let value = self.stack.pop_value().ok_or_else(|| Error::Bug("over pop value stack").to_list())?;

        assert!(self.stack.len()>=ret_data.ret);
        while self.stack.len()>ret_data.ret {
            self.stack.pop_value().ok_or_else(|| Error::Bug("impossible").to_list())?;
        }
        assert!(self.stack.len()==ret_data.ret);


        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        
        self.func = ret_data.func.clone();
        self.pos = ret_data.pos;

        self.mut_vars=ret_data.mut_vars;

        Ok(())
    }

    

    fn call(&mut self) -> Result<(),ErrList> {
        //get function code

        let called = self.stack.pop_value()
            .ok_or_else(||{Error::Bug("over poping").to_list()}
            )?;

        let func = match called {
            Value::Func(f) => f,
            Value::WeakFunc(wf) => wf.upgrade().ok_or_else(||{Error::Bug("weak function failed to upgrade").to_list()}
            )?,
            _ => todo!()

        };

        let mut new_vars = Box::new(func.mut_vars.clone());

        //set up return 

        std::mem::swap(&mut self.mut_vars,&mut new_vars);

        let ret = RetData{
            ret:self.stack.len(),
            func:self.func.clone(),
            pos:self.pos,
            mut_vars:new_vars,
        };

        self.call_stack.try_push(ret)
            .map_err(|_| Error::Recursion(
                RecursionError{depth:MAX_RECURSION}
            ).to_list())?;

        todo!()
    }

    fn tail_call(&mut self) -> Result<(),ErrList> {

        //get function

        let called = self.stack.pop_value()
            .ok_or_else(||{Error::Bug("over poping").to_list()}
            )?;

        let func = match called {
            Value::Func(f) => f,
            Value::WeakFunc(wf) => wf.upgrade().ok_or_else(||{Error::Bug("weak function failed to upgrade").to_list()}
            )?,
            _ => todo!()

        };

        self.mut_vars = Box::new(func.mut_vars.clone());


        todo!()
    }
        
    fn match_jump(&mut self,map:&StaticMatch<'code>) -> Result<(),ErrList> {
        let x = self.stack.pop_value()
            .ok_or_else(||{Error::Bug("over poping").to_list()}
            )?;

        let jump_pos = map.get(&x)
            .ok_or_else(||{Error::Match(MatchError{span:map.span}).to_list()}
            )?;
        
        self.pos=jump_pos;
        Ok(())
    }
    
    fn handle_op(&mut self,op:Operation<'code>) -> Result<(),ErrList> {
        match op {
            NoOp => Ok(()),

            BinOp(b) => handle_bin(&mut self.stack,self.table,b),
            PopTo(id) => self.pop_to(id),
            PushFrom(id) => self.push_from(id),
            PushConst(id) => self.push_const(id),
            PushClosure(id) => self.push_closure(id),

            Return => self.big_ret(),
            Call => self.call(),
            TailCall => self.tail_call(),
            MatchJump(map) => self.match_jump(map),
            Jump(pos) => {
                self.pos=pos;
                Ok(())
            },
            
            Operation::CaptureClosure => todo!(),

            Operation::PushBool(b) => self.stack.push_bool(b)
                .map_err(|_| Error::StackOverflow.to_list()),
            
            Operation::PushAtom(a) => self.stack.push_atom(a)
                .map_err(|_| Error::StackOverflow.to_list()),
            
            Operation::PushNil => self.stack.push_nil()
                .map_err(|_| Error::StackOverflow.to_list()),

            // _ => todo!(),
        }
    }

    //returns true if we should keep going
    pub fn next_op(&mut self) -> Result<bool,ErrList>{
        if self.pos>=self.func.code.len() {return Ok(false);}
        let op = self.func.code[self.pos];
        self.pos+=1;
        self.handle_op(op).map(|_| true)
    }

    pub fn run(&mut self) -> Result<Value<'code>,ErrList> {
        let mut keep_running = true;
        while keep_running {
            keep_running = self.next_op()?;
        }
        self.stack.pop_value()
            .ok_or_else(||{Error::Bug("over poping").to_list()}
            )
    }

}


#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Operation<'code> {

    Call,//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
    TailCall,//similar to call but does not push its own vars. instead it drops
    Return,//returns out of the function scope. 

    PopTo(usize),
    PushFrom(usize),
    PushConst(usize),
    PushClosure(usize),

    PushBool(bool),
    PushAtom(u32),
    PushNil,

    BinOp(BinOp),

    MatchJump(&'code StaticMatch<'code>),//pops a value to match aginst then jumps to a position based on it
    Jump(usize), //jumps to a position usually outside of a match case
    
    //basic match pattern is similar to ifs in assembly
    // jmp (table) -> [code to push value | Jump to end]

    CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
    NoOp,
}

use Operation::*;

#[derive(PartialEq,Debug,Clone)]
#[repr(C)] //static match should probably hold the map first because its acessed first
pub struct StaticMatch<'code> {
    //note that offsets are not from the start of the code 
    pub map: HashMap<Value<'code>,usize>,
    pub default: Option<usize>,
    pub span: Span
}


impl<'a> StaticMatch<'a> {
    pub fn get(&self,value:&Value<'a>) -> Option<usize> {
        match self.map.get(value) {
            Some(id) => Some(*id),
            None => self.default
        }
    }
}

#[test]
fn test_vm_push_pop() {
    // Step 1: Setup the StringTable
    let mut string_table = StringTable::new();
    let atom_a_id = string_table.get_id(":a");
    let atom_b_id = string_table.get_id(":b");

    let a_id = string_table.get_id("var_a");
    let b_id = string_table.get_id("var_b");

    //make function
    let mut_vars = VarTable::default();

    let mut vars = VarTable::default();
    vars.add_ids(&[a_id, b_id]);
    vars.set(0, Value::Atom(atom_a_id)).unwrap(); 
    vars.set(1, Value::Atom(atom_b_id)).unwrap();

    let code = vec![
        Operation::PushConst(0),
        Operation::PushConst(1),
        Operation::PushClosure(1),
    ]
    .into_boxed_slice(); // Box the slice for FuncData

    let func_data = Arc::new(FuncData::new(vars,mut_vars, &code,Span::default()));

    //global vars
    let mut global_vars = VarTable::default();
    global_vars.add_ids(&[a_id, b_id]);
    global_vars.set(0, Value::Atom(atom_a_id)).unwrap(); 
    global_vars.set(1, Value::Atom(atom_b_id)).unwrap();

    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);



    // After executing PushConst(atom_a_id), PushConst(atom_b_id), BinOp::Add, the result should be 15 on the stack
    let result = context.run().unwrap();
    assert_eq!(result, Value::Atom(atom_b_id));
}


// #[test]
// fn test_invalid_lifetime_with_context() {
//     // Setup the StringTable and create a global context
//     let mut string_table = StringTable::new();
//     let atom_local_id = string_table.get_id(":local_atom");

//     let func_data = Arc::new(FuncData::new(VarTable::default(), &[]));
//     let global_vars = VarTable::default();
//     let mut context = Context::new(func_data.clone(), &global_vars, &string_table);

//     // Create a local context with a shorter lifetime and push a value from it
//     {
//         let a = [NoOp];
//         let dead_func = Arc::new(FuncData::new(VarTable::default(), &a));
//         let mut local_context = Context::new(dead_func.clone(), &global_vars, &string_table);
        
//         let local_value = Value::Atom(atom_local_id);
//         local_context.stack.push_value(local_value).unwrap(); // Invalid push
//     } // `local_value` goes out of scope here, making it invalid

//     // Attempt to pop the value, which should be a dangling reference
//     let result = context.stack.pop_value().unwrap();
//     println!("{:?}", result);
// }

#[test]
fn test_not_gate_match() {
    let mut string_table = StringTable::new();

    let mut match_map = HashMap::new();
    match_map.insert(Value::Bool(false), 3); // false => true
    match_map.insert(Value::Bool(true), 1);  // true => false

    // Span for reporting errors (dummy span for now)
    let dummy_span = Span::default();

    let not_gate_match = StaticMatch {
        map: match_map,
        default: None, // No default behavior, should throw an error on a match failure
        span: dummy_span,
    };

    // Global variables (holding `true` and `false` as booleans)
    let mut global_vars = VarTable::default();
    global_vars.add_ids(&[0]); // No IDs needed in this case

    // Function that just has a MatchJump with the NOT gate
    let code = vec![
        Operation::MatchJump(&not_gate_match), // Perform the NOT operation
        Operation::PushBool(false),      // Push `false` (id 0 in this case)
        Operation::Jump(4),
        Operation::PushBool(true),      // Push `true` (id 1 as the result of NOT false)
    ]
    .into_boxed_slice();

    let mut_vars = VarTable::default();
    let vars = VarTable::default();

    let func_data = Arc::new(FuncData::new(vars, mut_vars, &code,dummy_span));

    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);

    // Step 5: Assert the results (false => true)
    context.stack.push_bool(false).unwrap();
    let result = context.run().unwrap();
    assert_eq!(result, Value::Bool(true)); // Should return true

    context.stack.push_bool(true).unwrap();
    let result = context.run().unwrap();
    assert_eq!(result, Value::Bool(false)); // Should return true

    // Test Match Error
    let mut error_context = Context::new(func_data.clone(), &global_vars, &string_table);
    error_context.stack.push_value(Value::Int(1)).unwrap(); // Invalid value for matching
    let match_result = error_context.match_jump(&not_gate_match);
    assert!(match_result.is_err()); // Should return a match error
}
