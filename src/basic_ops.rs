// use crate::stack::{ValueStack};



use std::sync::Arc;
use crate::reporting::{ErrList,Error};

use crate::value::Value;

use ast::ast::StringTable;
use crate::stack::ValueStack;



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

pub fn handle_bin<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,op:BinOp) -> Result<(),ErrList>{
    match op {
        BinOp::Equal => {
            let value = is_equal(stack,table)?;
            stack.push_bool(value).map_err(|_|{Error::StackOverflow.to_list()})?;
            Ok(())
        },

        BinOp::NotEqual => {
            let value = !is_equal(stack,table)?;
            stack.push_bool(value).map_err(|_|{Error::StackOverflow.to_list()})?;
            Ok(())
        },
        _ => todo!()
    }
}

pub fn is_equal<'code>(stack:&mut ValueStack<'code>,_table:&StringTable<'code>) -> Result<bool, ErrList> {
    let a = stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    let b = stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| Error::Bug("failed to pop terminator").to_list())?;
    
    Ok(a==b)
}

//can never ever fail because that would imply we can fail reporting an error
#[inline(never)]
pub fn to_string_debug<'code>(value: &Value<'code>, table: &StringTable<'code>) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Bool(b) => format!("bool({})", b),
        Value::Int(i) => format!("int({})", i),
        Value::Float(f) => format!("float({})", f),
        Value::Atom(atom_id) => format!("atom({})", table.get_raw_str(*atom_id)),
        Value::String(s) => format!("string(\"{}\")", s),
        Value::Func(func) => format!("func({:p})", Arc::as_ptr(func)),
        Value::WeakFunc(weak_func) => format!("weak_func({:p})", weak_func.as_ptr()),
        Value::StaticFunc(static_func) => format!("static_func({:p})", static_func as *const _),
    }
}

pub fn is_equal_wraped<'code>(stack:&mut ValueStack<'code>,_table:&StringTable<'code>) -> Result<(), ErrList> {
    let a = stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    let b = stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    stack.pop_terminator().ok_or_else(|| Error::Bug("failed to pop terminator").to_list())?;
    stack.push_bool(a==b).map_err(|_| Error::Bug("impossible push fail").to_list())
}

#[test]
fn test_is_equal() {

    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();


    // let mut func_inputs = FuncInputs {
    //     stack: value_stack,
    //     table: &string_table,
    // };

    //equal explictly terminated atoms
    let atom_a = Value::Atom(1); 
    let atom_b = Value::Atom(1); 

    #[cfg(feature = "debug_terminators")]
    value_stack.push_terminator().unwrap(); //with a terminator
    value_stack.push_value(atom_a).unwrap();
    value_stack.push_value(atom_b).unwrap();

    handle_bin(&mut value_stack,&string_table, BinOp::Equal).unwrap();

    let result = value_stack.pop_bool().unwrap();
    assert!(result);


    //non equal implictly terminated atoms

    let atom_a = Value::Atom(2);
    let atom_b = Value::Atom(1); 
    value_stack.push_value(atom_a).unwrap();
    value_stack.push_value(atom_b).unwrap();

    handle_bin(&mut value_stack,&string_table, BinOp::Equal).unwrap();

    let result = value_stack.pop_bool().unwrap();
    assert!(!result);

    //atom and nil

    let atom_a = Value::Atom(2); 
    let nil = Value::Nil; 
    value_stack.push_value(atom_a).unwrap();
    value_stack.push_value(nil).unwrap();

    handle_bin(&mut value_stack,&string_table, BinOp::Equal).unwrap();

    #[cfg(feature = "debug_terminators")]{
        let result = value_stack.pop_bool().unwrap();
        assert!(!result);

        //too many values
        value_stack.push_value(Value::Nil).unwrap();
        value_stack.push_value(Value::Nil).unwrap();
        value_stack.push_value(Value::Nil).unwrap();

        let res = handle_bin(&mut value_stack,&string_table, BinOp::Equal);
        assert!(res.is_err());

        //to few values
        value_stack.push_terminator().unwrap();
        let res = handle_bin(&mut value_stack,&string_table, BinOp::Equal);
        assert!(res.is_err());
    }

}