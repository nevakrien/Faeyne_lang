use crate::reporting::arg_error;
use crate::runtime::FuncHolder;
use core::hash::Hasher;
use core::hash::Hash;
use core::fmt;
use crate::basic_ops::call_string;
use crate::reporting::stacked_error;
use crate::reporting::match_error;
use crate::basic_ops::non_callble_error;
use crate::reporting::recursion_error;
use crate::reporting::sig_error;
use crate::reporting::overflow_error;
use crate::reporting::bug_error;
use std::collections::LinkedList;
use crate::reporting::InternalError;

use codespan::Span;
use std::collections::HashMap;
use std::sync::Arc;
use crate::value::Value;
use crate::value::VarTable;

use crate::basic_ops;


use crate::reporting::{Error,ErrList};
use crate::stack::ValueStack;
// use crate::value::Scope;
use ast::ast::StringTable;

use arrayvec::ArrayVec;


#[cfg(test)]
use ast::id::ERR_ID;
#[cfg(test)]
use ast::get_id;

pub type DynFunc =dyn for<'code> Fn(&mut ValueStack<'code>,&StringTable<'code>) -> Result<(),ErrList>;
pub type StaticFunc = for<'code> fn(&mut ValueStack<'code>,&StringTable<'code>) -> Result<(),ErrList>;

#[derive(Clone)]
#[repr(transparent)]
pub struct DataFunc {
    pub inner: Arc<DynFunc>
}

// Implement Debug for DynFunc
impl fmt::Debug for DataFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pointer = Arc::as_ptr(&self.inner); // Get the raw pointer from the Arc
        f.debug_struct("DynFunc")
            .field("pointer", &format!("{:p}", pointer)) // Format the pointer
            .finish()
    }
}

// Implement PartialEq for DataFunc (== operator)
impl PartialEq for DataFunc {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner) // Compare the raw pointers
    }
}

// Implement Eq to complete the Eq trait (since it’s reflexive)
impl Eq for DataFunc {}

impl Hash for DataFunc {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let pointer = Arc::as_ptr(&self.inner); // Get the raw pointer from the Arc
        pointer.hash(state); // Hash the pointer value
    }
}

impl From<Arc<DynFunc>> for DataFunc {

    fn from(val: Arc<(dyn for<'code> Fn(&mut ValueStack<'code>, &StringTable<'code>) -> Result<(),ErrList> + 'static)>) -> Self { DataFunc { inner: val }}
}


#[derive(Clone,PartialEq,Debug)]
// #[derive(Default)]
// #[repr(C)]
pub struct FuncData<'code> {
    pub num_args: usize,
    pub mut_vars_template: &'code VarTable<'code>,
    pub vars: VarTable<'code>,
    pub code: &'code [Operation],
}

impl<'code> FuncData<'code> {
    pub fn new(vars: VarTable<'code>,mut_vars_template: &'code VarTable<'code>, code: &'code [Operation],num_args:usize) -> FuncData<'code> {
        FuncData::<'code> {
            vars,
            mut_vars_template,
            code,
            num_args,
            // span,
        }
    }
}



#[derive(Clone,PartialEq,Debug)]
pub struct FuncMaker {
    // pub num_args : usize,
    // pub mut_vars_template: VarTable<'static>,
    // pub vars: VarTable<'static>,
    // pub code: Box<[Operation]>,
    pub holder:FuncHolder<'static>,
    pub span: Span,
}

#[derive(PartialEq,Debug,Clone,Default)]
#[repr(C)] //static match should probably hold the map first because its acessed first
pub struct StaticMatch {
    //note that offsets are not from the start of the code 
    pub map: HashMap<Value<'static>,usize>,
    pub default: Option<usize>,
    pub span: Span
}


impl StaticMatch {
    pub fn get(&self,value:&Value) -> Option<usize> {
        match self.map.get(value) {
            Some(id) => Some(*id),
            None => self.default
        }
    }
}

pub struct RetData<'code> {
    pos:usize,
    func:Arc<FuncData<'code>>,
    mut_vars:Box<VarTable<'code>>,
    pub tail_debug: TailDebug,

}

pub struct TailDebug {
    pub inner: LinkedList<(Span,bool)>
}

impl TailDebug  {
    fn new_empty() -> Self {
        TailDebug{inner:LinkedList::new()}
    }
    fn new(span:Span) -> Self {
        let mut inner = LinkedList::new();
        inner.push_front((span,false));
        TailDebug{inner}
    }

    fn push(&mut self,span:Span) {
        self.inner.push_front((span,false));
    }

    fn mark_tailed(&mut self) {
        self.inner.front_mut().unwrap().1 = true;
    }
}

pub const MAX_RECURSION :usize=2_500;

// #[repr(C)] //want to orgenize by importance
pub struct Context<'code> {
    pos:usize,
    func:Arc<FuncData<'code>>,
    call_stack:  ArrayVec<RetData<'code>,MAX_RECURSION>,
    

    mut_vars:Box<VarTable<'code>>,
    global_vars:&'code VarTable<'code>,

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
        
        let ret = RetData{
            func:func.clone(),
            pos:func.code.len(),
            mut_vars:Box::new(VarTable::default()),
            tail_debug:TailDebug::new_empty(),
        };
        let mut call_stack = ArrayVec::new();
        call_stack.push(ret);

        Context{
            pos:0,mut_vars:Box::new(func.mut_vars_template.clone()),
            stack:ValueStack::default(),
            func,global_vars,
            table,call_stack,
        }
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.mut_vars.names.iter()
        .chain(self.func.vars.names.iter())
        .chain(self.global_vars.names.iter())
        .map(|id| self.table.get_raw_str(*id))
        .collect()
    } 

    #[inline(always)]
    fn pop_to(&mut self,id:usize) -> Result<(),ErrList>{
        match self.stack.pop_value(){
            Some(x) => self.mut_vars.set(id,x)
                .map_err(|_| bug_error("tried seting a non existent id")),
            
            None  => Err(bug_error("over poping")),
        }
    }

    fn push_from(&mut self,id:usize) -> Result<(),ErrList>{
        let value = self.mut_vars.get(id)
            .ok_or_else(|| bug_error("tried seting a non existent id"))?;
        self.stack.push_value(value).map_err(|_|{overflow_error()})?;
        Ok(())
    }

    fn push_global(&mut self,id:usize) -> Result<(),ErrList>{
        let value = self.global_vars.get(id)
            .ok_or_else(|| bug_error("tried seting a non existent id"))?;
        self.stack.push_value(value).map_err(|_|{overflow_error()})?;
        Ok(())
    }

    fn push_local(&mut self,id:usize) -> Result<(),ErrList>{
        let value = self.func.vars.get(id)
            .ok_or_else(|| bug_error("tried seting a non existent id"))?;
        self.stack.push_value(value).map_err(|_|{overflow_error()})?;
        Ok(())
    }

    

    // fn pop_arg_to(&mut self,id:usize) -> Result<(),ErrList>{
    //     match self.stack.pop_value(){
    //         None  => Err(sig_error()),

    //         Some(x) => self.mut_vars.set(id,x)
    //             .map_err(|_| bug_error("tried seting a non existent arg id")
    //             ),
            
    //     }
    // }

    // #[cold]
    // fn pop_extra_arg(&mut self,num:usize) -> Result<(),ErrList>{
    //     for _ in 0..num {
    //         match self.stack.pop_value(){
    //             None  => {return Err(sig_error());},

    //             Some(_) => {},
                
    //         }
    //     }
    //     Ok(())
    // }
    

    fn big_ret(&mut self) -> Result<(),ErrList> {
        let Some(ret_data) = self.call_stack.pop() else { 
            self.pos=self.func.code.len();//we are in main and instructed to return so we are done
            return Ok(())
        };
        let value = self.stack.pop_value().ok_or_else(|| bug_error("over pop value stack"))?;


        self.stack.push_value(value).map_err(|_|{overflow_error()})?;
        
        self.func = ret_data.func.clone();
        self.pos = ret_data.pos;

        self.mut_vars=ret_data.mut_vars;

        Ok(())
    }

    #[inline]
    fn pop_function_or_run(&mut self,span:Span) -> Result<Option<Arc<FuncData<'code>>>,ErrList> {
        let called = self.stack.pop_value()
            .ok_or_else(||{bug_error("over poping while getting called")}
            )?;

        match called {
            Value::Func(f) => Ok(Some(f)),
            Value::WeakFunc(wf) => Ok(Some(wf.upgrade().ok_or_else(||{bug_error("weak function failed to upgrade")}
            )?)),
            Value::String(s) => {
                #[cfg(feature = "debug_print_vm")] 
                println!("calling string: {:?}",s);

                call_string(s,&mut self.stack,self.table,span)?;
                Ok(None)
            },
            Value::StaticFunc(extern_func) => {
                #[cfg(feature = "debug_print_vm")] 
                println!("calling extern func: {:?}",extern_func);
                
                extern_func(&mut self.stack,self.table)
                .map_err(|err| 
                    stacked_error("while calling external function",err,span)
                 )?;
                
                Ok(None)
            },
            Value::DataFunc(extern_func) => {
                #[cfg(feature = "debug_print_vm")] 
                println!("calling extern data func: {:?}",extern_func);

                (extern_func.inner)(&mut self.stack,self.table)
                .map_err(|err| 
                    stacked_error("while calling external stateful function",err,span)
                 )?;
                
                Ok(None)
            },
            _ =>{
                    Err(non_callble_error(span,&called,self.table))
                }

        }
    } 

    fn set_args(&mut self) -> Result<(),ErrList> {
        #[cfg(feature = "debug_print_vm")]{
            let names:Vec<&str> = self.func
            .mut_vars_template.names.iter()
            .map(|x| self.table.get_raw_str(*x))
            .collect();

            println!("\nloading mut_vars {:?}",names);
        }


        for i in (0.. self.func.num_args).rev() {
            let value = self.stack.pop_value().ok_or_else(|| 
                arg_error(self.func.num_args,i)
            )?;
            self.mut_vars.set(i,value).map_err(|_| bug_error("poping arg to no where"))?;
        }

        self.stack.pop_terminator().ok_or_else(|| {
            let mut extra = 0;
            while self.stack.pop_value().is_some() {extra+=1}; 
            arg_error(self.func.num_args,self.func.num_args+extra)
        })?;

        #[cfg(feature = "debug_print_vm")]
        println!("loading mut_vars {:?}",self.mut_vars.data);

        
        Ok(())
    }

    fn call(&mut self,span:Span) -> Result<(),ErrList> {
        

        let Some(func) = self.pop_function_or_run(span)? else {return Ok(())};

        let mut new_vars = Box::new(func.mut_vars_template.clone());

        //set up return 

        std::mem::swap(&mut self.mut_vars,&mut new_vars);



        let ret = RetData{
            func:self.func.clone(),
            pos:self.pos,
            mut_vars:new_vars,
            tail_debug:TailDebug::new(span)
        };

        self.call_stack.try_push(ret)
            .map_err(|_| recursion_error(MAX_RECURSION))?;

        self.func = func;
        self.pos = 0;

        self.set_args()
    }

    fn tail_call(&mut self,span:Span) -> Result<(),ErrList> {

        //get function
        let Some(func) = self.pop_function_or_run(span)? else {return Ok(())};

        // self.mut_vars = Box::new(func.mut_vars_template.clone());
        self.mut_vars.become_copy(func.mut_vars_template);

        self.call_stack.last_mut().unwrap().tail_debug.push(span);

        self.func = func;
        self.pos = 0;
        
        self.set_args()
    }

    fn call_this(&mut self) -> Result<(),ErrList> {
        self.call_stack.last_mut().unwrap().tail_debug.mark_tailed();

        // self.mut_vars = Box::new(self.func.mut_vars_template.clone());
        self.mut_vars.become_copy(self.func.mut_vars_template);
        
        self.pos = 0;
        
        self.set_args()
    }
        
    fn match_jump(&mut self,map:&StaticMatch) -> Result<(),ErrList> {
        let x = self.stack.pop_value()
            .ok_or_else(||{bug_error("over poping match")}
            )?;

        let jump_pos = map.get(&x)
            .ok_or_else(||{match_error(map.span)}
            )?;
        
        self.pos=jump_pos;
        Ok(())
    }

    fn capture_closure(&mut self,maker: &'code FuncMaker) -> Result<(),ErrList> {
        let holder = &maker.holder;
        let mut vars = holder.vars.clone();

        // for i in maker.captures.iter() {
        //     vars.set(*i,self.stack.pop_value()
        //         .ok_or_else(||{bug_error("over poping")})?)
        //         .map_err(|_|{bug_error("missing id")})?;
        // }
        for spot in vars.data.iter_mut().rev() {
            *spot = Some(self.stack.pop_value()
                .ok_or_else(||{bug_error("over poping closure close")})?);
        }

        #[cfg(feature = "debug_terminators")]
        self.stack.pop_terminator()
            .ok_or_else(||{bug_error("too many args")})?;

        let func = Value::Func(Arc::new(
            FuncData::new(
            vars,&holder.mut_vars_template,&holder.code,holder.num_args //very happy this works not sure why it works tho...
            )
        ));

        self.stack.push_value(func)
            .map_err(|_| overflow_error())
    }
    

    pub fn handle_op(&mut self,op:&'code Operation) -> Result<(),ErrList> {
        #[cfg(feature = "debug_print_vm")] 
        println!("handeling op: {:?}",op);

        match op {
            NoOp => Ok(()),

            // BinOp{op,span} => handle_bin(&mut self.stack,self.table,op,span),//probably needs span as well...
            PopTo(id) => self.pop_to(*id),
            PushFrom(id) => self.push_from(*id),
            PushGlobal(id) => self.push_global(*id),
            PushLocal(id) => self.push_local(*id),

            PopDump => {
                let _ = self.stack.pop_value().ok_or_else(|| bug_error("over poping dump"))?;
                Ok(())
            },


            Return => self.big_ret(),
            
            Call(span) => self.call(*span),
            TailCall(span) => self.tail_call(*span),
            Operation::CallThis => self.call_this(),


            MatchJump(map) => self.match_jump(map),
            Jump(pos) => {
                self.pos= * pos;
                Ok(())
            },
            
            CaptureClosure(maker) => self.capture_closure(maker),

            //some utils for small funcs
            Operation::PushThis => self.stack.push_func(self.func.clone())
                .map_err(|_| overflow_error()),

            PushBool(b) => self.stack.push_bool(*b)
                .map_err(|_| overflow_error()),
            
            PushAtom(a) => self.stack.push_atom(*a)
                .map_err(|_| overflow_error()),
            
            PushNil => self.stack.push_nil()
                .map_err(|_| overflow_error()),

            Operation::PushFloat(f) => self.stack.push_float(*f)
                .map_err(|_| overflow_error()),
            
            Operation::PushInt(i) => self.stack.push_int(*i)
                .map_err(|_| overflow_error()),

            Operation::PushString(s) => self.stack.push_string(s.clone())
                .map_err(|_| overflow_error()),

            
            //args managment
            // PopArgTo(id) => self.pop_arg_to(*id),
            // PopExtraArgs(num) => self.pop_extra_arg(*num),

            PushTerminator => self.stack.push_terminator()
                .map_err(|_| overflow_error()),
            
            PopTerminator => self.stack.pop_terminator()
                .ok_or_else(sig_error),
            
            //basic ops
            Equal(span) => basic_ops::is_equal_value(&mut self.stack,self.table,*span),
            NotEqual(span) => basic_ops::is_not_equal_value(&mut self.stack,self.table,*span),

            DoubleAnd(span) => basic_ops::logical_and(&mut self.stack,self.table,*span),
            DoubleOr(span) => basic_ops::logical_or(&mut self.stack,self.table,*span),
            DoubleXor(span) => basic_ops::logical_xor(&mut self.stack,self.table,*span),

            Smaller(span) => basic_ops::smaller(&mut self.stack,self.table,*span),
            Bigger(span) => basic_ops::bigger(&mut self.stack,self.table,*span),
            SmallerEq(span) => basic_ops::smaller_eq(&mut self.stack,self.table,*span),
            BiggerEq(span) => basic_ops::bigger_eq(&mut self.stack,self.table,*span),

            Pow(span) => basic_ops::pow(&mut self.stack,self.table,*span),
            Mul(span) => basic_ops::mul(&mut self.stack,self.table,*span),
            Sub(span) => basic_ops::sub(&mut self.stack,self.table,*span),

            Div(span) => basic_ops::div(&mut self.stack,self.table,*span),
            Modulo(span) => basic_ops::modulo(&mut self.stack,self.table,*span),
            IntDiv(span) => basic_ops::int_div(&mut self.stack,self.table,*span),
                    
            And(span) => basic_ops::bitwise_and(&mut self.stack,self.table,*span),
            Or(span) => basic_ops::bitwise_or(&mut self.stack,self.table,*span),
            Xor(span) => basic_ops::bitwise_xor(&mut self.stack,self.table,*span),
            
            Operation::Add(span) => basic_ops::add(&mut self.stack,self.table,*span),

        }.map_err(|e| self.trace_error(e))
    }

    #[cold]
    fn trace_error(&self,mut err:ErrList) -> ErrList {
        
        for ret in self.call_stack.iter().rev() {
            for (span,tailed) in &ret.tail_debug.inner {
                let err_in = InternalError{span: *span,err,message:"while calling function"};
                err = match tailed {
                    false => Error::Stacked(err_in).to_list(),
                    true => Error::StackedTail(err_in).to_list(),
                }           

            }
        }

        err
    }

    //returns true if we should keep going
    pub fn next_op(&mut self) -> Result<bool,ErrList>{
        if self.pos>=self.func.code.len() {return Ok(false);}
        let op = &self.func.code[self.pos];

        #[cfg(feature = "debug_print_vm")]
        println!("currently at pos {}",self.pos);

        self.pos+=1;

        match self.handle_op(op) {
            Ok(()) => Ok(true),
            Err(e) => Err(self.trace_error(e)),
        }
    }

    pub fn run(&mut self) -> Result<Value<'code>,ErrList> {
        self.set_args()?;

        let mut keep_running = true;
        while keep_running {
            keep_running = self.next_op()?;
        }
        self.stack.pop_value()
            .ok_or_else(||{bug_error("over poping")}
            )
    }

    pub fn reset(&mut self) {
        *self.mut_vars = self.func.mut_vars_template.clone();
        self.pos=0;
        self.stack = ValueStack::new();
    }
}

#[derive(Debug,PartialEq,Clone)]
pub enum Operation {

    Call(Span),//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
    TailCall(Span),//similar to call but does not push its own vars. instead it drops
    CallThis, //tail call!!! unlike other call methods this one does not apear in the reporting stack
    PushThis,
    Return,//returns out of the function scope. 

    PopTo(usize),
    PushFrom(usize),
    PushGlobal(usize),
    PushLocal(usize),

    PopDump,


    // PopArgTo(usize),
    // PopExtraArgs(usize),
    PushTerminator,
    PopTerminator,

    MatchJump(Box<StaticMatch>),//pops a value to match aginst then jumps to a position based on it
    Jump(usize), //jumps to a position usually outside of a match case
    
    //basic match pattern is similar to ifs in assembly
    // jmp (table) -> [code to push value | Jump to end]

    CaptureClosure(Box<FuncMaker>),//pops the data off the stack and creates a new function returning it as an IRValue to the stack
    NoOp,

    PushBool(bool),
    PushAtom(u32),
    PushNil,
    PushFloat(f64),
    PushInt(i64),
    PushString(Arc<String>),

    // BinOp{op:basic_ops::BinOp,span: Span},//too fat
    Add(Span),
    Sub(Span),
    Mul(Span),
    Div(Span),
    IntDiv(Span),
    Modulo(Span),
    Pow(Span),

    Equal(Span),
    NotEqual(Span),
    Smaller(Span),
    Bigger(Span),
    SmallerEq(Span),
    BiggerEq(Span),

    And(Span),
    Or(Span),
    Xor(Span),

    DoubleAnd(Span),
    DoubleOr(Span),
    DoubleXor(Span),
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
    let mut_vars = VarTable::default();

    let mut vars = VarTable::default();
    vars.add_ids(&[a_id, b_id]);
    vars.set(0, Value::Atom(atom_a_id)).unwrap(); 
    vars.set(1, Value::Atom(atom_b_id)).unwrap();

    let code = vec![
        Operation::PushGlobal(0),
        Operation::PushGlobal(1),
        Operation::PushLocal(1),
    ]
    .into_boxed_slice(); // Box the slice for FuncData

    let func_data = Arc::new(FuncData::new(vars,&mut_vars, &code,0));

    //global vars
    let mut global_vars = VarTable::default();
    global_vars.add_ids(&[a_id, b_id]);
    global_vars.set(0, Value::Atom(atom_a_id)).unwrap(); 
    global_vars.set(1, Value::Atom(atom_b_id)).unwrap();

    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);



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
    let string_table = StringTable::new();

    let mut match_map = HashMap::new();
    match_map.insert(Value::Bool(false), 4); // false => true
    match_map.insert(Value::Bool(true), 2);  // true => false

    // Span for reporting errors (dummy span for now)
    let dummy_span = Span::default();

    let not_gate_match = Box::new(StaticMatch {
        map: match_map,
        default: None, // No default behavior, should throw an error on a match failure
        span: dummy_span,
    });

    // Global variables (holding `true` and `false` as booleans)
    let mut global_vars = VarTable::default();
    global_vars.add_ids(&[0]); // 1 ID

    // Function that just has a MatchJump with the NOT gate
    let code = vec![
        Operation::PushFrom(0),
        Operation::MatchJump(not_gate_match.clone()), // Perform the NOT operation
        Operation::PushBool(false),      // Push `false` (id 0 in this case)
        Operation::Jump(5),
        Operation::PushBool(true),      // Push `true` (id 1 as the result of NOT false)
    ]
    .into_boxed_slice();

    let mut mut_vars = VarTable::default();
    mut_vars.add_ids(&[0]);
    let vars = VarTable::default();

    let func_data = Arc::new(FuncData::new(vars, &mut_vars, &code,1));

    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);

    // Step 5: Assert the results (false => true)
    context.stack.push_bool(false).unwrap();
    let result = context.run().unwrap();
    assert_eq!(result, Value::Bool(true)); // Should return true

    context.pos = 0;
    context.stack.push_bool(true).unwrap();
    let result = context.run().unwrap();
    assert_eq!(result, Value::Bool(false)); // Should return true

    // Test Match Error
    let mut error_context = Context::new(func_data.clone(), &global_vars, &string_table);

    error_context.stack.push_value(Value::Int(1)).unwrap(); // Invalid value for matching
    let match_result = error_context.match_jump(&not_gate_match);
    assert!(match_result.is_err()); // Should return a match error
}

#[test]
fn test_string_match() {
    let mut string_table = StringTable::new();

    // Create two different `Arc` instances holding the same string content
    let arc_str1 = Arc::new("match_string".to_string());
    let arc_str2 = Arc::new("match_string".to_string());

    let mut match_map = HashMap::new();
    match_map.insert(Value::String(arc_str1.clone()), 4); // Map the string to some operation (4)

    // Span for reporting errors (dummy span for now)
    let dummy_span = Span::default();

    // No default match behavior
    let string_match = Box::new(StaticMatch {
        map: match_map,
        default: Some(2), // No default behavior
        span: dummy_span,
    });



    // Function that just has a MatchJump with the string match
    let code = vec![
        Operation::PushFrom(0),
        Operation::MatchJump(string_match), // Perform the string match
        Operation::PushAtom(string_table.get_id(":err")),             // Push constant (dummy operation)
        Operation::Jump(5),
        Operation::PushGlobal(0),             // Dummy result operation
    ]
    .into_boxed_slice();

    let mut mut_vars = VarTable::default();
    mut_vars.add_ids(&[0]);
    let vars = VarTable::default();


    let func_data = Arc::new(FuncData::new(vars, &mut_vars, &code,1));

    // Global variables (holding some strings)
    let mut global_vars = VarTable::default();
    global_vars.add_ids(&[0]); // No IDs needed in this case
    global_vars.set(0,Value::WeakFunc(Arc::downgrade(&func_data))).unwrap();

    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);

   
    // Test generic
    let unmatched_str = Arc::new("different_string".to_string());
     // Push the second Arc (different address but same content)
    context.stack.push_value(Value::String(unmatched_str)).unwrap();
    let result = context.run().unwrap();
    assert_eq!(result, Value::Atom(get_id!(":err"))); // Should match the string

     // Push the second Arc (different address but same content)
    context.stack.push_value(Value::String(arc_str2.clone())).unwrap();
    context.pos=0;
    let result = context.run().unwrap();
    assert_eq!(result, Value::Func(func_data.clone())); // Should match the string

}

#[test]
fn test_capture_closure() {
    // Step 1: Setup the StringTable
    let mut string_table = StringTable::new();
    let var_a_id = string_table.get_id("var_a");

    // Step 2: Setup the VarTable for the function
    let mut vars = VarTable::default();
    vars.add_ids(&[var_a_id]); // Adding the variable to capture

    // Step 3: Create a FuncMaker that defines the closure capturing
    let mut_vars = VarTable::default();
    let code = vec![Operation::Return].into_boxed_slice(); // Simple code that just returns
    let span = Span::default();

    let func_maker = FuncMaker {
        holder:FuncHolder{
            num_args:0,
            mut_vars_template:VarTable::default(),
            vars: vars.clone(),
            code: code.clone(),
        },
        span,
    };

    // Step 4: Create a Context and push a value to the stack
    let func_data = Arc::new(FuncData::new(vars, &mut_vars, &code,0));
    let global_vars = VarTable::default();
    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);

    // Push a value onto the stack to be captured
    context.stack.push_value(Value::Int(42)).unwrap();

    // Step 5: Perform the CaptureClosure operation
    context.capture_closure(&func_maker).unwrap();

    // Step 6: Verify the result
    let captured_func = context.stack.pop_value().unwrap();
    if let Value::Func(captured_func_data) = captured_func {
        let captured_value = captured_func_data.vars.get(0).unwrap();
        assert_eq!(captured_value, Value::Int(42));
    } else {
        panic!("Expected a captured function on the stack");
    }
}

#[test]
fn test_call_function() {
    // Step 1: Setup the StringTable
    let mut string_table = StringTable::new();
    let var_a_id = string_table.get_id("var_a");

    // Step 2: Setup the VarTable for the function
    let mut mut_vars = VarTable::default();
    mut_vars.add_ids(&[var_a_id]);

    // Step 3: Create the code for the function
    let code = vec![
        Operation::PushNil, // Push nil before returning
        Operation::Return,  // Return from the function
    ].into_boxed_slice();
    let span = Span::default();

    // Step 4: Create the FuncData and Context
    let func_data = Arc::new(FuncData::new(VarTable::default(),&mut_vars, &code,0));
    let global_vars = VarTable::default();
    let mut context = Context::new(func_data.clone(), &global_vars, &string_table);

    // Step 5: Push the function onto the stack and call it
    context.stack.push_value(Value::Func(func_data.clone())).unwrap();
    context.call(span).unwrap();

    // Step 6: Run the function and verify the result
    let result = context.run().unwrap();
    assert_eq!(result, Value::Nil);
}