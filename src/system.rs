use crate::vm::DataFunc;
use std::sync::Arc;
use ast::id::*;

use crate::value::Value;
use crate::stack::ValueStack;

use crate::reporting::*;
// use crate::ir::*;
use crate::basic_ops::*;
use ast::ast::StringTable;
use ast::get_id;

use std::fs::{File, OpenOptions, remove_file};
use std::io::{Read, Write};
use std::fs::{create_dir, read_dir, remove_dir};

pub fn system<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let atom = stack.pop_atom().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    let func = match atom {
        get_id!(":println") => print_fn,
        get_id!(":read_file") => file_read_fn,
        get_id!(":write_file") => file_write_fn,
        get_id!(":delete_file") => file_delete_fn,
        get_id!(":make_dir") => create_dir_fn,
        get_id!(":delete_dir") => delete_dir_fn,
        get_id!(":read_dir") => read_dir_fn,
        get_id!(":type") => get_type,
        _ => {return Err(sig_error());},
    };

    stack.push_value(Value::StaticFunc(func)).map_err(|_| overflow_error())?;
    Ok(())
}

pub fn print_fn<'code>(stack:&mut ValueStack<'code>,table:&StringTable<'code>) -> Result<(),ErrList>{
    let value = stack.pop_value().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    println!("{:?}",to_string_runtime(&value,table) );
    stack.push_value(value).unwrap();

    Ok(())
}

pub fn make_array(v:Vec<Value<'static>>) -> DataFunc {
    let inner = Arc::new(move |stack: &mut ValueStack, _table: &StringTable|{
        let Some(id) = stack.pop_int() else {
            let atom = stack.pop_atom().ok_or_else(sig_error)?;
            match atom {
                get_id!(":len") => {
                    return stack.push_int(v.len().try_into().unwrap()).map_err(|_| overflow_error());
                },
                _ => {return Err(sig_error())},
            }
        };
        stack.pop_terminator().ok_or_else(sig_error)?;

        let id :usize = match id.try_into(){
            Ok(id) =>id,
            Err(_) => {
                return stack.push_nil().map_err(|_| overflow_error());
            },
        };
        match v.get(id) {
            Some(v) => stack.push_value(v.clone()).map_err(|_| overflow_error())?,
            None => stack.push_nil().map_err(|_| overflow_error())?,
        }
        Ok(())
    });
    DataFunc{inner}
}



// File Read Function
pub fn file_read_fn<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let file_name = stack.pop_string().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    let mut file = match File::open(&*file_name) {
        Ok(f) => f,
        Err(_) => {
            stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
            return Ok(());
        }
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
        return Ok(());
    }

    stack.push_string(Arc::new(contents)).map_err(|_| overflow_error())?;
    Ok(())
}

// File Write Function
pub fn file_write_fn<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let file_name = stack.pop_string().ok_or_else(sig_error)?;
    let content = stack.pop_string().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    let mut file = match OpenOptions::new().create(true).write(true).open(&*file_name) {
        Ok(f) => f,
        Err(_) => {
            stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
            return Ok(());
        }
    };

    if file.write_all(content.as_bytes()).is_err() {
        stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
        return Ok(());
    }

    stack.push_atom(get_id!(":ok")).map_err(|_| overflow_error())?;
    Ok(())
}

// File Delete Function
pub fn file_delete_fn<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let file_name = stack.pop_string().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    if remove_file(&*file_name).is_err() {
        stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
        return Ok(());
    }

    stack.push_atom(get_id!(":ok")).map_err(|_| overflow_error())?;
    Ok(())
}


// Directory Creation Function
pub fn create_dir_fn<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let dir_name = stack.pop_string().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    if create_dir(&*dir_name).is_err() {
        stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
        return Ok(());
    }

    stack.push_atom(get_id!(":ok")).map_err(|_| overflow_error())?;
    Ok(())
}

// Directory Deletion Function
pub fn delete_dir_fn<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let dir_name = stack.pop_string().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    if remove_dir(&*dir_name).is_err() {
        stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
        return Ok(());
    }

    stack.push_atom(get_id!(":ok")).map_err(|_| overflow_error())?;
    Ok(())
}

// Directory Read Function
pub fn read_dir_fn<'code>(stack: &mut ValueStack<'code>, _table: &StringTable<'code>) -> Result<(), ErrList> {
    let dir_name = stack.pop_string().ok_or_else(sig_error)?;
    stack.pop_terminator().ok_or_else(sig_error)?;

    let paths = match read_dir(&*dir_name) {
        Ok(paths) => paths,
        Err(_) => {
            stack.push_atom(get_id!(":err")).map_err(|_| overflow_error())?;
            return Ok(());
        }
    };

    let mut entries = Vec::new();
    for entry in paths {
        if let Ok(entry) = entry {
            entries.push(Value::String(Arc::new(entry.path().display().to_string())));
        }
    }

    let list = Value::DataFunc(make_array(entries));
    stack.push_value(list).map_err(|_| overflow_error())?;
    Ok(())
}