use crate::reporting::*;
use crate::ir::*;
use crate::basic_ops::*;
use crate::ast::StringTable;

//IMPORTANT first value is assumed to not be a var
//if this is broken then Lamda match statments will break
//we have it as the nil type so that we can do easy nil checks

pub const NIL_ID: usize = 0;
pub const BOOL_ID: usize = 1;
pub const STRING_ID: usize = 2;
pub const INT_ID: usize = 3;
pub const FLOAT_ID: usize = 4;
pub const ATOM_ID: usize = 5;
pub const FUNC_ID: usize = 6;
pub const UNDERSCORE_ID: usize = 7;
pub const PRINTLN_ID: usize = 8;
pub const TYPE_ATOM_ID: usize = 9;

pub fn preload_table(table: &mut StringTable) {
    assert_eq!(table.get_id(":nil"), NIL_ID);
    assert_eq!(table.get_id(":bool"), BOOL_ID);
    assert_eq!(table.get_id(":string"), STRING_ID);
    assert_eq!(table.get_id(":int"), INT_ID);
    assert_eq!(table.get_id(":float"), FLOAT_ID);
    assert_eq!(table.get_id(":atom"), ATOM_ID);
    assert_eq!(table.get_id(":func"), FUNC_ID);
    assert_eq!(table.get_id("_"), UNDERSCORE_ID);
    assert_eq!(table.get_id(":println"), PRINTLN_ID);
    assert_eq!(table.get_id(":type"), TYPE_ATOM_ID);
}

#[macro_export]
macro_rules! get_id {
    (":nil") => { NIL_ID };
    (":bool") => { BOOL_ID };
    (":string") => { STRING_ID };
    (":int") => { INT_ID };
    (":float") => { FLOAT_ID };
    (":atom") => { ATOM_ID };
    (":func") => { FUNC_ID };
    ("_") => { UNDERSCORE_ID };
    (":println") => { PRINTLN_ID };
    (":type") => { TYPE_ATOM_ID };
    ($other:expr) => { // Fallback to the runtime version if it's not predefined
        $other
    };
}

pub fn get_system(string_table: &'static StringTable) -> Value {
    let print_fn = create_ffi_println(string_table);

    
    let x = move |args: Vec<Value>| -> Result<Value, ErrList> {
        if args.len() != 1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        let atom = match args[0] {
            Value::Atom(id) => id,
            _ => {
                return Err(Error::Sig(SigError {}).to_list());
            }
        };

        match atom {
            get_id!(":println") => Ok(Value::Func(FunctionHandle::StateFFI(
                print_fn,
            ))),
            get_id!(":type") => Ok(Value::Func(FunctionHandle::FFI(
                get_type_ffi,
            ))),
            _ => Err(Error::Sig(SigError {}).to_list()),
        }
    };

    let b :Box<dyn Fn(Vec<Value>) -> Result<Value, ErrList>>= Box::new(x); 


    Value::Func(FunctionHandle::StateFFI(Box::leak(b)))
}

fn create_ffi_println(table: &'static StringTable) -> &'static dyn Fn(Vec<Value>) -> Result<Value, ErrList> {
    let x  = move |args: Vec<Value>| -> Result<Value, ErrList> {
        // Here we capture the string table reference and print using it
        if args.len()!=1 {
        	return Err(Error::Sig(SigError {}).to_list());
        }
        match &args[0] {
        	Value::Atom(id) => {println!("{}", table.get_string(*id).unwrap());},
        	Value::Int(x) => {println!("{:?}", x);},
        	Value::Float(x) => {println!("{:?}", x);},
        	Value::String(s) => {println!("{}", s);},
        	_ => {println!("{:?}", args[0]);}
        }
        
        Ok(Value::Nil)
    };

    let b :Box<dyn Fn(Vec<Value>) -> Result<Value, ErrList>>= Box::new(x);

    Box::leak(b)
}

// pub fn get_system() -> Value {
// 	Value::Func(FunctionHandle::FFI(ffi_system))
// }

// fn ffi_println(args: Vec<Value>) -> Result<Value, ErrList> {
// 	println!("{:?}",args);
//     Ok(Value::Nil)
// }

// fn ffi_system(args: Vec<Value>) -> Result<Value, ErrList> {
// 	if args.len() != 1 {
// 		return Err(Error::Sig(SigError{}).to_list());
// 	}

// 	let atom = match args[0]{
// 		Value::Atom(id) => id,
// 		_ => {return Err(Error::Sig(SigError{}).to_list());}
// 	};

// 	match atom {
// 		get_id!(":println") => Ok(Value::Func(FunctionHandle::FFI(ffi_println))),
// 		_ => Err(Error::Sig(SigError{}).to_list())
// 	}

// }