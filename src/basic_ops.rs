// use crate::stack::{ValueStack};



use crate::vm::FuncInputs;
use crate::reporting::{ErrList,Error};

#[cfg(test)]
use crate::value::Value;

#[cfg(test)]
use ast::ast::StringTable;

#[cfg(test)]
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

pub fn is_equal_wraped(inputs: &mut FuncInputs) -> Result<(), ErrList> {
    let a = inputs.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    let b = inputs.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    inputs.stack.pop_terminator().ok_or_else(|| Error::Bug("failed to pop terminator").to_list())?;
    inputs.stack.push_bool(a==b).map_err(|_| Error::Bug("impossible push fail").to_list())
}

#[test]
fn test_is_equal() {

    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();


    let mut func_inputs = FuncInputs {
        stack: &mut value_stack,
        table: &string_table,
    };

    //equal explictly terminated atoms
    let atom_a = Value::Atom(1); 
    let atom_b = Value::Atom(1); 

    func_inputs.stack.push_terminator().unwrap(); //with a terminator
    func_inputs.push_value(atom_a).unwrap();
    func_inputs.push_value(atom_b).unwrap();

    handle_bin(&mut func_inputs, BinOp::Equal).unwrap();

    let result = func_inputs.stack.pop_bool().unwrap();
    assert!(result);


    //non equal implictly terminated atoms

    let atom_a = Value::Atom(2);
    let atom_b = Value::Atom(1); 
    func_inputs.push_value(atom_a).unwrap();
    func_inputs.push_value(atom_b).unwrap();

    handle_bin(&mut func_inputs, BinOp::Equal).unwrap();

    let result = func_inputs.stack.pop_bool().unwrap();
    assert!(!result);

    //atom and nil

    let atom_a = Value::Atom(2); 
    let nil = Value::Nil; 
    func_inputs.push_value(atom_a).unwrap();
    func_inputs.push_value(nil).unwrap();

    handle_bin(&mut func_inputs, BinOp::Equal).unwrap();

    let result = func_inputs.stack.pop_bool().unwrap();
    assert!(!result);

    //too many values
    func_inputs.push_value(Value::Nil).unwrap();
    func_inputs.push_value(Value::Nil).unwrap();
    func_inputs.push_value(Value::Nil).unwrap();

    let res = handle_bin(&mut func_inputs, BinOp::Equal);
    assert!(res.is_err());

    //to few values
    func_inputs.stack.push_terminator().unwrap();
    let res = handle_bin(&mut func_inputs, BinOp::Equal);
    assert!(res.is_err());

}