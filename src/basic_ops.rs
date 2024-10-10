// use crate::stack::{ValueStack};


use crate::vm::Context;
use crate::reporting::{ErrList,Error};

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


pub fn handle_bin(context:&mut Context,op:BinOp) -> Result<(),ErrList>{
    match op {
        BinOp::Equal => {
            let value = is_equal(context)?;
            context.stack.push_bool(value).map_err(|_|{Error::StackOverflow.to_list()})?;
            Ok(())
        },

        BinOp::NotEqual => {
            let value = !is_equal(context)?;
            context.stack.push_bool(value).map_err(|_|{Error::StackOverflow.to_list()})?;
            Ok(())
        },
        _ => todo!()
    }
}

pub fn is_equal(context: &mut Context) -> Result<bool, ErrList> {
    let a = context.stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    let b = context.stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    Ok(a==b)
}

