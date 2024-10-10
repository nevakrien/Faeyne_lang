use crate::value::VarTable;

use crate::basic_ops::handle_bin;
use crate::basic_ops::BinOp;
use crate::reporting::Error;
use crate::reporting::ErrList;
use crate::stack::ValueStack;
// use crate::value::Scope;
use ast::ast::StringTable;

use arrayvec::ArrayVec;

pub type Code<'code> = &'code[Operation];

#[derive(Clone)]
pub enum Function<'code> {
    Native(Code<'code>),
    FFI,
}

pub struct RetData<'code> {
    ret:usize,
    code:Code<'code>,
    // scope:Scope<'ctx>
}

pub const MAX_LOCAL_SCOPES: usize = 1000;
pub const MAX_RECURSION :usize=2_500;

// #[repr(C)] //want to orgenize by importance
pub struct Context<'ctx,'code> {
    // pub pos:usize,
    pos:usize,
    pub code: Code<'code>,//we need a varible length stack for these...
    call_stack:  &'ctx mut ArrayVec<RetData<'code>,MAX_RECURSION>,
    local_call_stack: &'ctx mut ArrayVec<RetData<'code>,MAX_LOCAL_SCOPES>,
    vars:&'ctx mut VarTable,

    pub stack: &'ctx mut ValueStack,
    
    //pub constants: &'code[IRValue],
    
    pub table: &'ctx StringTable<'code>,//for errors only
}

impl<'ctx,'code> Context<'ctx,'code> {
    /// # Safety
    ///
    /// code must never tell us to pop the wrong type from stack.
    /// as long as code allways pops any non value type on stack that it pushed
    /// the code is safe
    pub unsafe fn new(
        
        table: &'ctx StringTable<'code>,
        code: Code<'code>,//constants: &'code[IRValue],
        
        stack: &'ctx mut ValueStack,vars: &'ctx mut VarTable,
        call_stack:  &'ctx mut ArrayVec<RetData<'code>,MAX_RECURSION>,
        local_call_stack: &'ctx mut ArrayVec<RetData<'code>,MAX_LOCAL_SCOPES>,
        
    ) -> Self{
        
        Context{
            pos:0,table,code,stack,call_stack,local_call_stack,vars
        }
    }


    fn pop_to(&mut self,id:u32) -> Result<(),ErrList>{
        match self.stack.pop_value(){
            Some(x) => self.vars.set(id as usize,x)
                .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
            None  => Err(Error::Bug("over poping").to_list()),
        }
    }

    fn push_from(&mut self,id:u32) -> Result<(),ErrList>{
        let value = self.vars.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    fn push_constant(&mut self) -> Result<(),ErrList>{
        todo!()
        // let res = unsafe{self.code.pop()};
        // let val = res.ok_or_else(|| Error::Bug("over pop").to_list())?;
        // self.stack.push_value(&val.to_inner()).map_err(|_|{Error::StackOverflow.to_list()})
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.vars.names.iter()
        .map(|id| self.table.get_raw_str(*id))
        .collect()
    } 

    fn big_ret(&mut self) -> Result<(),ErrList> {
        let ret_data = self.call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
        let value = self.stack.pop_value().ok_or_else(|| Error::Bug("over pop value stack").to_list())?;

        self.code = ret_data.code;
        
        //unwind the stack
        while self.stack.len()>ret_data.ret {
            self.stack.pop_value();
        }
        assert_eq!(ret_data.ret,self.stack.len());
        

        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;
        

        // self.scope = ret_data.scope;
        self.local_call_stack.clear();

        todo!()
    }

    fn small_ret(&mut self) -> Result<(),ErrList> {
        let ret_data = self.local_call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
        let value = self.stack.pop_value().ok_or_else(|| Error::Bug("over pop value stack").to_list())?;

        self.code = ret_data.code;
        //unwind the stack
        while self.stack.len()>ret_data.ret {
            self.stack.pop_value();
        }
        assert_eq!(ret_data.ret,self.stack.len());
        

        self.stack.push_value(value).map_err(|_|{Error::StackOverflow.to_list()})?;

        // self.scope = ret_data.scope;

        todo!()
    }

    // fn push_scope(&mut self,code:StackView<'code>,ret:StackRet) -> Result<(),ErrList> {
    //     let mut ret = RetData{scope:self.scope,code,ret};
    //     self.scope = ret.scope.add_scope(&[]);
    //     Ok(())
    // }


    fn handle_op(&mut self,op:Operation) -> Result<(),ErrList> {
        match op {
            BinOp(b) => handle_bin(self,b),
            PopTo(id) => self.pop_to(id),
            PushFrom(id) => self.push_from(id),
            PushConst => self.push_constant(),

            RetSmall => self.small_ret(),
            RetBig => self.big_ret(),

            _ => todo!(),
        }
    }

    //returns true if we should keep going
    pub fn next_op(&mut self) -> Result<bool,ErrList>{
        if self.pos>=self.code.len() {return Ok(false);}
        self.pos+=1;
        let op = self.code[self.pos];
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
    PushConst,

    BinOp(BinOp),

    Match,//pops a value to match aginst then returns the result (not quite sure about the detais)
    CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
}

use Operation::*;