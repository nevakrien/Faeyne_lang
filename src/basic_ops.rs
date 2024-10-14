// use crate::stack::{ValueStack};



use codespan::Span;
use std::sync::Arc;
use crate::reporting::{ErrList,Error,InternalError};

use crate::value::Value;

use ast::ast::StringTable;
use crate::stack::ValueStack;



#[inline(always)]
fn _is_equal<'code>(stack:&mut ValueStack<'code>,_table:&StringTable<'code>) -> Result<bool, ErrList> {
    let a = stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;
    let b = stack.pop_value().ok_or_else(|| Error::Bug("over popping").to_list())?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| Error::Bug("failed to pop terminator").to_list())?;
    
    Ok(a==b)
}

#[inline(always)]
pub fn is_equal<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,span:Span) -> Result<bool, ErrList> {
    _is_equal(stack,table).map_err(|err|{
        Error::Stacked(InternalError{
            message:"while using is equal",
            err,
            span:span,
        }).to_list()
    })
}

pub fn is_equal_value<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,span:Span) -> Result<(), ErrList> {
    match is_equal(stack,table,span){
        Ok(b) => {
            stack.push_bool(b).map_err(|_| Error::StackOverflow.to_list())
        },
        Err(e) => Err(e),
    }
}

pub fn is_not_equal_value<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,span:Span) -> Result<(), ErrList> {
    match is_equal(stack,table,span){
        Ok(b) => {
            stack.push_bool(!b).map_err(|_| Error::StackOverflow.to_list())
        },
        Err(e) => Err(e),
    }
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

    let mock_span = Span::default();


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

    is_equal_value(&mut value_stack,&string_table,mock_span).unwrap();

    let result = value_stack.pop_bool().unwrap();
    assert!(result);


    //non equal implictly terminated atoms

    let atom_a = Value::Atom(2);
    let atom_b = Value::Atom(1); 
    value_stack.push_value(atom_a).unwrap();
    value_stack.push_value(atom_b).unwrap();

    is_equal_value(&mut value_stack,&string_table,mock_span).unwrap();
    

    let result = value_stack.pop_bool().unwrap();
    assert!(!result);

    //atom and nil

    let atom_a = Value::Atom(2); 
    let nil = Value::Nil; 
    value_stack.push_value(atom_a).unwrap();
    value_stack.push_value(nil).unwrap();

    is_equal_value(&mut value_stack,&string_table,mock_span).unwrap();
    

    #[cfg(feature = "debug_terminators")]{
        let result = value_stack.pop_bool().unwrap();
        assert!(!result);

        //too many values
        value_stack.push_value(Value::Nil).unwrap();
        value_stack.push_value(Value::Nil).unwrap();
        value_stack.push_value(Value::Nil).unwrap();

        let res = is_equal_value(&mut value_stack,&string_table,mock_span);
        assert!(res.is_err());

        //to few values
        value_stack.push_terminator().unwrap();
        let res = is_equal_value(&mut value_stack,&string_table,mock_span);
        assert!(res.is_err());
    }

}