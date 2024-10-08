use crate::stack::StackView;
use crate::value::IRValue;
use crate::basic_ops::handle_bin;
use crate::basic_ops::BinOp;
use crate::value::StringRegistry;
use crate::value::Registry;
use crate::reporting::Error;
use crate::reporting::ErrList;
use crate::value::ValueStack;
use crate::stack::Stack;
use crate::value::Scope;
use ast::ast::StringTable;


pub type Function<'code> = StackView<'code>;
pub type FunctionRegistry<'code>=Registry<Function<'code>>;

// #[repr(C)] //want to orgenize by importance
pub struct Context<'ctx,'code> {
    pub pos:usize,
    pub code: Function<'code>,//we need a varible length stack for these...
    pub scope: Scope<'ctx>,
    pub stack: &'ctx mut Stack,
    
    pub constants: &'code[IRValue],
    pub funcs: &'ctx FunctionRegistry<'code>,
    pub strings: &'ctx StringRegistry,
    
    pub table: &'ctx StringTable<'code>,//for errors only

}

impl<'ctx,'code> Context<'ctx,'code> {
    pub fn new(
        table: &'ctx StringTable<'code>,
        code: StackView<'code>,constants: &'code[IRValue],
        stack: &'ctx mut Stack,scope: Scope<'ctx>,
        strings: &'ctx StringRegistry,funcs: &'ctx FunctionRegistry<'code>
    ) -> Self{
        
        Context{pos:0,table,code,constants,stack,scope,strings,funcs}
    }

    pub fn pop_to(&mut self,id:u32) -> Result<(),ErrList>{
        match self.stack.pop_value(){
            Ok(x) => self.scope.set(id as usize,x)
                .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
            Err(..)  => Err(Error::Bug("over poping").to_list()),
        }
    }

    pub fn push_from(&mut self,id:u32) -> Result<(),ErrList>{
        let value = self.scope.get(id as usize)
            .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
        self.stack.push_grow_value(&value);
        Ok(())
    }

    pub fn push_constant(&mut self,id:u32) -> Result<(),ErrList>{
        let val = self.constants[id as usize];
        self.stack.push_value(&val).map_err(|_|{Error::StackOverflow.to_list()})
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.scope.table.names.iter()
        .map(|id| self.table.get_raw_str(*id))
        .collect()
    } 

    pub fn handle_op(&mut self,op:Operation) -> Result<(),ErrList> {
        match op {
            BinOp(b) => handle_bin(self,b),
            PopTo(id) => self.pop_to(id),
            PushFrom(id) => self.push_from(id),
            PushConst(id) => self.push_constant(id),

            _ => todo!(),
        }
    }

    // pub fn ret(&mut self,p:u32) -> Result<(),ErrList> {
    //     self.pos = p as usize;
    //     todo!()

    // }
}



#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Operation {
    Call(u32),//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
    RetBig(u32),//returns out of the function scope. 
    RetSmall(u32),//returns out of a match block

    PopTo(u32),
    PushFrom(u32),
    PushConst(u32),

    BinOp(BinOp),

    CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
}

use Operation::*;