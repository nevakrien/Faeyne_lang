use crate::ast::StringTable;
use crate::ast::BuildIn;
use crate::ir::Value;
use crate::reporting::*;
use crate::get_id;

use crate::system::*;

pub fn get_type_ffi<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 1 {
        Err(Error::Sig(SigError {}).to_list())
    } else {
        Ok(get_type(args[0].clone()))
    }
}

pub fn get_type<'ctx>(v: Value<'ctx>) -> Value<'ctx> {
    Value::Atom(get_type_id(v))
}

pub fn get_type_id<'ctx>(v: Value<'ctx>) -> usize {
    match v {
        Value::Nil => get_id!(":nil"),
        Value::Bool(_) => get_id!(":bool"),
        Value::String(_) => get_id!(":string"),
        Value::Int(_) => get_id!(":int"),
        Value::Float(_) => get_id!(":float"),
        Value::Atom(_) => get_id!(":atom"),
        Value::Func(_) => get_id!(":func"),
    }
}

pub fn to_bool<'ctx>(v: &Value<'ctx>) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Nil => false,
        Value::String(p) => !p.is_empty(),
        _ => true, // default to truthy for other types
    }
}

pub fn is_equal<'ctx>(v1: &Value<'ctx>, v2: &Value<'ctx>) -> bool {
    match (v1, v2) {
        (Value::Int(i1), Value::Int(i2)) => i1 == i2,
        (Value::Float(f1), Value::Float(f2)) => f1 == f2,
        (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => (*i as f64) == *f,
        (Value::Atom(a), Value::Atom(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Nil, Value::Nil) => true,
        _ => false, // Type mismatch or unsupported types
    }
}

pub fn to_string<'ctx>(value: &Value<'ctx>, table: &StringTable<'ctx>) -> String {
    match value {
        Value::Atom(id) => table
            .get_string(*id)
            .unwrap_or("<unknown atom>")
            .to_string(),
        Value::Int(x) => format!("{}", x),
        Value::Float(x) => format!("{}", x),
        Value::String(s) => s.to_string(),
        _ => format!("{:?}", value), // For other types
    }
}

pub fn try_string<'a,'ctx>(x: &'a Value<'ctx>) -> Result<&'a str,ErrList> {
    let Value::String(gc) = x else { return Err(Error::Sig(SigError {}).to_list()); };
    Ok(&*gc)
}

pub fn try_int<'a,'ctx>(x: &'a Value<'ctx>) -> Result<i64,ErrList> {
    let Value::Int(i) = x else { return Err(Error::Sig(SigError {}).to_list()); };
    Ok(*i)
}

fn nerfed_to_string<'ctx>(value: &Value<'ctx>) -> String {
    match value {
        Value::Atom(id) => format!("Atom<{}>", id),
        Value::Int(x) => format!("{}", x),
        Value::Float(x) => format!("{}", x),
        Value::String(s) => s.to_string(),
        _ => format!("{:?}", value), // For other types
    }
}

// Arithmetic Functions

pub fn buildin_add<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let a = &args[0];
    let b = &args[1];

    match (a, b) {
        (Value::String(s1), Value::String(s2)) => {
            let mut ans = String::with_capacity(s1.len() + s2.len());
            ans.push_str(s1);
            ans.push_str(s2);
            Ok(Value::String(ans.into()))
        }
        (Value::String(s1), b) => {
            let s2 = nerfed_to_string(b);
            let mut ans = String::with_capacity(s1.len() + s2.len());
            ans.push_str(s1);
            ans.push_str(&s2);
            Ok(Value::String(ans.into()))
        }
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 + i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 + f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 + f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f + *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_sub<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let a = &args[0];
    let b = &args[1];

    match (a, b) {
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 - i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 - f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 - f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f - *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_mul<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let a = &args[0];
    let b = &args[1];

    match (a, b) {
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int(i1 * i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 * f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 * f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f * *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_div<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let a = &args[0];
    let b = &args[1];

    match (a, b) {
        (Value::Int(i1), Value::Int(i2)) => {
            if *i2 == 0 {
                Err(Error::Sig(SigError {}).to_list())
            } else if i1 % i2 == 0 {
                Ok(Value::Int(i1 / i2))
            } else {
                Ok(Value::Float(*i1 as f64 / *i2 as f64))
            }
        }
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 / f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Float(*i as f64 / f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Float(*f / *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_int_div<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    if let (Value::Int(i1), Value::Int(i2)) = (&args[0], &args[1]) {
        if *i2 == 0 {
            Err(Error::Sig(SigError {}).to_list())
        } else {
            Ok(Value::Int(i1 / i2))
        }
    } else {
        Err(Error::Sig(SigError {}).to_list())
    }
}

pub fn buildin_modulo<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    if let (Value::Int(i1), Value::Int(i2)) = (&args[0], &args[1]) {
        if *i2 == 0 {
            Err(Error::Sig(SigError {}).to_list())
        } else {
            Ok(Value::Int(i1 % i2))
        }
    } else {
        Err(Error::Sig(SigError {}).to_list())
    }
}

pub fn buildin_pow<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let a = &args[0];
    let b = &args[1];

    match (a, b) {
        (Value::Int(i1), Value::Int(i2)) => {
            if *i2 >= 0 {
                Ok(Value::Int(i1.pow(*i2 as u32)))
            } else {
                Ok(Value::Float((*i1 as f64).powf(*i2 as f64)))
            }
        }
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1.powf(*f2))),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Float((*i as f64).powf(*f))),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Float(f.powf(*i as f64))),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

// Comparison Functions

pub fn buildin_equal<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    Ok(Value::Bool(is_equal(&args[0], &args[1])))
}

pub fn buildin_not_equal<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    Ok(Value::Bool(!is_equal(&args[0], &args[1])))
}

pub fn buildin_smaller<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    match (&args[0], &args[1]) {
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 < i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 < f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Bool((*i as f64) < *f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Bool(*f < *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_bigger<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    match (&args[0], &args[1]) {
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 > i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 > f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Bool((*i as f64) > *f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Bool(*f > *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_smaller_eq<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    match (&args[0], &args[1]) {
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 <= i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 <= f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Bool((*i as f64) <= *f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Bool(*f <= *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

pub fn buildin_bigger_eq<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    match (&args[0], &args[1]) {
        (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool(i1 >= i2)),
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool(f1 >= f2)),
        (Value::Int(i), Value::Float(f)) => Ok(Value::Bool((*i as f64) >= *f)),
        (Value::Float(f), Value::Int(i)) => Ok(Value::Bool(*f >= *i as f64)),
        _ => Err(Error::Sig(SigError {}).to_list()),
    }
}

// Bitwise Operations

pub fn buildin_and<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    if let (Value::Int(i1), Value::Int(i2)) = (&args[0], &args[1]) {
        Ok(Value::Int(i1 & i2))
    } else {
        Err(Error::Sig(SigError {}).to_list())
    }
}

pub fn buildin_or<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    if let (Value::Int(i1), Value::Int(i2)) = (&args[0], &args[1]) {
        Ok(Value::Int(i1 | i2))
    } else {
        Err(Error::Sig(SigError {}).to_list())
    }
}

pub fn buildin_xor<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    if let (Value::Int(i1), Value::Int(i2)) = (&args[0], &args[1]) {
        Ok(Value::Int(i1 ^ i2))
    } else {
        Err(Error::Sig(SigError {}).to_list())
    }
}

// Logical Operations

pub fn buildin_double_and<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let lhs_bool = to_bool(&args[0]);
    let rhs_bool = to_bool(&args[1]);
    Ok(Value::Bool(lhs_bool && rhs_bool))
}

pub fn buildin_double_or<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let lhs_bool = to_bool(&args[0]);
    let rhs_bool = to_bool(&args[1]);
    Ok(Value::Bool(lhs_bool || rhs_bool))
}

pub fn buildin_double_xor<'ctx>(args: Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    if args.len() != 2 {
        return Err(Error::Sig(SigError {}).to_list());
    }
    let lhs_bool = to_bool(&args[0]);
    let rhs_bool = to_bool(&args[1]);
    Ok(Value::Bool(lhs_bool ^ rhs_bool))
}

// Function Mapper

pub fn get_buildin_function(
    op: BuildIn,
) -> for<'ctx> fn(Vec<Value<'ctx>>) -> Result<Value<'ctx>, ErrList> {
    match op {
        BuildIn::Add => buildin_add,
        BuildIn::Sub => buildin_sub,
        BuildIn::Mul => buildin_mul,
        BuildIn::Div => buildin_div,
        BuildIn::IntDiv => buildin_int_div,
        BuildIn::Modulo => buildin_modulo,
        BuildIn::Pow => buildin_pow,

        BuildIn::Equal => buildin_equal,
        BuildIn::NotEqual => buildin_not_equal,

        BuildIn::Smaller => buildin_smaller,
        BuildIn::Bigger => buildin_bigger,
        BuildIn::SmallerEq => buildin_smaller_eq,
        BuildIn::BiggerEq => buildin_bigger_eq,

        BuildIn::And => buildin_and,
        BuildIn::Or => buildin_or,
        BuildIn::Xor => buildin_xor,

        BuildIn::DoubleAnd => buildin_double_and,
        BuildIn::DoubleOr => buildin_double_or,
        BuildIn::DoubleXor => buildin_double_xor,
    }
}
