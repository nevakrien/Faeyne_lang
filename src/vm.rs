use crate::value::StringRegistry;
use crate::value::Registry;
use crate::reporting::Error;
use crate::reporting::ErrList;
use crate::value::ValueStack;
use crate::stack::Stack;
use crate::value::Scope;
use ast::ast::StringTable;


#[derive(Clone,Debug)]
pub struct Function{
    pub code:Vec<Operation>
}

// #[repr(C)] //want to orgenize by importance
pub struct Context<'ctx,'code> {
    scope: Scope<'ctx>,
    stack: &'ctx mut Stack,

    funcs: &'ctx Registry<Function>,
    strings: &'ctx StringRegistry,
    
    table: &'ctx StringTable<'code>,//for errors only

}

impl<'ctx,'code> Context<'ctx,'code> {
    pub fn new(
        table: &'ctx StringTable<'code>,stack: &'ctx mut Stack,scope: Scope<'ctx>,
        strings: &'ctx StringRegistry,funcs: &'ctx Registry<Function>
    ) -> Self{
        
        Context{table,stack,scope,strings,funcs}
    }

    pub fn pop_to(&mut self,id:u32) -> Result<(),ErrList>{
        match self.stack.pop_value(){
            Ok(x) => self.scope.set(id as usize,x)
                .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
            Err(..)  => Err(Error::Bug("stack overflow").to_list()),
        }
        
    }

    pub fn curent_var_names(&self) -> Vec<&'code str> {
        self.scope.table.names.iter()
        .map(|id| self.table.get_raw_str(*id))
        .collect()
    } 
}

#[derive(Debug,PartialEq,Clone,Copy)]
#[repr(u32)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Modulo,
    Pow,

    Equal,
    NotEqual,
    Smaller,
    Bigger,
    SmallerEq,
    BiggerEq,

    And,
    Or,
    Xor,

    DoubleAnd,
    DoubleOr,
    DoubleXor,
}

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum Operation {
    Call(u32),//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
    RetBig,//returns out of the function scope. 
    RetSmall,//returns out of a match block

    PopTo(u32),
    PushFrom(u32),

    BinOp(BinOp),

    CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
}