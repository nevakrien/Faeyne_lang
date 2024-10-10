#![allow(clippy::result_unit_err)]


use std::sync::Weak;
use std::sync::Arc;

#[derive(Clone,PartialEq,Debug)]
pub struct NativeFunction{
    //holds raw code
}

#[derive(Clone,Debug)]
#[repr(u32)] //optimized for 64bit architctures
pub enum Value {
    Nil=1,
    Bool(bool)=3,
    Int(i64),
    Float(f64),
    Atom(u32),
    String(Arc<String>),
    Func(Arc<NativeFunction>),
    WeakFunc(Weak<NativeFunction>),
    
}



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

    pub fn set(&mut self, id: usize, val: Value) -> Result<(), ()> {
        if let Some(elem) = self.data.get_mut(id) {
            *elem = Some(val);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get(&self, id: usize) -> Option<Value> {
        self.data.get(id)?.clone()
    }

    pub fn get_debug_id(&self, id: usize) -> Option<u32> {
        self.names.get(id).copied()
    }
}

