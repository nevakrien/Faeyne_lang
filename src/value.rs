// #![allow(clippy::result_unit_err)]


use crate::vm::StaticFunc;
use crate::vm::FuncData;
use std::sync::Weak;
use std::sync::Arc;

// #[derive(Clone,PartialEq,Debug)]
// pub struct FuncData{
//     //holds raw code
// }


#[derive(Clone,Debug)]
#[repr(u32)] //optimized for 64bit architctures
pub enum Value {
    //these indecies are made to match with the ValueTag
    Nil=2,
    Bool(bool)=0,//bool is 0/1 to make loading a bool type easier on registers.
    Int(i64)=3,
    Float(f64)=4,
    Atom(u32)=5,
    String(Arc<String>)=6,
    Func(Arc<FuncData>)=7,
    WeakFunc(Weak<FuncData>)=8,
    StaticFunc(StaticFunc)=9,
    
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::WeakFunc(weak_a), Value::WeakFunc(weak_b)) => weak_a.ptr_eq(weak_b),
            (Value::WeakFunc(weak), Value::Func(func)) | (Value::Func(func), Value::WeakFunc(weak)) => weak.ptr_eq(&Arc::downgrade(func)),
            (Value::Func(a), Value::Func(b)) => Arc::ptr_eq(a, b),
            (Value::StaticFunc(a),Value::StaticFunc(b)) => a==b,
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Atom(a), Value::Atom(b)) => a == b,
            (Value::String(a), Value::String(b)) => Arc::ptr_eq(a, b) || *a == *b,
            _ => false,
        }
    }
}

#[test]
fn test_value_partial_eq() {
    let value_nil = Value::Nil;
    let value_bool_true = Value::Bool(true);
    let value_bool_false = Value::Bool(false);
    let value_int_42 = Value::Int(42);
    let value_float = Value::Float(6.9);
    let value_atom = Value::Atom(123);
    let value_string = Value::String(Arc::new(String::from("Hello")));
    let func = Arc::new(FuncData::default());
    let value_func = Value::Func(func.clone());
    let value_weak_func = Value::WeakFunc(Arc::downgrade(&func));

    assert_eq!(value_nil, Value::Nil);
    assert_ne!(value_bool_true, value_bool_false);
    assert_eq!(value_int_42, Value::Int(42));
    assert_ne!(value_float, Value::Float(2.71));
    assert_eq!(value_atom, Value::Atom(123));
    assert_eq!(value_string, Value::String(Arc::new(String::from("Hello"))));
    assert_eq!(value_func, Value::Func(func.clone()));
    assert_eq!(value_weak_func, Value::WeakFunc(Arc::downgrade(&func)));
    assert_eq!(value_weak_func, value_func);
}

#[derive(Clone,Debug,PartialEq,Copy)]
pub struct MissingID;

#[derive(Clone,Debug,PartialEq)]
#[derive(Default)]
pub struct VarTable {
    data: Vec<Option<Value>>,
    pub names: Vec<u32>,
}

impl VarTable {
    pub fn clear(&mut self) {
        self.data.clear();
        self.names.clear();
    }

    pub fn reset_data(&mut self) {
        self.data.iter_mut().for_each(|x| *x=None);
    }

    pub fn truncate(&mut self, n: usize) {
        self.data.truncate(n);
        self.names.truncate(n);
    }

    pub fn add_ids(&mut self, ids: &[u32]) {
        self.names.reserve(ids.len());
        self.data.reserve(ids.len());

        self.names.extend(ids.iter());
        self.data.extend(ids.iter().map(|_| None));
    }

    pub fn set(&mut self, id: usize, val: Value) -> Result<(), MissingID> {
        if let Some(elem) = self.data.get_mut(id) {
            *elem = Some(val);
            Ok(())
        } else {
            Err(MissingID)
        }
    }

    pub fn get(&self, id: usize) -> Option<Value> {
        self.data.get(id)?.clone()
    }

    pub fn get_debug_id(&self, id: usize) -> Option<u32> {
        self.names.get(id).copied()
    }
}

#[derive(Debug)]
pub struct VarTableView<'a> {
    pub names: &'a [u32],
    data: Vec<Option<Value>>,
}

impl<'a> VarTableView<'a> {
    pub fn new(var_table: &'a VarTable) -> Self {
        VarTableView {
            names: &var_table.names,
            data: var_table.data.clone()
        }
    }

    pub fn set(&mut self, id: usize, val: Value) -> Result<(), MissingID> {
        if let Some(elem) = self.data.get_mut(id) {
            *elem = Some(val);
            Ok(())
        } else {
            Err(MissingID)
        }
    }

    pub fn get(&self, id: usize) -> Option<Value> {
        self.data.get(id)?.clone()
    }

    pub fn get_debug_id(&self, id: usize) -> Option<u32> {
        self.names.get(id).copied()
    }

    pub fn reset_data(&mut self) {
        self.data.iter_mut().for_each(|x| *x = None);
    }
}
