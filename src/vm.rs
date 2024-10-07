use crate::reporting::Error;
use crate::reporting::ErrList;
use crate::value::ValueStack;
use crate::stack::Stack;
use crate::value::Scope;
use ast::ast::StringTable;

pub struct Context<'ctx,'code> {
    table: &'ctx StringTable<'code>,
    stack: &'ctx mut Stack,
    scope: Scope<'ctx>
}

impl<'ctx,'code> Context<'ctx,'code> {
    pub fn new(table: &'ctx StringTable<'code>,stack: &'ctx mut Stack,scope: Scope<'ctx>) -> Self{
        Context{table,stack,scope}
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
    Call(u32),
    RetBig,
    RetSmall,

    PopTo(u32),
    PushFrom(u32),

    BinOp(BinOp),
}