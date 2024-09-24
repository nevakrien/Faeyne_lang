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
pub const MAIN_ID: usize = 10;

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
    assert_eq!(table.get_id("main"), MAIN_ID);
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
    ("main") => { MAIN_ID };
    ($other:expr) => { // Fallback to the runtime version if it's not predefined
        $other
    };
}



pub struct FreeHandle<'ctx> {
    vars : Vec <*mut DynFFI<'ctx>>
}


impl<'ctx> FreeHandle<'ctx>{
    pub fn new() -> Self {
        FreeHandle{vars:Vec::new()}
    }

    pub fn make_ref(&mut self,x:Box<DynFFI<'ctx>>) -> &'static DynFFI<'ctx>{
        let ptr = Box::into_raw(x);
        self.vars.push(ptr);
        unsafe{Box::leak(Box::from_raw(ptr))}
    }

    pub unsafe fn free(self) {
        for p in self.vars.into_iter().rev() {
            {
                _ = Box::from_raw(p);
            }
        }
    }
}

pub fn get_system<'ctx>(string_table: &'static StringTable<'ctx>) -> (Value<'ctx>,FreeHandle<'ctx>) {
    let mut handle = FreeHandle::new();
    let print_fn = {create_ffi_println(string_table,&mut handle)};

    

    let x =  |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
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


    let leaked = handle.make_ref(Box::new(x));
    (Value::Func(FunctionHandle::StateFFI(leaked)),handle)
    
}

fn create_ffi_println<'ctx>(table: &'static StringTable<'ctx>,handle:&mut FreeHandle<'ctx>) ->  &'static DynFFI<'ctx> {
    let x  =  |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        // Here we capture the string table reference and print using it
        if args.len()!=1 {
        	return Err(Error::Sig(SigError {}).to_list());
        }

        println!("{}", to_string(&args[0],table));
        Ok(Value::Nil)
    };


    handle.make_ref(Box::new(x))
}
