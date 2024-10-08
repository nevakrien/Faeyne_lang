use crate::value::IRValue;
use crate::value::ValueStack;
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
            context.stack.push_grow_bool(value);
            Ok(())
        },

        BinOp::NotEqual => {
            let value = !is_equal(context)?;
            context.stack.push_grow_bool(value);
            Ok(())
        },
        _ => todo!()
    }
}

pub fn is_equal(context:&mut Context) -> Result<bool,ErrList>{
    let a = context.stack.pop_value().map_err(|_|Error::Bug("over poping").to_list())? ;
    let b = context.stack.pop_value().map_err(|_|Error::Bug("over poping").to_list())? ;
    match (a,b){
        (IRValue::String(a),IRValue::String(b)) => {
            if a==b {
                return Ok(true);
            }
            let a = context.strings.get(a).ok_or_else(|| {Error::Bug("non existent id").to_list()})?;
            let b = context.strings.get(b).ok_or_else(|| {Error::Bug("non existent id").to_list()})?;

            Ok(a==b)
        },
        (a,b) => Ok(a==b)
    }
}