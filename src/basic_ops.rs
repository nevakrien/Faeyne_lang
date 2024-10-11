// use crate::stack::{ValueStack};


use crate::vm::FuncInputs;
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


pub fn handle_bin(inputs:&mut FuncInputs,op:BinOp) -> Result<(),ErrList>{
    match op {
        BinOp::Equal => {
            let value = is_equal(inputs)?;
            inputs.stack.push_bool(value).map_err(|_|{Error::StackOverflow.to_list()})?;
            Ok(())
        },

        BinOp::NotEqual => {
            let value = !is_equal(inputs)?;
            inputs.stack.push_bool(value).map_err(|_|{Error::StackOverflow.to_list()})?;
            Ok(())
        },
        _ => todo!()
    }
}

pub fn is_equal(inputs: &mut FuncInputs) -> Result<bool, ErrList> {
    let a = inputs.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    let b = inputs.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    inputs.stack.pop_terminator().ok_or_else(|| Error::Bug("failed to pop terminator").to_list())?;
    Ok(a==b)
}

