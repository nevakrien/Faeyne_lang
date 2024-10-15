// use crate::stack::{ValueStack};



use crate::reporting::zero_div_error;
use crate::reporting::sig_error;
use crate::reporting::stacked_error;
use crate::reporting::NoneCallble;
use crate::reporting::overflow_error;
use crate::reporting::bug_error;
use codespan::Span;
use std::sync::Arc;
use crate::reporting::{ErrList,Error};

use crate::value::Value;

use ast::ast::StringTable;
use crate::stack::ValueStack;


#[cfg(test)]
use crate::reporting::report_err_list;

#[inline(always)]
fn _is_equal<'code>(stack:&mut ValueStack<'code>,_table:&StringTable<'code>) -> Result<bool, ErrList> {
    let a = stack.pop_value().ok_or_else(|| bug_error("over popping"))?;
    let b = stack.pop_value().ok_or_else(|| bug_error("over popping"))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| bug_error("failed to pop terminator"))?;
    
    Ok(a==b)
}

#[inline(always)]
pub fn is_equal<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,span:Span) -> Result<bool, ErrList> {
    _is_equal(stack,table).map_err(|err|{
        stacked_error("while using is equal",err,span)
        // Error::Stacked(InternalError{
        //     message:"while using is equal",
        //     err,
        //     span:span,
        // }).to_list()
    })
}


pub fn is_equal_value<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,span:Span) -> Result<(), ErrList> {
    match is_equal(stack,table,span){
        Ok(b) => {
            stack.push_bool(b).map_err(|_| overflow_error())
        },
        Err(e) => Err(e),
    }
}


pub fn is_not_equal_value<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>,span:Span) -> Result<(), ErrList> {
    match is_equal(stack,table,span){
        Ok(b) => {
            stack.push_bool(!b).map_err(|_| overflow_error())
        },
        Err(e) => Err(e),
    }
}

//can never ever fail because that would imply we can fail reporting an error
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

//can never ever fail because that would imply we can fail reporting an error
pub fn to_string_runtime<'code>(value: &Value<'code>, table: &StringTable<'code>) -> String {
    match value {
        Value::Nil => "nil".to_string(),
        Value::Bool(b) => format!("{}", b),
        Value::Int(i) => format!("{}", i),
        Value::Float(f) => format!("{}", f),
        Value::Atom(atom_id) => format!("{}", table.get_raw_str(*atom_id)),
        Value::String(s) => s.to_string(),
        Value::Func(func) => format!("func({:p})", Arc::as_ptr(func)),
        Value::WeakFunc(weak_func) => format!("weak_func({:p})", weak_func.as_ptr()),
        Value::StaticFunc(static_func) => format!("static_func({:p})", static_func as *const _),
    }
}


#[cold]
#[inline(never)]
pub fn non_callble_error<'code>(span:Span,called:&Value<'code>,table:&StringTable<'code>) -> ErrList {
    let value = to_string_debug(called,table);
    Error::NoneCallble(NoneCallble{span,value}).to_list()
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


#[inline]
fn to_bool<'code>(value: &Value<'code>) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        Value::Int(i) => *i > 0,
        Value::Float(f) => *f > 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Atom(_) | Value::Func(_) | Value::WeakFunc(_) | Value::StaticFunc(_) => true,
    }
}

pub fn logical_and<'code>(stack: &mut ValueStack<'code>,_table:&StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling &&", bug_error("over popping"), span))?;
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling &&", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling &&", bug_error("failed to pop terminator"), span))?;

    let result = to_bool(&a) && to_bool(&b);
    stack.push_bool(result).map_err(|_| stacked_error("while calling &&", overflow_error(), span))
}

pub fn logical_or<'code>(stack: &mut ValueStack<'code>,_table:&StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling ||",bug_error("over popping"), span))?;
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling ||", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling ||", bug_error("failed to pop terminator"), span))?;

    let result = to_bool(&a) || to_bool(&b);
    stack.push_bool(result).map_err(|_| stacked_error("while calling ||", overflow_error(), span))
}

pub fn logical_xor<'code>(stack: &mut ValueStack<'code>,_table:&StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling ^^", bug_error("over popping"), span))?;
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling ^^", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling ^^", bug_error("failed to pop terminator"), span))?;

    let result = to_bool(&a) ^ to_bool(&b);
    stack.push_bool(result).map_err(|_| stacked_error("while calling ^^", overflow_error(), span))
}

#[test]
fn test_logical_operations() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Test to_bool function
    assert!(!to_bool(&Value::Nil));
    assert!(!to_bool(&Value::Bool(false)));
    assert!(to_bool(&Value::Bool(true)));
    assert!(to_bool(&Value::Int(1)));
    assert!(!to_bool(&Value::Int(0)));
    assert!(to_bool(&Value::Float(1.0)));
    assert!(!to_bool(&Value::Float(0.0)));
    assert!(to_bool(&Value::String("non-empty".to_string().into())));
    assert!(!to_bool(&Value::String("".to_string().into())));
    assert!(to_bool(&Value::Atom(1)));

    // Test logical AND (&&)
    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(true)).unwrap();
    logical_and(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);

    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    logical_and(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(!result);

    // Test logical OR (||)
    value_stack.push_value(Value::Bool(false)).unwrap();
    value_stack.push_value(Value::Bool(true)).unwrap();
    logical_or(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);

    value_stack.push_value(Value::Bool(false)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    logical_or(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(!result);

    // Test logical XOR (^^)
    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    logical_xor(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);

    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(true)).unwrap();
    logical_xor(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(!result);

    value_stack.push_value(Value::Bool(false)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    logical_xor(&mut value_stack,&string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(!result);
}

pub fn smaller<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling <", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling <", bug_error("over popping"), span))?;


    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling <", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => a < b,
        (Value::Float(a), Value::Float(b)) => a < b,
        (Value::Int(a), Value::Float(b)) => (a as f64) < b,
        (Value::Float(a), Value::Int(b)) => a < (b as f64),
        _ => return Err(stacked_error("while calling <", bug_error("invalid comparison"), span)),
    };
    stack.push_bool(result).map_err(|_| stacked_error("while calling <", overflow_error(), span))
}

pub fn bigger<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling >", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling >", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling >", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => a > b,
        (Value::Float(a), Value::Float(b)) => a > b,
        (Value::Int(a), Value::Float(b)) => (a as f64) > b,
        (Value::Float(a), Value::Int(b)) => a > (b as f64),
        _ => return Err(stacked_error("while calling >", bug_error("invalid comparison"), span)),
    };
    stack.push_bool(result).map_err(|_| stacked_error("while calling >", overflow_error(), span))
}

pub fn smaller_eq<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling <=", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling <=", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling <=", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => a <= b,
        (Value::Float(a), Value::Float(b)) => a <= b,
        (Value::Int(a), Value::Float(b)) => (a as f64) <= b,
        (Value::Float(a), Value::Int(b)) => a <= (b as f64),
        _ => return Err(stacked_error("while calling <=", bug_error("invalid comparison"), span)),
    };
    stack.push_bool(result).map_err(|_| stacked_error("while calling <=", overflow_error(), span))
}

pub fn bigger_eq<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling >=", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling >=", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling >=", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => a >= b,
        (Value::Float(a), Value::Float(b)) => a >= b,
        (Value::Int(a), Value::Float(b)) => (a as f64) >= b,
        (Value::Float(a), Value::Int(b)) => a >= (b as f64),
        _ => return Err(stacked_error("while calling >=", bug_error("invalid comparison"), span)),
    };
    stack.push_bool(result).map_err(|_| stacked_error("while calling >=", overflow_error(), span))
}

#[test]
fn test_comparison_operations() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Test Smaller (<)
    value_stack.push_value(Value::Int(1)).unwrap();
    value_stack.push_value(Value::Int(2)).unwrap();
    smaller(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);

    // Test Bigger (>)
    value_stack.push_value(Value::Int(3)).unwrap();
    value_stack.push_value(Value::Int(2)).unwrap();
    bigger(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);

    // Test SmallerEq (<=)
    value_stack.push_value(Value::Int(2)).unwrap();
    value_stack.push_value(Value::Int(2)).unwrap();
    smaller_eq(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);

    // Test BiggerEq (>=)
    value_stack.push_value(Value::Float(3.0)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    bigger_eq(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_bool().unwrap();
    assert!(result);
}

#[cold]
pub fn pow<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling **", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling **", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling pow", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a.pow(b as u32)),
        (Value::Float(a), Value::Float(b)) => Value::Float(a.powf(b)),
        (Value::Int(a), Value::Float(b)) => Value::Float((a as f64).powf(b)),
        (Value::Float(a), Value::Int(b)) => Value::Float(a.powf(b as f64)),

        _ => return Err(stacked_error("while calling pow", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling pow", overflow_error(), span))
}

pub fn mul<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling *", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling *", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling *", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
        (Value::Int(a), Value::Float(b)) => Value::Float((a as f64) * b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a * (b as f64)),
        _ => return Err(stacked_error("while calling *", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling *", overflow_error(), span))
}

pub fn sub<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling -", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling -", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling -", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
        (Value::Int(a), Value::Float(b)) => Value::Float((a as f64) - b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a - (b as f64)),
        _ => return Err(stacked_error("while calling -", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling -", overflow_error(), span))
}

#[test]
fn test_arithmetic_operations() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Test Pow
    value_stack.push_value(Value::Int(2)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    pow(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(8));

    // Test Mul (*)
    value_stack.push_value(Value::Int(3)).unwrap();
    value_stack.push_value(Value::Int(4)).unwrap();
    mul(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(12));

    // Test Sub (-)
    value_stack.push_value(Value::Int(10)).unwrap();
    value_stack.push_value(Value::Int(4)).unwrap();
    sub(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(6));
}

pub fn div<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling /", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling /", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling /", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                return Err(stacked_error("while calling /", zero_div_error(), span));
            }
            if a % b == 0 {
                Value::Int(a / b)
            } else {
                Value::Float((a as f64) / (b as f64))
            }
        }
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(stacked_error("while calling /", zero_div_error(), span));
            }
            Value::Float(a / b)
        }
        (Value::Int(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(stacked_error("while calling /", zero_div_error(), span));
            }
            Value::Float((a as f64) / b)
        }
        (Value::Float(a), Value::Int(b)) => {
            if b == 0 {
                return Err(stacked_error("while calling /", zero_div_error(), span));
            }
            Value::Float(a / (b as f64))
        }
        _ => return Err(stacked_error("while calling /", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling /", overflow_error(), span))
}

pub fn modulo<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling %", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling %", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling %", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                return Err(stacked_error("while calling %", zero_div_error(), span));
            }
            Value::Int(a % b)
        }
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(stacked_error("while calling %", zero_div_error(), span));
            }
            Value::Float(a % b)
        }
        (Value::Int(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(stacked_error("while calling %", zero_div_error(), span));
            }
            Value::Float((a as f64) % b)
        }
        (Value::Float(a), Value::Int(b)) => {
            if b == 0 {
                return Err(stacked_error("while calling %", zero_div_error(), span));
            }
            Value::Float(a % (b as f64))
        }
        _ => return Err(stacked_error("while calling %", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling %", overflow_error(), span))
}

pub fn int_div<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling //", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling //", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling //", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                return Err(stacked_error("while calling //", zero_div_error(), span));
            }
            Value::Int(a / b)
        }
        (Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(stacked_error("while calling //", zero_div_error(), span));
            }
            Value::Int((a / b).floor() as i64)
        }
        (Value::Int(a), Value::Float(b)) => {
            if b == 0.0 {
                return Err(stacked_error("while calling //", zero_div_error(), span));
            }
            Value::Int(((a as f64) / b).floor() as i64)
        }
        (Value::Float(a), Value::Int(b)) => {
            if b == 0 {
                return Err(stacked_error("while calling //", zero_div_error(), span));
            }
            Value::Int((a / (b as f64)).floor() as i64)
        }
        _ => return Err(stacked_error("while calling //", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling //", overflow_error(), span))
}

#[test]
fn test_division_operations() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Test Div (/)
    value_stack.push_value(Value::Int(10)).unwrap();
    value_stack.push_value(Value::Int(2)).unwrap();
    div(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(5));

    value_stack.push_value(Value::Int(10)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    div(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Float(10.0 / 3.0));

    // Test Modulo (%)
    value_stack.push_value(Value::Int(10)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    modulo(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(1));

    // Test IntDiv (//)
    value_stack.push_value(Value::Float(10.0)).unwrap();
    value_stack.push_value(Value::Float(3.0)).unwrap();
    int_div(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(3));
}

#[cold]
pub fn bitwise_and<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling &", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling &", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling &", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a & b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a & b),
        _ => return Err(stacked_error("while calling &", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling &", overflow_error(), span))
}

#[cold]
pub fn bitwise_or<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling |", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling |", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling |", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a | b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a | b),
        _ => return Err(stacked_error("while calling |", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling |", overflow_error(), span))
}

#[cold]
pub fn bitwise_xor<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling ^", bug_error("over popping"), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling ^", bug_error("over popping"), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling ^", bug_error("failed to pop terminator"), span))?;

    let result = match (a, b) {
        (Value::Int(a), Value::Int(b)) => Value::Int(a ^ b),
        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a ^ b),
        _ => return Err(stacked_error("while calling ^", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling ^", overflow_error(), span))
}

#[test]
fn test_bitwise_operations() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Test Bitwise AND (&)
    value_stack.push_value(Value::Int(6)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    bitwise_and(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(2));

    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    bitwise_and(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Bool(false));

    // Test Bitwise OR (|)
    value_stack.push_value(Value::Int(6)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    bitwise_or(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(7));

    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    bitwise_or(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Bool(true));

    // Test Bitwise XOR (^)
    value_stack.push_value(Value::Int(6)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    bitwise_xor(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(5));

    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();
    bitwise_xor(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Bool(true));
}

pub fn add<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>, span: Span) -> Result<(), ErrList> {
    let b = stack.pop_value().ok_or_else(|| stacked_error("while calling +", sig_error(), span))?;
    let a = stack.pop_value().ok_or_else(|| stacked_error("while calling +", sig_error(), span))?;

    #[cfg(feature = "debug_terminators")]
    stack.pop_terminator().ok_or_else(|| stacked_error("while calling +", sig_error(), span))?;

    let result = match (a, b) {
        // Numeric addition for Ints and Floats
        (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
        (Value::Int(a), Value::Float(b)) => Value::Float((a as f64) + b),
        (Value::Float(a), Value::Int(b)) => Value::Float(a + (b as f64)),

        // String concatenation
        (Value::String(mut s1), Value::String(s2)) => {
            if let Some(s1_mut) = Arc::get_mut(&mut s1) {
                s1_mut.push_str(&s2);
                Value::String(s1)
            } else {
                let mut ans = String::with_capacity(s1.len() + s2.len());
                ans.push_str(&s1);
                ans.push_str(&s2);
                Value::String(ans.into())
            }
        }
        (Value::String(mut s1), b) => {
            let s2 = to_string_runtime(&b, _table);
            if let Some(s1_mut) = Arc::get_mut(&mut s1) {
                s1_mut.push_str(&s2);
                Value::String(s1)
            } else {
                let mut ans = String::with_capacity(s1.len() + s2.len());
                ans.push_str(&s1);
                ans.push_str(&s2);
                Value::String(ans.into())
            }
        }
        (a, Value::String(mut s2)) => {
            let s1 = to_string_runtime(&a, _table);
            if let Some(s2_mut) = Arc::get_mut(&mut s2) {
                s2_mut.insert_str(0, &s1);
                Value::String(s2)
            } else {
                let mut ans = String::with_capacity(s1.len() + s2.len());
                ans.push_str(&s1);
                ans.push_str(&s2);
                Value::String(ans.into())
            }
        }
        _ => return Err(stacked_error("while calling +", sig_error(), span)),
    };
    stack.push_value(result).map_err(|_| stacked_error("while calling +", overflow_error(), span))
}




#[test]
fn test_add_operation() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Test numeric addition for Ints and Floats
    value_stack.push_value(Value::Int(5)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(8));

    value_stack.push_value(Value::Float(2.5)).unwrap();
    value_stack.push_value(Value::Float(1.5)).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Float(4.0));

    value_stack.push_value(Value::Int(2)).unwrap();
    value_stack.push_value(Value::Float(3.5)).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Float(5.5));

    // Test string concatenation
    value_stack.push_value(Value::String("Hello".to_string().into())).unwrap();
    value_stack.push_value(Value::String(" World".to_string().into())).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::String("Hello World".to_string().into()));

    value_stack.push_value(Value::String("Hello".to_string().into())).unwrap();
    value_stack.push_value(Value::Int(123)).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap();
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::String("Hello123".to_string().into()));
}



#[test]
fn test_chained_operations() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::default();

    // Chain operations: 5 + 3 - 2
    value_stack.push_value(Value::Int(5)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: [8]

    value_stack.push_value(Value::Int(2)).unwrap();
    sub(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: [6]
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(6));

    // Chain operations: (2 * 3) + (4 / 2)
    value_stack.push_value(Value::Int(2)).unwrap();
    value_stack.push_value(Value::Int(3)).unwrap();
    mul(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: [6]

    #[cfg(feature = "debug_terminators")]
    value_stack.push_terminator().unwrap();
    value_stack.push_value(Value::Int(4)).unwrap();
    value_stack.push_value(Value::Int(2)).unwrap();
    div(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: [6, 2]

    add(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: [8]
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::Int(8));

    // Chain operations with strings: "Hello" + " " + "World" + 123
    value_stack.push_value(Value::String("Hello".to_string().into())).unwrap();
    value_stack.push_value(Value::String(" ".to_string().into())).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: ["Hello "]


    value_stack.push_value(Value::String("World".to_string().into())).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: ["Hello World"]


    value_stack.push_value(Value::Int(123)).unwrap();
    add(&mut value_stack, &string_table, mock_span).unwrap(); // Stack: ["Hello World123"]
    let result = value_stack.pop_value().unwrap();
    assert_eq!(result, Value::String("Hello World123".to_string().into()));
}

#[test]
fn test_division_by_zero() {
    let mut value_stack = ValueStack::new();
    let string_table = StringTable::new();
    let mock_span = Span::new(0, 10);

    // Test division by zero with integers
    value_stack.push_value(Value::Int(10)).unwrap();
    value_stack.push_value(Value::Int(0)).unwrap();
    let result = div(&mut value_stack, &string_table, mock_span);
    assert!(result.is_err());
    if let Err(err_list) = result {
        report_err_list(&err_list, "10 / 0", &string_table);
    }

    // Test division by zero with floats
    value_stack.push_value(Value::Float(10.0)).unwrap();
    value_stack.push_value(Value::Float(0.0)).unwrap();
    let result = div(&mut value_stack, &string_table, mock_span);
    assert!(result.is_err());
    if let Err(err_list) = result {
        report_err_list(&err_list, "10.0 / 0.0", &string_table);
    }
}

pub fn call_string<'code>(_string:Arc<String>,_stack: &mut ValueStack<'code>, _table: &StringTable<'code>, _span: Span) -> Result<(), ErrList> {
    todo!()
}
