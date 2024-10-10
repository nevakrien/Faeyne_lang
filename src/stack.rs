use std::sync::Weak;
use crate::value::NativeFunction;
use std::sync::Arc;
use crate::value::Value;
use core::ptr;
use std::mem::{MaybeUninit, size_of};

// Aligned to 8 bytes for any generic type.
#[derive(Debug, PartialEq)]
#[repr(align(8))] //both options work we can really do as we wish
// #[repr(transparent)]
pub struct Aligned<T> {
    pub inner: T,
}

impl<T: Sized> Aligned<T> {
    pub fn new(value: T) -> Self {
        assert!(std::mem::align_of::<T>() <= 8, "T must have alignment of 8 bytes or lower");
        Aligned { inner: value }
    }

    pub fn to_inner(self) -> T {
        self.inner
    }

    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut_ref(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T:Clone> Clone for Aligned<T> {

    fn clone(&self) -> Self { Aligned::new(self.inner.clone()) }
} 

impl<T:Copy> Copy for Aligned<T> {}

// Stack that stores bytes using a statically allocated aligned buffer.
struct Stack<const STACK_CAPACITY:usize =1_000> {
    len: usize,
    data: [MaybeUninit<u8>; STACK_CAPACITY], // Static array for aligned memory
}

#[derive(Debug)]
pub struct StackOverflow;
// const STACK_CAPACITY: usize = 1_000; // Fixed stack capacity

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl<const STACK_CAPACITY:usize> Stack<STACK_CAPACITY> {
    pub fn new() -> Self {
        Self { len: 0, data: [MaybeUninit::uninit(); STACK_CAPACITY] }
    }

    /// # Safety
    ///
    /// The destructor may or may not be called depending if this is poped later
    #[inline]
    pub unsafe fn push<T: Sized>(&mut self, aligned: &Aligned<T>) -> Result<(), StackOverflow> {
        let end = self.len + size_of::<Aligned<T>>();

        if end <= STACK_CAPACITY {
            // let bytes = aligned.as_u8_slice();
            let ptr = aligned as *const _ as *const MaybeUninit<u8>;

            // Write the bytes into the stack
            unsafe {
                let data_ptr = self.data.as_mut_ptr().add(self.len);
                ptr::copy_nonoverlapping(ptr, data_ptr, size_of::<Aligned<T>>());
            }

            self.len = end;
            Ok(())
        } else {
            Err(StackOverflow)
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    #[inline]
    pub unsafe fn pop<T>(&mut self) -> Option<Aligned<T>> {
        if self.len >= size_of::<Aligned<T>>() {
            self.len -= size_of::<Aligned<T>>();
            let start = self.len;

            let ptr = self.data.as_ptr().add(start) as *const Aligned<T>;

            Some(ptr.read())
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    #[inline]
    pub unsafe fn peak<T>(&self) -> Option<&Aligned<T>> {
        if self.len >= size_of::<Aligned<T>>() {
            let start = self.len -size_of::<Aligned<T>>();

            let ptr = self.data.as_ptr().add(start) as *const Aligned<T>;

            Some(&*ptr)
        } else {
            None
        }
    }


}

#[test]
fn test_stack() {
    let mut stack: Stack<1_000> = Stack::new();

    // Create an aligned value with i32 (which is 4 bytes)
    let aligned_value = Aligned::new(42i32);

    // Push the value (by reference)
    unsafe { stack.push(&aligned_value).unwrap(); }

    // Pop the value back (unsafe because we assume we know the type)

    let ref_value = unsafe { stack.peak() };
    assert_eq!(ref_value, Some(&aligned_value));
    
    let value: Option<Aligned<i32>> = unsafe { stack.pop() };

    // Compare with the original i32 value inside Aligned.
    assert_eq!(value.as_ref(), Some(&aligned_value));

    if let Some(ref val) = value {
        println!("Popped value: {}", val.inner_ref());
    }

    // Test to_inner method
    if let Some(popped_value) = value {
        let inner_value = popped_value.to_inner();
        assert_eq!(inner_value, 42i32);
        println!("Moved out inner value: {}", inner_value);
    }

    // #[repr(align(16))]
    // #[derive(Debug,PartialEq,Clone)]
    // struct Dumb{
    //     inner: u8
    // }

    //should either panic or be safe
    // unsafe{
    //     let dumb = Dumb{inner:2};
    //     stack.push(&Aligned::new(dumb.clone())).unwrap();
    //     let d = stack.pop().unwrap().to_inner();
    //     assert_eq!(dumb,d);
    // }

    // Create an aligned value with tuple (usize, usize)
    let aligned_value2 = Aligned::new((3usize, 2usize));

    // Push the value (by reference)
    unsafe { stack.push(&aligned_value2).unwrap(); }

    // Pop the value back (unsafe because we assume we know the type)
    let value: Option<Aligned<(usize, usize)>> = unsafe { stack.pop() };

    // Compare with the original tuple value inside Aligned.
    assert_eq!(value.as_ref(), Some(&aligned_value2));

    // Test to_inner method
    if let Some(popped_value) = value {
        let inner_value = popped_value.to_inner();
        assert_eq!(inner_value, (3usize, 2usize));
    }
}

#[repr(transparent)]
pub struct ValueStack<const STACK_CAPACITY:usize =1_000>{
    stack:Stack<STACK_CAPACITY>
}

#[repr(u32)]
#[derive(Debug, PartialEq,Clone,Copy)]
pub enum ValueTag{

    Terminator=16,
    BoolFalse=0,
    BoolTrue=1,
    Nil=2,
    Int=3,
    Float=4,
    Atom(u32)=5,
    String=6,
    Func=7,
    WeakFunc=8,
    
}


impl Default for ValueStack {
    fn default() -> Self {
        Self::new()
    }
}

impl<const STACK_CAPACITY:usize> ValueStack<STACK_CAPACITY> {
    #[inline]
    pub fn new() -> Self {
        ValueStack { stack: Stack::new() }
    }

    pub fn len(&self) -> usize {
        self.stack.len
    }

    pub fn is_empty(&self) -> bool {
        self.stack.len==0
    }

    #[inline]
    pub fn push_value(&mut self, x: Value) -> Result<(), StackOverflow> {
        unsafe {
            match x {
                Value::Nil => self.stack.push(&Aligned::new(ValueTag::Nil)),
                Value::Bool(b) => {
                    let tag = if b { ValueTag::BoolTrue } else { ValueTag::BoolFalse };
                    self.stack.push(&Aligned::new(tag))
                }
                Value::Int(i) => {
                    let aligned_value = Aligned::new(i);
                    self.stack.push(&aligned_value)?;
                    self.stack.push(&Aligned::new(ValueTag::Int))
                }
                Value::Float(f) => {
                    let aligned_value = Aligned::new(f);
                    self.stack.push(&aligned_value)?;
                    self.stack.push(&Aligned::new(ValueTag::Float))
                }
                Value::Atom(id) => self.stack.push(&Aligned::new(ValueTag::Atom(id))),
                Value::String(s) => {
                    let aligned_value = Aligned::new(s);
                    self.stack.push(&aligned_value)?;

                    std::mem::forget(aligned_value); //stack has sucessfully took ownership of the value
                    self.stack.push(&Aligned::new(ValueTag::String))
                }
                Value::Func(f) => {
                    let aligned_value = Aligned::new(f);
                    self.stack.push(&aligned_value)?;
                    std::mem::forget(aligned_value); //stack has sucessfully took ownership of the value
                    self.stack.push(&Aligned::new(ValueTag::Func))
                }
                Value::WeakFunc(wf) => {
                    let aligned_value = Aligned::new(wf);
                    self.stack.push(&aligned_value)?;
                    std::mem::forget(aligned_value); //stack has sucessfully took ownership of the value
                    self.stack.push(&Aligned::new(ValueTag::WeakFunc))
                }
            }
        }
    }

    //what we are inlining here is PURELY checking the tag which when used correctly can remove the type check later
    //its also worth noting that 99% of the time we are poping and matching on the poped value
    //in those cases we SHOULD see an optimization where we never create Value.
    //for things such as bool this is a huge win
    #[inline(always)] 
    pub fn pop_value(&mut self) -> Option<Value> {
        unsafe {
            match self.stack.pop()?.to_inner() {
                ValueTag::Nil => Some(Value::Nil),
                ValueTag::BoolTrue => Some(Value::Bool(true)),
                ValueTag::BoolFalse => Some(Value::Bool(false)),
                ValueTag::Int => Some(Value::Int(self.stack.pop()?.to_inner())),
                ValueTag::Float => Some(Value::Float(self.stack.pop()?.to_inner())),
                ValueTag::Atom(id) => Some(Value::Atom(id)),
                ValueTag::String => Some(Value::String(self.stack.pop()?.to_inner())),
                ValueTag::Func => Some(Value::Func(self.stack.pop()?.to_inner())),
                ValueTag::WeakFunc => Some(Value::WeakFunc(self.stack.pop()?.to_inner())),
                _ => None,
            }
        }
    }

    pub fn peak_tag(&mut self) -> Option<ValueTag>{
        unsafe{ self.stack.peak()?.to_inner()}
    }

    #[inline]
    pub fn push_terminator(&mut self) -> Result<(), StackOverflow> {
        unsafe { self.stack.push(&Aligned::new(ValueTag::Terminator)) }
    }

    //Typed Pops

    #[inline]
    pub fn push_nil(&mut self) -> Result<(), StackOverflow> {
        unsafe { self.stack.push(&Aligned::new(ValueTag::Nil)) }
    }

    #[inline]
    pub fn pop_nil(&mut self) -> Option<()> {
        if self.peak_tag()? == ValueTag::Nil {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(())
        } else {
            None
        }
    }

    #[inline]
    pub fn push_bool(&mut self, b: bool) -> Result<(), StackOverflow> {
        unsafe {
            let tag = if b { ValueTag::BoolTrue } else { ValueTag::BoolFalse };
            self.stack.push(&Aligned::new(tag))
        }
    }

    #[inline]
    pub fn pop_bool(&mut self) -> Option<bool> {
        match self.peak_tag()? {
            ValueTag::BoolTrue => {
                self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
                Some(true)
            }
            ValueTag::BoolFalse => {
                self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
                Some(false)
            }
            _ => None,
        }
    }

    #[inline]
    pub fn push_int(&mut self, i: i64) -> Result<(), StackOverflow> {
        unsafe {
            let aligned_value = Aligned::new(i);
            self.stack.push(&aligned_value)?;
            self.stack.push(&Aligned::new(ValueTag::Int))
        }
    }

    #[inline]
    pub fn pop_int(&mut self) -> Option<i64> {
        if self.peak_tag()? == ValueTag::Int {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(unsafe { self.stack.pop::<i64>()?.to_inner() })
        } else {
            None
        }
    }

    #[inline]
    pub fn push_float(&mut self, f: f64) -> Result<(), StackOverflow> {
        unsafe {
            let aligned_value = Aligned::new(f);
            self.stack.push(&aligned_value)?;
            self.stack.push(&Aligned::new(ValueTag::Float))
        }
    }

    #[inline]
    pub fn pop_float(&mut self) -> Option<f64> {
        if self.peak_tag()? == ValueTag::Float {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(unsafe { self.stack.pop::<f64>()?.to_inner() })
        } else {
            None
        }
    }

    #[inline]
    pub fn push_atom(&mut self, id: u32) -> Result<(), StackOverflow> {
        unsafe { self.stack.push(&Aligned::new(ValueTag::Atom(id))) }
    }

    #[inline]
    pub fn pop_atom(&mut self) -> Option<u32> {
        if let ValueTag::Atom(id) = self.peak_tag()? {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(id)
        } else {
            None
        }
    }

    #[inline]
    pub fn push_string(&mut self, s: Arc<String>) -> Result<(), StackOverflow> {
        unsafe {
            let aligned_value = Aligned::new(s);
            self.stack.push(&aligned_value)?;
            std::mem::forget(aligned_value);
            self.stack.push(&Aligned::new(ValueTag::String))
        }
    }

    #[inline]
    pub fn pop_string(&mut self) -> Option<Arc<String>> {
        if self.peak_tag()? == ValueTag::String {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(unsafe { self.stack.pop::<Arc<String>>()?.to_inner() })
        } else {
            None
        }
    }

    #[inline]
    pub fn push_func(&mut self, f: Arc<NativeFunction>) -> Result<(), StackOverflow> {
        unsafe {
            let aligned_value = Aligned::new(f);
            self.stack.push(&aligned_value)?;
            std::mem::forget(aligned_value);
            self.stack.push(&Aligned::new(ValueTag::Func))
        }
    }

    #[inline]
    pub fn pop_func(&mut self) -> Option<Arc<NativeFunction>> {
        if self.peak_tag()? == ValueTag::Func {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(unsafe { self.stack.pop::<Arc<NativeFunction>>()?.to_inner() })
        } else {
            None
        }
    }

    #[inline]
    pub fn push_weak_func(&mut self, wf: Weak<NativeFunction>) -> Result<(), StackOverflow> {
        unsafe {
            let aligned_value = Aligned::new(wf);
            self.stack.push(&aligned_value)?;
            std::mem::forget(aligned_value);
            self.stack.push(&Aligned::new(ValueTag::WeakFunc))
        }
    }

    #[inline]
    pub fn pop_weak_func(&mut self) -> Option<Weak<NativeFunction>> {
        if self.peak_tag()? == ValueTag::WeakFunc {
            self.stack.len -= std::mem::size_of::<Aligned<ValueTag>>();
            Some(unsafe { self.stack.pop::<Weak<NativeFunction>>()?.to_inner() })
        } else {
            None
        }
    }
}



impl<const STACK_CAPACITY: usize> Drop for ValueStack<STACK_CAPACITY> {
    fn drop(&mut self) {
        //calling destrutors
        while self.stack.len != 0 {
            self.pop_value();
        }
    }
}

#[test]
fn test_weak_pointer_drop() {
    use crate::value::NativeFunction;

    use std::sync::{Arc};

    let mut value_stack = ValueStack::<100>::new();
    let arc_value = Arc::new(NativeFunction {});
    let weak_value = Arc::downgrade(&arc_value);

    
    value_stack.push_value(Value::Func(arc_value)).unwrap();
    

    std::mem::drop(value_stack);

    assert!(weak_value.upgrade().is_none(), "Weak pointer should not be able to upgrade after stack is dropped");
}



#[test]
fn test_stack_operations() {
    use crate::value::NativeFunction;

    use std::sync::{Arc};

    let mut value_stack = Box::new(ValueStack::<1_000>::new());

    // Push Nil, Bool, Int, and Float values
    value_stack.push_value(Value::Nil).unwrap();
    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Int(123)).unwrap();
    value_stack.push_value(Value::Float(6.9)).unwrap();

    // Pop a few values and verify them
    assert_eq!(value_stack.pop_value(), Some(Value::Float(6.9)));
    assert_eq!(value_stack.pop_value(), Some(Value::Int(123)));

    // Push a few more values
    value_stack.push_value(Value::Atom(42)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();

    // Push and pop a terminator
    value_stack.push_terminator().unwrap();
    assert!(value_stack.pop_value().is_none()); // Terminator should be Nil

    // Test with Weak pointer
    let arc_value = Arc::new(NativeFunction {}); 
    let weak_value= Arc::downgrade(&arc_value);

    value_stack.push_value(Value::Func(arc_value)).unwrap();

    drop(value_stack);

    assert!(weak_value.upgrade().is_none(), "Weak pointer should not be able to upgrade after stack is dropped");
    let mut value_stack = Box::new(ValueStack::<1_000>::new());

    // Push Nil, Bool, Int, and Float values
    value_stack.push_value(Value::Nil).unwrap();
    value_stack.push_value(Value::Bool(true)).unwrap();
    value_stack.push_value(Value::Int(123)).unwrap();
    value_stack.push_value(Value::Float(6.9)).unwrap();

    // Pop a few values and verify them
    assert_eq!(value_stack.pop_value(), Some(Value::Float(6.9)));
    assert_eq!(value_stack.pop_value(), Some(Value::Int(123)));

    // Push a few more values
    value_stack.push_value(Value::Atom(42)).unwrap();
    value_stack.push_value(Value::Bool(false)).unwrap();

    // Push and pop a terminator
    value_stack.push_terminator().unwrap();
    assert!(value_stack.pop_value().is_none()); // Terminator should be Nil

    // Test with Weak pointer
    let arc_value = Arc::new(NativeFunction {});
    let weak_value = Arc::downgrade(&arc_value);
    
    value_stack.push_value(Value::Func(arc_value)).unwrap();
    

    drop(value_stack);

    assert!(weak_value.upgrade().is_none(), "Weak pointer should not be able to upgrade after stack is dropped");


    let mut value_stack = Box::new(ValueStack::<1_000>::new());
    let arc_value = Arc::new(NativeFunction {});
    let weak_value = Arc::downgrade(&arc_value);

    
    value_stack.push_value(Value::Func(arc_value)).unwrap();
    

    std::mem::drop(value_stack);

    assert!(weak_value.upgrade().is_none(), "Weak pointer should not be able to upgrade after stack is dropped");
}


#[test]
fn test_typed_stack_operations() {
    const STACK_CAPACITY: usize = 1024;
    let mut stack = ValueStack::<STACK_CAPACITY>::new();

    // Push all values
    stack.push_nil().unwrap();
    stack.push_bool(true).unwrap();
    stack.push_int(42).unwrap();
    stack.push_float(6.9).unwrap();
    stack.push_atom(123).unwrap();
    let s = Arc::new(String::from("Hello"));
    stack.push_string(s.clone()).unwrap();
    let f = Arc::new(NativeFunction{});
    stack.push_func(f.clone()).unwrap();
    let wf = Arc::downgrade(&f);
    stack.push_weak_func(wf.clone()).unwrap();

    // Pop all values in reverse order
    assert!(stack.pop_weak_func().is_some()); // WeakFunc
    assert!(matches!(stack.pop_func(), Some(func) if Arc::ptr_eq(&func, &f))); // Func
    assert_eq!(stack.pop_string(), Some(s)); // String
    assert_eq!(stack.pop_atom(), Some(123)); // Atom
    assert_eq!(stack.pop_float(), Some(6.9)); // Float
    assert_eq!(stack.pop_int(), Some(42)); // Int
    assert_eq!(stack.pop_bool(), Some(true)); // Bool
    assert_eq!(stack.pop_nil(), Some(())); // Nil

    // Stack should now be empty
    assert!(stack.pop_value().is_none());
}