use crate::ast::StringTable;
use crate::ast::BuildIn;
use crate::ir::Value;
use crate::reporting::*;
use crate::get_id;

use crate::system::*;

pub fn get_type_ffi(args : Vec<Value>) -> Result<Value,ErrList> {
    if args.len() != 1 {
        Err(Error::Sig(SigError{}).to_list())
    }
    else{
        Ok(get_type(args[0].clone()))
    }
}

pub fn get_type(v : Value) -> Value {
	Value::Atom(get_type_id(v))
}

pub fn get_type_id(v : Value) -> usize{
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

pub fn to_bool(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Int(i) => *i != 0,
        Value::Float(f) => *f != 0.0,
        Value::Nil => false,
        Value::String(p) => p.len()>0,
        _ => true, // default to truthy for other types
    }
}
pub fn is_equal(v1: &Value, v2: &Value) -> bool {
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


macro_rules! perform_arithmetic {
    ($v1:expr, $v2:expr, $op:expr) => {
        match ($v1, $v2) {
            (Value::Int(i1), Value::Int(i2)) => Ok(Value::Int($op(*i1, *i2))),
            (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float($op(*f1, *f2))),
            (Value::Int(i), Value::Float(f)) => Ok(Value::Float($op(*i as f64, *f))),
            (Value::Float(f), Value::Int(i)) => Ok(Value::Float($op(*f, *i as f64))),
            _ => Err(SigError {
                // Handle type mismatch error here
            }),
        }
    };
}

macro_rules! perform_num_comparison {
    ($v1:expr, $v2:expr, $op:expr) => {
        match ($v1, $v2) {
            (Value::Int(i1), Value::Int(i2)) => Ok(Value::Bool($op(*i1 , *i2 ))),
            (Value::Float(f1), Value::Float(f2)) => Ok(Value::Bool($op(*f1, *f2))),
            (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => {
                Ok(Value::Bool($op(*i as f64, *f)))
            },
            _ => Err(SigError {
                // handle type mismatch error here
            }),
        }
    };
}

pub fn handle_buildin(args: Vec<Value>, op: BuildIn) -> Result<Value, SigError> {
    if args.len()!=2 {
    	return Err(SigError {
                    // Handle type mismatch error here
                });
    }

    match op {
        //equality
        BuildIn::Equal => Ok(Value::Bool(is_equal(&args[0], &args[1]))),
        BuildIn::NotEqual => Ok(Value::Bool(!is_equal(&args[0], &args[1]))),

         // Bitwise Operations
        BuildIn::And | BuildIn::Or | BuildIn::Xor => perform_bitwise_op(&args[0], &args[1], op),
        BuildIn::DoubleAnd | BuildIn::DoubleOr | BuildIn::DoubleXor => perform_logical_op(&args[0], &args[1], op),

        //special cases for int int
        BuildIn::Div => perform_division(&args[0], &args[1]), // Special case for division
        BuildIn::Pow => perform_power(&args[0], &args[1]),    // Special case for power
        BuildIn::IntDiv => perform_int_div(&args[0], &args[1]),
        BuildIn::Modulo => perform_modulo(&args[0], &args[1]),

        //string
        BuildIn::Add => {
            if let (Value::String(s1), Value::String(s2)) = (&args[0], &args[1]) {
                let mut ans = String::with_capacity(s1.len()+s2.len());
                ans.push_str(s1);
                ans.push_str(s2);
                
                Ok(Value::String(ans.into()))
            } else {
                perform_arithmetic!(&args[0], &args[1], |a, b| a + b)
            }
        },
        // standard arithmetic,
        BuildIn::Sub => perform_arithmetic!(&args[0], &args[1], |a, b| a - b),
        BuildIn::Mul => perform_arithmetic!(&args[0], &args[1], |a, b| a * b),

        //Standard numeric comperisons
        BuildIn::Smaller => perform_num_comparison!(&args[0], &args[1], |a, b| a < b),
        BuildIn::Bigger => perform_num_comparison!(&args[0], &args[1], |a, b| a > b),
        BuildIn::SmallerEq => perform_num_comparison!(&args[0], &args[1], |a, b| a <= b),
        BuildIn::BiggerEq => perform_num_comparison!(&args[0], &args[1], |a, b| a >= b),
    }
}




fn perform_division(v1: &Value, v2: &Value) -> Result<Value, SigError> {
    match (v1, v2) {
        (Value::Int(i), Value::Int(j)) => {
            if i % j == 0 {
                Ok(Value::Int(i / j))
            } else {
                Ok(Value::Float(*i as f64 / *j as f64))
            }
        }
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1 / f2)),
        (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => {
            Ok(Value::Float(*i as f64 / *f))
        }
        _ => Err(SigError {
            // handle type mismatch error here
        }),
    }
}

fn perform_power(v1: &Value, v2: &Value) -> Result<Value, SigError> {
    match (v1, v2) {
        (Value::Int(i), Value::Int(j)) => {
            if *j >= 0 {
                Ok(Value::Int(i.pow(*j as u32)))
            } else {
                Ok(Value::Float((*i as f64).powf(*j as f64)))
            }
        }
        (Value::Float(f1), Value::Float(f2)) => Ok(Value::Float(f1.powf(*f2))),
        (Value::Int(i), Value::Float(f)) | (Value::Float(f), Value::Int(i)) => {
            Ok(Value::Float((*i as f64).powf(*f)))
        }
        _ => Err(SigError {
            // handle type mismatch error here
        }),
    }
}


fn perform_int_div(v1: &Value, v2: &Value) -> Result<Value, SigError> {
    if let (Value::Int(i), Value::Int(j)) = (v1, v2) {
        if *j == 0 {
            Err(SigError {
                // Handle division by zero error here
            })
        } else {
            Ok(Value::Int(i / j))
        }
    } else {
        Err(SigError {
            // Handle type mismatch error here
        })
    }
}

fn perform_modulo(v1: &Value, v2: &Value) -> Result<Value, SigError> {
    if let (Value::Int(i), Value::Int(j)) = (v1, v2) {
        if *j == 0 {
            Err(SigError {
                // Handle division by zero error here
            })
        } else {
            Ok(Value::Int(i % j))
        }
    } else {
        Err(SigError {
            // Handle type mismatch error here
        })
    }
}

fn perform_bitwise_op(v1: &Value, v2: &Value, op: BuildIn) -> Result<Value, SigError> {
    match (v1, v2) {
        (Value::Int(i), Value::Int(j)) => {
            let result = match op {
                BuildIn::And => i & j,
                BuildIn::Or => i | j,
                BuildIn::Xor => i ^ j,
                _ => unreachable!(),
            };
            Ok(Value::Int(result))
        }
        _ => Err(SigError {
            // handle type mismatch error here
        }),
    }
}

fn perform_logical_op(v1: &Value, v2: &Value, op: BuildIn) -> Result<Value, SigError> {
    let lhs_bool = to_bool(v1);
    let rhs_bool = to_bool(v2);

    let result = match op {
        BuildIn::DoubleAnd => lhs_bool && rhs_bool,
        BuildIn::DoubleOr => lhs_bool || rhs_bool,
        BuildIn::DoubleXor => lhs_bool ^ rhs_bool,
        _ => unreachable!(),
    };

    Ok(Value::Bool(result))
}


macro_rules! define_builtin_function {
    ($($func_name:ident => $op:expr),* $(,)?) => {
        $(
            pub fn $func_name(args: Vec<Value>) -> Result<Value, ErrList> {
                handle_buildin(args, $op)
                    .map_err(|e| Error::Sig(e).to_list())
            }
        )*
    };
}

// Use the macro to generate all the functions with full function names
define_builtin_function!(
    buildin_add => BuildIn::Add,
    buildin_sub => BuildIn::Sub,
    buildin_mul => BuildIn::Mul,
    buildin_div => BuildIn::Div,
    buildin_int_div => BuildIn::IntDiv,
    buildin_modulo => BuildIn::Modulo,
    buildin_pow => BuildIn::Pow,

    buildin_equal => BuildIn::Equal,
    buildin_not_equal => BuildIn::NotEqual,

    buildin_smaller => BuildIn::Smaller,
    buildin_bigger => BuildIn::Bigger,
    buildin_smaller_eq => BuildIn::SmallerEq,
    buildin_bigger_eq => BuildIn::BiggerEq,

    buildin_and => BuildIn::And,
    buildin_or => BuildIn::Or,
    buildin_xor => BuildIn::Xor,

    buildin_double_and => BuildIn::DoubleAnd,
    buildin_double_or => BuildIn::DoubleOr,
    buildin_double_xor => BuildIn::DoubleXor,
);


pub fn get_buildin_function(op: BuildIn) -> fn(Vec<Value>) -> Result<Value, ErrList> {
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
