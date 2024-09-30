use ast::get_id;
use ast::id::*;
use std::io::{Write,Read};
use std::fs::{read_dir,remove_dir,create_dir,remove_file,OpenOptions,File};

use crate::reporting::*;
use crate::ir::*;
use crate::basic_ops::*;
use ast::ast::StringTable;




pub struct FreeHandle<'ctx> {
    vars : Vec <*mut DynFFI<'ctx>>
}


impl<'ctx> Default for FreeHandle<'ctx> {
    fn default() -> Self {
        Self::new()
    }
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
    let to_string_fn = {create_ffi_to_string(string_table,&mut handle)};

    let file_read_fn = {create_ffi_file_read(string_table,&mut handle)};
    let file_write_fn = {create_ffi_file_write(string_table,&mut handle)};
    let file_delete_fn = {create_ffi_file_delete(string_table,&mut handle)};

    let ffi_create_dir_fn = {create_ffi_create_dir(string_table,&mut handle)};
    let ffi_create_read_fn = {create_ffi_read_dir(string_table,&mut handle)};
    let ffi_create_remove_fn = {create_ffi_remove_dir(string_table,&mut handle)};

    

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
            get_id!(":to_string") => Ok(Value::Func(FunctionHandle::StateFFI(
                to_string_fn,
            ))),
            
            get_id!(":type") => Ok(Value::Func(FunctionHandle::FFI(
                get_type_ffi,
            ))),
            
            get_id!(":read_file") => Ok(Value::Func(FunctionHandle::StateFFI(
                file_read_fn,
            ))),

            get_id!(":write_file") => Ok(Value::Func(FunctionHandle::StateFFI(
                file_write_fn,
            ))),

            get_id!(":delete_file") => Ok(Value::Func(FunctionHandle::StateFFI(
                file_delete_fn,
            ))),


            get_id!(":read_dir") => Ok(Value::Func(FunctionHandle::StateFFI(
                ffi_create_read_fn,
            ))),

            get_id!(":make_dir") => Ok(Value::Func(FunctionHandle::StateFFI(
                ffi_create_dir_fn,
            ))),

            get_id!(":delete_dir") => Ok(Value::Func(FunctionHandle::StateFFI(
                ffi_create_remove_fn,
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
        Ok(args[0].clone())
    };


    handle.make_ref(Box::new(x))
}

fn create_ffi_to_string<'ctx>(table: &'static StringTable<'ctx>,handle:&mut FreeHandle<'ctx>) ->  &'static DynFFI<'ctx> {
    let x  =  |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        // Here we capture the string table reference and print using it
        if args.len()!=1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        match &args[0]{
            Value::String(s) => Ok(Value::String(s.clone())),
            _=> Ok(Value::String(GcPointer::new(to_string(&args[0],table))))
        }
        
    };


    handle.make_ref(Box::new(x))
}




#[allow(dead_code,unused_variables,unreachable_code)]
fn create_ffi_file_read<'ctx>(
    table: &'static StringTable<'ctx>,
    handle: &mut FreeHandle<'ctx>,
) -> &'static DynFFI<'ctx> {
    let x = |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        if args.len() != 1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        let file_name = try_string(&args[0])?;


        #[cfg(test)]{
            panic!("tried to read file... this is not allowed in atomated testing");
        }

        let mut file = match File::open(file_name) {
            Ok(file) => file,
            Err(_e) => return Ok(Value::Atom(get_id!(":err"))),
        };

        let mut contents = String::new();
        if let Err(_e) = file.read_to_string(&mut contents) {
            return Ok(Value::Atom(get_id!(":err")));
        }

        Ok(Value::String(GcPointer::new(contents)))
    };

    handle.make_ref(Box::new(x))
}



#[allow(unused_variables,unreachable_code)]
fn create_ffi_file_write<'ctx>(
    table: &'static StringTable<'ctx>,
    handle: &mut FreeHandle<'ctx>,
) -> &'static DynFFI<'ctx> {
    let x = |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        if args.len() != 2 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        let file_name = try_string(&args[0])?;
        let content = try_string(&args[0])?;


        #[cfg(test)] {
            panic!("tried to write to file... this is not allowed in automated testing");
        }

        let mut file = match OpenOptions::new().create(true).write(true).open(file_name) {
            Ok(file) => file,
            Err(_e) => return Ok(Value::Atom(get_id!(":err"))),
        };

        if let Err(_e) = file.write_all(content.as_bytes()) {
            return Ok(Value::Atom(get_id!(":err")));
        }

        Ok(Value::Atom(get_id!(":ok")))
    };

    handle.make_ref(Box::new(x))
}



#[allow(unused_variables,unreachable_code)]
fn create_ffi_file_delete<'ctx>(
    table: &'static StringTable<'ctx>,
    handle: &mut FreeHandle<'ctx>,
) -> &'static DynFFI<'ctx> {
    let x = |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        if args.len() != 1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        // let path = to_string(&args[0], table);
        let path = try_string(&args[0])?;

        #[cfg(test)] {
            panic!("tried to delete a file or directory... this is not allowed in automated testing");
        }

        // Try deleting as a file first
        if let Err(_file_err) = remove_file(path) {
            // // If it's not a file, try deleting as a directory
            // if let Err(_dir_err) = remove_dir_all(&path) {
            //     return Ok(Value::Atom(get_id!(":err")));
            // }
            return Ok(Value::Atom(get_id!(":err")));
        }

        Ok(Value::Atom(get_id!(":ok")))
    };

    handle.make_ref(Box::new(x))
}



#[allow(unused_variables,unreachable_code)]
fn create_ffi_create_dir<'ctx>(
    table: &'static StringTable<'ctx>,
    handle: &mut FreeHandle<'ctx>,
) -> &'static DynFFI<'ctx> {
    let x = |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        if args.len() != 1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        let dir_name = try_string(&args[0])?;

        #[cfg(test)] {
            panic!("tried to create a directory... this is not allowed in automated testing");
        }

        match create_dir(dir_name) {
            Ok(_) => Ok(Value::Atom(get_id!(":ok"))),
            Err(_) => Ok(Value::Atom(get_id!(":err"))),
        }
    };

    handle.make_ref(Box::new(x))
}



#[allow(unused_variables,unreachable_code)]
fn create_ffi_remove_dir<'ctx>(
    table: &'static StringTable<'ctx>,
    handle: &mut FreeHandle<'ctx>,
) -> &'static DynFFI<'ctx> {
    let x = |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        if args.len() != 1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        let dir_name = try_string(&args[0])?;

        #[cfg(test)] {
            panic!("tried to remove a directory... this is not allowed in automated testing");
        }

        match remove_dir(dir_name) {
            Ok(_) => Ok(Value::Atom(get_id!(":ok"))),
            Err(_) => Ok(Value::Atom(get_id!(":err"))),
        }
    };

    handle.make_ref(Box::new(x))
}



#[allow(unused_variables,unreachable_code)]
fn create_ffi_read_dir<'ctx>(
    table: &'static StringTable<'ctx>,
    handle: &mut FreeHandle<'ctx>,
) -> &'static DynFFI<'ctx> {
    let x = |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
        if args.len() != 1 {
            return Err(Error::Sig(SigError {}).to_list());
        }

        let dir_name = try_string(&args[0])?;

        #[cfg(test)] {
            panic!("tried to read directory... this is not allowed in automated testing");
        }

        let paths = match read_dir(dir_name) {
            Ok(paths) => paths,
            Err(_) => return Ok(Value::Atom(get_id!(":err"))),
        };

        let mut entries = Vec::new();
        for path in paths {
            if let Ok(entry) = path {
                entries.push(Value::String(GcPointer::new(entry.path().display().to_string())));
            }
        }
        let list = move |args: Vec<Value<'ctx>>| -> Result<Value<'ctx>, ErrList> {
            if args.len() != 1 {
                return Err(Error::Sig(SigError {}).to_list());
            }
            match entries.get(try_int(&args[0])? as usize) {
                None => Ok(Value::Nil),
                Some(x) => Ok(x.clone()),
            }
        };

        Ok(Value::Func(FunctionHandle::DataFFI(GcPointer::new(list))))
    };

    handle.make_ref(Box::new(x))
}
