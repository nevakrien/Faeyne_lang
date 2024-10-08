use core::num::NonZeroU32;
use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use crate::stack::{Aligned, Stack};


// Enum for value types
#[repr(u32)] //used because the stack is 64bits aligned and this lets us cram a u32 id next to this baby
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    //0 is undefined
    Nil = 1,
    BoolTrue,
    BoolFalse,
    Atom,
    String,
    Int,
    Float,
    Func,
}

impl TryFrom<u32> for ValueType {
    type Error = ();
    #[inline(always)]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ValueType::Nil),
            2 => Ok(ValueType::BoolTrue),
            3 => Ok(ValueType::BoolFalse),
            4 => Ok(ValueType::Atom),
            5 => Ok(ValueType::String),
            6 => Ok(ValueType::Int),
            7 => Ok(ValueType::Float),
            8 => Ok(ValueType::Func),
            _ => Err(()),
        }
    }
}

#[repr(u32)] //this is to fit nicely with ValueType
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum IRValue {
    Nil = ValueType::Nil as u32,
    Bool(bool),
    Atom(u32),
    String(u32),
    Int(i64),
    Float(f64),
    Func(u32),
}

// Trait for specialized Stack operations for IRValue
pub trait ValueStack {
    //note that push_grow is 2x slower than push. so its recommended to ensure capacity and then push with unwraps.
    fn pop_value(&mut self) -> Result<IRValue, ()>;
    fn pop_nil(&mut self) -> Result<(), ()>;
    fn pop_atom(&mut self) -> Result<u32, ()>;
    fn pop_string(&mut self) -> Result<u32, ()>;
    fn pop_int(&mut self) -> Result<i64, ()>;
    fn pop_float(&mut self) -> Result<f64, ()>;
    
    fn push_value(&mut self, value: &IRValue) -> Result<(), ()>;    
    fn push_nil(&mut self) -> Result<(), ()>;
    fn push_bool(&mut self, val: bool) -> Result<(), ()>;
    fn push_atom(&mut self, id: u32) -> Result<(), ()>;
    fn push_string(&mut self, id: u32) -> Result<(), ()>;
    fn push_int(&mut self, val: i64) -> Result<(), ()>;
    fn push_float(&mut self, val: f64) -> Result<(), ()>;
    fn push_func(&mut self, id: u32) -> Result<(), ()>;

    fn push_grow_value(&mut self, value: &IRValue);    
    fn push_grow_nil(&mut self);
    fn push_grow_bool(&mut self, val: bool);
    fn push_grow_atom(&mut self, id: u32);
    fn push_grow_string(&mut self, id: u32);
    fn push_grow_int(&mut self, val: i64);
    fn push_grow_float(&mut self, val: f64);
    fn push_grow_func(&mut self, id: u32);
}

impl ValueStack for Stack {
    #[inline]
    fn push_value(&mut self, value: &IRValue) -> Result<(), ()> {
        match value {
            IRValue::Nil => self.push_nil(),
            IRValue::Bool(val) => self.push_bool(*val),
            IRValue::Atom(id) => self.push_atom(*id),
            IRValue::String(id) => self.push_string(*id),
            IRValue::Int(val) => self.push_int(*val),
            IRValue::Float(val) => self.push_float(*val),
            IRValue::Func(id) => self.push_func(*id),
        }
    }

    #[inline]
    fn push_grow_value(&mut self, value: &IRValue){
        match value {
            IRValue::Nil => self.push_grow_nil(),
            IRValue::Bool(val) => self.push_grow_bool(*val),
            IRValue::Atom(id) => self.push_grow_atom(*id),
            IRValue::String(id) => self.push_grow_string(*id),
            IRValue::Int(val) => self.push_grow_int(*val),
            IRValue::Float(val) => self.push_grow_float(*val),
            IRValue::Func(id) => self.push_grow_func(*id),
        }
    }


    #[inline]
    fn pop_value(&mut self) -> Result<IRValue, ()> {
        unsafe {
            match self.pop::<u64>() {
                Some(aligned_tag) => {
                    let tag = aligned_tag.to_inner();
                    match ValueType::try_from((tag & 0xFFFFFFFF) as u32) {
                        Ok(ValueType::Nil) => Ok(IRValue::Nil),
                        Ok(ValueType::BoolTrue) => Ok(IRValue::Bool(true)),
                        Ok(ValueType::BoolFalse) => Ok(IRValue::Bool(false)),
                        Ok(ValueType::Atom) => Ok(IRValue::Atom((tag >> 32) as u32)),
                        Ok(ValueType::String) => Ok(IRValue::String((tag >> 32) as u32)),
                        Ok(ValueType::Func) => Ok(IRValue::Func((tag >> 32) as u32)),
                        Ok(ValueType::Int) => {
                            let data = match self.pop::<u64>() {
                                Some(aligned_data) => aligned_data,
                                None => return Err(()),
                            };
                            Ok(IRValue::Int(data.to_inner() as i64))
                        }
                        Ok(ValueType::Float) => {
                            let data = match self.pop::<u64>() {
                                Some(aligned_data) => aligned_data,
                                None => return Err(()),
                            };
                            Ok(IRValue::Float(f64::from_bits(data.to_inner())))
                        }
                        _ => Err(()),
                    }
                }
                None => Err(()),
            }
        }
    }

    #[inline]
    fn pop_nil(&mut self) -> Result<(), ()> {
        if let Some(aligned) = unsafe { self.pop::<u64>() } {
            let tag = aligned.to_inner();
            if tag == ValueType::Nil as u64 {
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_atom(&mut self) -> Result<u32, ()> {
        if let Some(aligned) = unsafe { self.pop::<u64>() } {
            let tag = aligned.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::Atom as u32 {
                Ok((tag >> 32) as u32)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_string(&mut self) -> Result<u32, ()> {
        if let Some(aligned) = unsafe { self.pop::<u64>() } {
            let tag = aligned.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::String as u32 {
                Ok((tag >> 32) as u32)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_int(&mut self) -> Result<i64, ()> {
        if let Some(aligned_tag) = unsafe { self.pop::<u64>() } {
            let tag = aligned_tag.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::Int as u32 {
                if let Some(aligned_data) = unsafe { self.pop::<u64>() } {
                    Ok(aligned_data.to_inner() as i64)
                } else {
                    Err(())
                }
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn pop_float(&mut self) -> Result<f64, ()> {
        if let (Some(aligned_tag), Some(aligned_data)) = (unsafe { self.pop::<u64>() }, unsafe { self.pop::<u64>() }) {
            let tag = aligned_tag.to_inner();
            if (tag & 0xFFFFFFFF) as u32 == ValueType::Float as u32 {
                Ok(f64::from_bits(aligned_data.to_inner()))
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    #[inline]
    fn push_nil(&mut self) -> Result<(), ()> {
        let tag = ValueType::Nil as u64;
        let aligned = Aligned::new(tag);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_nil(&mut self) {
        let tag = ValueType::Nil as u64;
        let aligned = Aligned::new(tag);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_bool(&mut self, val: bool) -> Result<(), ()> {
        let tag = if val {
            ValueType::BoolTrue as u64
        } else {
            ValueType::BoolFalse as u64
        };
        let aligned = Aligned::new(tag);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_bool(&mut self, val: bool)  {
        let tag = if val {
            ValueType::BoolTrue as u64
        } else {
            ValueType::BoolFalse as u64
        };
        let aligned = Aligned::new(tag);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_atom(&mut self, id: u32) -> Result<(), ()> {
        let packed_data = (id as u64) << 32 | ValueType::Atom as u64;
        let aligned = Aligned::new(packed_data);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_atom(&mut self, id: u32) {
        let packed_data = (id as u64) << 32 | ValueType::Atom as u64;
        let aligned = Aligned::new(packed_data);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_string(&mut self, id: u32) -> Result<(), ()> {
        let packed_data = (id as u64) << 32 | ValueType::String as u64;
        let aligned = Aligned::new(packed_data);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_string(&mut self, id: u32) {
        let packed_data = (id as u64) << 32 | ValueType::String as u64;
        let aligned = Aligned::new(packed_data);
        self.push_grow(&aligned)
    }

    #[inline]
    fn push_int(&mut self, val: i64) -> Result<(), ()> {
        let tag = ValueType::Int as u64;
        let aligned_data = Aligned::new(val as u64);
        let aligned_tag = Aligned::new(tag);
        self.push(&aligned_data)?;
        self.push(&aligned_tag)
    }

    #[inline]
    fn push_grow_int(&mut self, val: i64)  {
        let tag = ValueType::Int as u64;
        let aligned_data = Aligned::new(val as u64);
        let aligned_tag = Aligned::new(tag);
        self.push_grow(&aligned_data);
        self.push_grow(&aligned_tag);
    }

    #[inline]
    fn push_float(&mut self, val: f64) -> Result<(), ()> {
        let tag = ValueType::Float as u64;
        let aligned_data = Aligned::new(val.to_bits() as u64);
        let aligned_tag = Aligned::new(tag);
        self.push(&aligned_data)?;
        self.push(&aligned_tag)
    }

    #[inline]
    fn push_grow_float(&mut self, val: f64) {
        let tag = ValueType::Float as u64;
        let aligned_data = Aligned::new(val.to_bits() as u64);
        let aligned_tag = Aligned::new(tag);
        self.push_grow(&aligned_data);
        self.push_grow(&aligned_tag);
    }

    #[inline]
    fn push_func(&mut self, id: u32) -> Result<(), ()> {
        let packed_data = (id as u64) << 32 | ValueType::Func as u64;
        let aligned = Aligned::new(packed_data);
        self.push(&aligned)
    }

    #[inline]
    fn push_grow_func(&mut self, id: u32) {
        let packed_data = (id as u64) << 32 | ValueType::Func as u64;
        let aligned = Aligned::new(packed_data);
        self.push_grow(&aligned);
    }
}



#[test]
fn test_value_stack() {
    let mut stack: Stack = Stack::with_capacity(100);

    // Push multiple items onto the stack
    let value_nil = IRValue::Nil;
    let value_bool = IRValue::Bool(true);
    let value_atom = IRValue::Atom(123);
    let value_string = IRValue::String(456);
    let value_func = IRValue::Func(789);
    let value_int = IRValue::Int(42);
    let value_float = IRValue::Float(3.14);

    stack.push_value(&value_nil).unwrap();
    stack.push_value(&value_bool).unwrap();
    stack.push_value(&value_atom).unwrap();
    stack.push_value(&value_string).unwrap();
    stack.push_value(&value_func).unwrap();
    stack.push_value(&value_int).unwrap();
    stack.push_value(&value_float).unwrap();

    // Pop items one by one and check them
    let popped_value = stack.pop_value().expect("Failed to pop Value::Float");
    assert_eq!(popped_value, IRValue::Float(3.14));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Int");
    assert_eq!(popped_value, IRValue::Int(42));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Func");
    assert_eq!(popped_value, IRValue::Func(789));

    let popped_value = stack.pop_value().expect("Failed to pop Value::String");
    assert_eq!(popped_value, IRValue::String(456));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Atom");
    assert_eq!(popped_value, IRValue::Atom(123));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Bool");
    assert_eq!(popped_value, IRValue::Bool(true));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Nil");
    assert_eq!(popped_value, IRValue::Nil);

    // Additional tests for specific pop methods
    stack.push_value(&value_atom).unwrap();
    stack.push_value(&value_string).unwrap();
    stack.push_value(&value_int).unwrap();

    let _popped_int = stack.pop_int().unwrap();
    let _popped_string = stack.pop_string().unwrap();
    let popped_atom = stack.pop_atom().unwrap();
    assert_eq!(popped_atom, 123);
}

pub struct VarTable {
    data: Vec<Option<IRValue>>,
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

    pub fn set(&mut self, id: usize, val: IRValue) -> Result<(), ()> {
        if let Some(elem) = self.data.get_mut(id) {
            *elem = Some(val);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get(&self, id: usize) -> Option<IRValue> {
        self.data.get(id)?.clone()
    }

    pub fn get_debug_id(&self, id: usize) -> Option<u32> {
        self.names.get(id).copied()
    }
}

pub struct Scope<'a> {
    pub table: &'a mut VarTable,
    base_size: usize,
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        // println!("calling drop length of {}", self.base_size);
        self.table.truncate(self.base_size);
    }
}

impl<'a> Scope<'a> {
    pub fn new_global(table: &'a mut VarTable) -> Self {
        let base_size = table.names.len();
        Scope { table, base_size }
    }

    pub fn add_scope(&mut self, ids: &[u32]) -> Scope {
        let base_size = self.table.names.len();
        self.table.add_ids(ids);
        Scope {
            table: self.table,
            base_size,
        }
    }

    pub fn consume(self) -> &'a mut VarTable{
        //we need to hold a pointer to the object AFTER its droped (without freeing it)
        //and we also need the borrow checker to know its legal
        //which is why unsafecell is needed
        //using memforget would break
        let cell = UnsafeCell::new(self);
        let manual = ManuallyDrop::new(cell);
        let ptr = manual.get();
        
        unsafe{&mut *(*ptr).table}
    }


    pub fn set(&mut self, id: usize, val: IRValue) -> Result<(), ()> {
        self.table.set(id, val)
    }

    pub fn get(&self, id: usize) -> Option<IRValue> {
        self.table.get(id)
    }

    pub fn get_debug_id(&self, id: usize) -> Option<u32> {
        self.table.get_debug_id(id)
    }

    pub fn len(&self) -> usize {
        self.table.names.len()
    }
}

#[test]
fn test_scope_add_and_remove() {
    let mut var_table = VarTable {
        data: Vec::new(),
        names: Vec::new(),
    };

    // Create a global scope
    let mut global_scope = Scope::new_global(&mut var_table);

    assert_eq!(global_scope.len(), 0);

    // Add a new nested scope with some variables
    {
        let ids = vec![1, 2, 3];
        let mut nested_scope = global_scope.add_scope(&ids);

        // Verify that the IDs and corresponding data are added
        assert_eq!(nested_scope.len(), 3);

        // Set some values for these variables
        nested_scope.set(0, IRValue::Int(42)).unwrap();
        nested_scope.set(1, IRValue::Bool(true)).unwrap();
        nested_scope.set(2, IRValue::String(123)).unwrap();

        // Verify the values are set correctly
        assert_eq!(nested_scope.get(0), Some(IRValue::Int(42)));
        assert_eq!(nested_scope.get(1), Some(IRValue::Bool(true)));
        assert_eq!(nested_scope.get(2), Some(IRValue::String(123)));

        std::mem::drop(nested_scope);
    } // The nested scope ends here, dropping it and clearing its entries.

    // After the nested scope is dropped, the size of the table should return to its previous value
    assert_eq!(global_scope.len(), 0);

    // Add another nested scope to test again
    {
        let ids = vec![4, 5];
        let mut nested_scope_2 = global_scope.add_scope(&ids);

        // Verify that the IDs and corresponding data are added
        assert_eq!(nested_scope_2.len(), 2);

        // Set some values for these variables
        nested_scope_2.set(0, IRValue::Float(3.14)).unwrap();
        nested_scope_2.set(1, IRValue::Atom(456)).unwrap();

        // Verify the values are set correctly
        assert_eq!(nested_scope_2.get(0), Some(IRValue::Float(3.14)));
        assert_eq!(nested_scope_2.get(1), Some(IRValue::Atom(456)));
    } // The second nested scope ends here, dropping it and clearing its entries.

    // After the second nested scope is dropped, the size of the table should return to its previous value
    assert_eq!(global_scope.len(), 0);
}

#[test]
fn test_scope_consume() {
    let mut var_table = VarTable {
        data: Vec::new(),
        names: Vec::new(),
    };

    // Create a global scope
    let mut global_scope = Scope::new_global(&mut var_table);

    // Add a new nested scope with some variables
    {
        let ids = vec![1, 2, 3];
        let mut nested_scope = global_scope.add_scope(&ids);

        // Set some values for these variables
        nested_scope.set(0, IRValue::Int(42)).unwrap();
        nested_scope.set(1, IRValue::Bool(true)).unwrap();
        nested_scope.set(2, IRValue::String(123)).unwrap();

        // Consume the nested scope and get back the mutable reference to VarTable
        let returned_table = nested_scope.consume();

        // Verify that the values are still present in the returned table
        assert_eq!(returned_table.get(0), Some(IRValue::Int(42)));
        assert_eq!(returned_table.get(1), Some(IRValue::Bool(true)));
        assert_eq!(returned_table.get(2), Some(IRValue::String(123)));
    }

    // After consuming the nested scope, the size of the global scope should not be affected
    assert_eq!(global_scope.len(), 3);

    // Verify that the values are still present in the returned table
    assert_eq!(global_scope.get(0), Some(IRValue::Int(42)));
    assert_eq!(global_scope.get(1), Some(IRValue::Bool(true)));
    assert_eq!(global_scope.get(2), Some(IRValue::String(123)));
}



pub struct Registry<T: Clone> {
    values: Vec<Option<(NonZeroU32, T)>>,
    cur_id: NonZeroU32,
}

pub type StringRegistry = Registry<String>;

const LEFT_PROBS: usize = 4; // Fixed constant for probing backwards
const RIGHT_PROBS: usize = 5; // Fixed constant for probing forwards

impl<T: Clone> Registry<T> {
    pub fn new(cap: usize) -> Self {
        Registry {
            values: vec![None; cap],
            cur_id: NonZeroU32::new(1).unwrap(),
        }
    }

    fn hash(&self, id: NonZeroU32) -> usize {
        (u32::from(id) - 1) as usize % self.values.len()
    }

    fn resize(&mut self) {
        let new_capacity = self.values.len() * 2;
        let old_values = std::mem::replace(&mut self.values, vec![None; new_capacity]);

        for entry in old_values.into_iter().flatten() {
            let (id, value) = entry;
            self.insert_internal(id, value); // Rehash and insert existing entries
        }
    }

    fn insert_internal(&mut self, id: NonZeroU32, value: T) {
        let idx = self.hash(id);

        // Quick immediate check for the initial index
        if let None = &self.values[idx] {
            self.values[idx] = Some((id, value));
            return;
        }

        // First probe backwards
        for i in 0..LEFT_PROBS {
            let idx = (idx + self.values.len() - i) % self.values.len(); // Move backwards with wrap around
            if let None = &self.values[idx] {
                self.values[idx] = Some((id, value));
                return;
            }
        }

        // Then probe forwards starting from 1
        for i in 1..=RIGHT_PROBS {
            let idx = (idx + i) % self.values.len();
            if let None = &self.values[idx] {
                self.values[idx] = Some((id, value));
                return;
            }
        }

        // If insertion fails within max_probes, resize and retry
        self.resize();
        self.insert_internal(id, value);
    }

    pub fn insert(&mut self, value: T) -> u32 {
        let id = self.cur_id;
        self.cur_id = NonZeroU32::new(u32::from(id) + 1).unwrap();
        self.insert_internal(id, value);
        id.into()
    }

    pub fn get(&self, id_raw: u32) -> Option<&T> {
        let id = NonZeroU32::new(id_raw).expect("WRONG ID PASSED");
        let idx = self.hash(id);

        // Quick immediate check for the initial index
        match &self.values[idx] {
            Some((existing_id, value)) if *existing_id == id => {
                return Some(value);
            }
            None => return None, // Not found
            _ => {}
        }

        // First probe backwards
        for i in 0..LEFT_PROBS {
            let idx = (idx + self.values.len() - i) % self.values.len(); // Move backwards with wrap around
            match &self.values[idx] {
                Some((existing_id, value)) if *existing_id == id => {
                    return Some(value);
                }
                None => return None, // Not found
                _ => {}
            }
        }

        // Then probe forwards starting from 1
        for i in 1..=RIGHT_PROBS {
            let idx = (idx + i) % self.values.len();
            match &self.values[idx] {
                Some((existing_id, value)) if *existing_id == id => {
                    return Some(value);
                }
                None => return None, // Not found
                _ => {}
            }
        }

        None // Not found, probing limit reached
    }

    pub fn del(&mut self, id_raw: u32) -> bool {
        let id = NonZeroU32::new(id_raw).expect("WRONG ID PASSED");
        let idx = self.hash(id);

        // Quick immediate check for the initial index
        match &self.values[idx] {
            Some((existing_id, _)) if *existing_id == id => {
                self.values[idx] = None; // Mark as deleted
                return true;
            }
            None => return false, // Not found
            _ => {}
        }

        // First probe backwards
        for i in 0..LEFT_PROBS {
            let idx = (idx + self.values.len() - i) % self.values.len(); // Move backwards with wrap around
            match &self.values[idx] {
                Some((existing_id, _)) if *existing_id == id => {
                    self.values[idx] = None; // Mark as deleted
                    return true;
                }
                None => return false, // Not found
                _ => {}
            }
        }

        // Then probe forwards starting from 1
        for i in 1..=RIGHT_PROBS {
            let idx = (idx + i) % self.values.len();
            match &self.values[idx] {
                Some((existing_id, _)) if *existing_id == id => {
                    self.values[idx] = None; // Mark as deleted
                    return true;
                }
                None => return false, // Not found
                _ => {}
            }
        }

        false // Not found, probing limit reached
    }
}

#[test]
fn test_insert_get_delete() {
    let mut registry = Registry::new(100);

    // Insert elements and verify returned IDs
    let id_one = registry.insert("one".to_string());
    let id_two = registry.insert("two".to_string());
    let id_three = registry.insert("three".to_string());
    assert_eq!(id_one, 1);
    assert_eq!(id_two, 2);
    assert_eq!(id_three, 3);

    // Get elements
    assert_eq!(registry.get(id_one), Some(&"one".to_string()));
    assert_eq!(registry.get(id_two), Some(&"two".to_string()));
    assert_eq!(registry.get(id_three), Some(&"three".to_string()));
    assert_eq!(registry.get(4), None);

    // Insert more elements
    for i in 4..=50 {
        let value = format!("value_{}", i);
        let id = registry.insert(value.clone());
        assert_eq!(registry.get(id), Some(&value));
    }

    // Delete elements and verify
    assert!(registry.del(id_two));
    assert_eq!(registry.get(id_two), None);

    // Insert new element after deletion
    let new_id_two = registry.insert("new_two".to_string());
    assert_eq!(registry.get(new_id_two), Some(&"new_two".to_string()));
    assert_ne!(id_two, new_id_two);

    // Random deletes
    for i in (5..=20).step_by(3) {
        assert!(registry.del(i));
        assert_eq!(registry.get(i), None);
    }

    // Reinsert and verify values
    for i in (5..=20).step_by(3) {
        let value = format!("reinserted_value_{}", i);
        let id = registry.insert(value.clone());
        assert_eq!(registry.get(id), Some(&value));
    }
}

#[test]
fn test_resize() {
    let mut registry = Registry::new(100);

    // Insert elements to force resize
    for i in 1..=200 {
        let value = format!("value_{}", i);
        let id = registry.insert(value.clone());
        assert_eq!(registry.get(id), Some(&value));
    }

    // Ensure all elements are present after resize
    for i in 1..=200 {
        let value = format!("value_{}", i);
        assert_eq!(registry.get(i), Some(&value));
    }

    // Random deletes after resizing
    for i in (50..=150).step_by(7) {
        assert!(registry.del(i));
        assert_eq!(registry.get(i), None);
    }

    // Insert new elements after deletions
    for i in (50..=150).step_by(7) {
        let value = format!("new_value_{}", i);
        let id = registry.insert(value.clone());
        assert_eq!(registry.get(id), Some(&value));
    }
}