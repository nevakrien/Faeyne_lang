use crate::value::{ValuePushStack,ValuePopStack};
use crate::stack::StackView;
use crate::basic_ops::handle_bin;
use crate::basic_ops::BinOp;
use crate::value::StringRegistry;
use crate::value::Registry;
use crate::reporting::Error;
use crate::reporting::ErrList;
use crate::stack::Stack;
use crate::value::Scope;
use ast::ast::StringTable;

use arrayvec::ArrayVec;

#[derive(Clone)]
pub enum Function<'code> {
    Native(StackView<'code>),
    FFI,
}
pub type FunctionRegistry<'code>=Registry<Function<'code>>;

// #[repr(C)] //want to orgenize by importance
pub struct Context<'ctx,'code> {
    // pub pos:usize,
    code: StackView<'code>,//we need a varible length stack for these...
    call_stack: ArrayVec<StackView<'code>,1000>,

    pub scope: Scope<'ctx>,
    static_ids: &'ctx[u32], //these ids should not be GCed
    pub stack: &'ctx mut Stack,
    
    //pub constants: &'code[IRValue],
    pub funcs: &'ctx FunctionRegistry<'code>,
    pub strings: &'ctx StringRegistry,
    
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
        code: StackView<'code>,//constants: &'code[IRValue],
        static_ids: &'ctx[u32],
        stack: &'ctx mut Stack,scope: Scope<'ctx>,
        strings: &'ctx StringRegistry,funcs: &'ctx FunctionRegistry<'code>
    ) -> Self{
        
        Context{
            call_stack:ArrayVec::new(),
            static_ids,table,code,stack,scope,strings,funcs
        }
    }

    pub fn get_code(&self) -> &StackView<'code>{
        &self.code
    }


    fn pop_to(&mut self,id:u32) -> Result<(),ErrList>{
        match self.stack.pop_value(){
            Ok(x) => self.scope.set(id as usize,x)
                .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
            Err(..)  => Err(Error::Bug("over poping").to_list()),
        }
    }

    fn push_from(&mut self,id:u32) -> Result<(),ErrList>{
        let value = self.scope.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.stack.push_value(&value).map_err(|_|{Error::StackOverflow.to_list()})?;
        Ok(())
    }

    fn push_constant(&mut self) -> Result<(),ErrList>{
        let res = unsafe{self.code.pop()};
        let val = res.ok_or_else(|| Error::Bug("over pop").to_list())?;
        self.stack.push_value(&val.to_inner()).map_err(|_|{Error::StackOverflow.to_list()})
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.scope.table.names.iter()
        .map(|id| self.table.get_raw_str(*id))
        .collect()
    } 

    fn small_ret(&mut self,id:u32) -> Result<(),ErrList> {
        // unsafe {self.code.set_index(id as isize);}
        todo!()
    }

    fn big_ret(&mut self) -> Result<(),ErrList> {
        self.code = self.call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
        todo!()
    }

    /// # Safety
    ///
    /// this is safe as long as we pop the ops 1 by 1 as the code says
    /// we are relying on the compilation step making correct code 
    pub unsafe fn handle_op(&mut self,op:Operation) -> Result<(),ErrList> {
        match op {
            BinOp(b) => handle_bin(self,b),
            PopTo(id) => self.pop_to(id),
            PushFrom(id) => self.push_from(id),
            PushConst => self.push_constant(),

            RetSmall(id) => self.small_ret(id),
            RetBig => self.big_ret(),

            _ => todo!(),
        }
    }

    //returns true if we should keep going
    pub fn next_op(&mut self) -> Result<bool,ErrList>{
        unsafe {
            let r = self.code.pop();
            let Some(op) = r else {return Ok(false)};
            self.handle_op(op.to_inner()).map(|_| true)
        }
        
    }

}



#[derive(Debug,PartialEq,Clone,Copy)]
#[repr(u32)]
pub enum Operation {
    Call,//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
    RetBig,//returns out of the function scope. 
    RetSmall(u32),//returns out of a match block

    PopTo(u32),
    PushFrom(u32),
    PushConst,

    BinOp(BinOp),

    Match,//pops a value to match aginst then returns the result (not quite sure about the detais)
    CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
}

use Operation::*;