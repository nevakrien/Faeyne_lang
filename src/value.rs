use core::hash::Hasher;
use core::hash::Hash;
use crate::vm::StaticFunc;
use crate::vm::FuncData;
use std::sync::Weak;
use std::sync::Arc;

#[derive(Clone,Debug)]
#[repr(u32)] //optimized for 64bit architctures
pub enum Value<'code> {
    //these indecies are made to match with the ValueTag
    Nil=2,
    Bool(bool)=0,//bool is 0/1 to make loading a bool type easier on registers.
    Int(i64)=3,
    Float(f64)=4,
    Atom(u32)=5,
    String(Arc<String>)=6,
    Func(Arc<FuncData<'code>>)=7,
    WeakFunc(Weak<FuncData<'code>>)=8,
    StaticFunc(StaticFunc)=9,
    
}

impl PartialEq for Value<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
       
        match (self, other) {
            (Value::WeakFunc(weak_a), Value::WeakFunc(weak_b)) => weak_a.as_ptr()==weak_b.as_ptr(),
            (Value::WeakFunc(weak), Value::Func(func)) | (Value::Func(func), Value::WeakFunc(weak)) => weak.as_ptr()==Arc::as_ptr(func),
            (Value::Func(a), Value::Func(b)) => Arc::as_ptr(a) == Arc::as_ptr(b),
            (Value::StaticFunc(a),Value::StaticFunc(b)) => a==b,
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b || a.is_nan() && b.is_nan(),
            (Value::Atom(a), Value::Atom(b)) => a == b,
            (Value::String(a), Value::String(b)) => Arc::ptr_eq(a, b) || *a == *b,
            _ => false,
        }
    }
}

impl Eq for Value<'_> {}

impl Hash for Value<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => state.write_u8(0),
            Value::Bool(b) => b.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Float(f) => {
                let normalized = if f.is_nan() {
                    f64::NAN.to_bits()
                } else if *f==0.0 {
                    0
                } else{
                    f.to_bits()
                };
                state.write_u64(normalized);
            }
            Value::Atom(a) => a.hash(state),
            Value::String(arc) => {
                    let string_content = &**arc;  // Dereference the `Arc` to get the string
                    string_content.hash(state);   // Hash the string itself
                },
            Value::Func(arc) => {
                let ptr = Arc::as_ptr(arc);
                state.write_usize(ptr as usize);
            }
            Value::WeakFunc(weak) => {
                let ptr = weak.as_ptr();
                state.write_usize(ptr as usize);
            }
            Value::StaticFunc(func) => func.hash(state),
        }
    }
}


#[test]
fn test_value_partial_eq() {
    let vars = VarTable::default();
    let mut_vars = VarTable::default();
    let func_data = FuncData::new(vars,&mut_vars,&[],0);

    let value_nil = Value::Nil;
    let value_bool_true = Value::Bool(true);
    let value_bool_false = Value::Bool(false);
    let value_int_42 = Value::Int(42);
    let value_float = Value::Float(6.9);
    let value_atom = Value::Atom(123);
    let value_string = Value::String(Arc::new(String::from("Hello")));
    let func = Arc::new(func_data);
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
pub struct VarTable<'code> {
    pub data: Vec<Option<Value<'code>>>,
    pub names: Vec<u32>,
}

impl<'code> VarTable<'code>  {
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

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

    pub fn set(&mut self, id: usize, val: Value<'code>) -> Result<(), MissingID> {
        if let Some(elem) = self.data.get_mut(id) {
            *elem = Some(val);
            Ok(())
        } else {
            Err(MissingID)
        }
    }

    pub fn get(&self, id: usize) -> Option<Value<'code>> {
        self.data.get(id)?.clone()
    }

    pub fn get_debug_id(&self, id: usize) -> Option<u32> {
        self.names.get(id).copied()
    }
}

