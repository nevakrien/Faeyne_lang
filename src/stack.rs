use crate::value::Value;
use core::slice;
use core::ptr;
use std::mem::{MaybeUninit, size_of};

// Aligned to 8 bytes for any generic type.
#[derive(Debug, PartialEq)]
#[repr(align(8))]
pub struct Aligned<T> {
    pub inner: T,
}

impl<T: Sized> Aligned<T> {
    // Constructor ensures that T is less than or equal to 8 bytes in size.
    pub fn new(value: T) -> Self {
        Aligned { inner: value }
    }

    // Method that returns an 8-byte slice of the inner value, padded with zeros if necessary.
    pub fn as_u8_slice(&self) -> &[MaybeUninit<u8>] {
        unsafe {
            let ptr = self as *const _ as *const MaybeUninit<u8>;
            slice::from_raw_parts(ptr, size_of::<Self>())
        }
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

// Stack that stores bytes using a statically allocated aligned buffer.
pub struct Stack {
    len: usize,
    data: [MaybeUninit<u8>; STACK_CAPACITY], // Static array for aligned memory
}

#[derive(Debug)]
pub struct StackOverflow;
const STACK_CAPACITY: usize = 1_000_000; // Fixed stack capacity

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
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
            let bytes = aligned.as_u8_slice();

            // Write the bytes into the stack
            unsafe {
                let data_ptr = self.data.as_mut_ptr().add(self.len);
                ptr::copy_nonoverlapping(bytes.as_ptr(), data_ptr, bytes.len());
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
}

#[test]
fn test_stack() {
    let mut stack = Stack::new();

    // Create an aligned value with i32 (which is 4 bytes)
    let aligned_value = Aligned::new(42i32);

    // Push the value (by reference)
    unsafe { stack.push(&aligned_value).unwrap(); }

    // Pop the value back (unsafe because we assume we know the type)
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
pub struct ValueStack{
    stack:Stack
}

#[repr(u32)]
enum ValueTag{
    Terminator=0,
    Nil=1,
    BoolTrue=2,
    BoolFalse=3,
    Int,
    Float,
    String,
    Atom(u32),
    Func,
    WeakFunc,
    
}


impl ValueStack {
    #[inline]
    pub fn new() -> Self {
        ValueStack { stack: Stack::new() }
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
                    let result = self.stack.push(&aligned_value);
                    std::mem::forget(aligned_value);
                    result.and_then(|_| self.stack.push(&Aligned::new(ValueTag::Int)))
                }
                Value::Float(f) => {
                    let aligned_value = Aligned::new(f);
                    let result = self.stack.push(&aligned_value);
                    std::mem::forget(aligned_value);
                    result.and_then(|_| self.stack.push(&Aligned::new(ValueTag::Float)))
                }
                Value::Atom(id) => self.stack.push(&Aligned::new(ValueTag::Atom(id))),
                Value::String(s) => {
                    let aligned_value = Aligned::new(s);
                    let result = self.stack.push(&aligned_value);
                    std::mem::forget(aligned_value);
                    result.and_then(|_| self.stack.push(&Aligned::new(ValueTag::String)))
                }
                Value::Func(f) => {
                    let aligned_value = Aligned::new(f);
                    let result = self.stack.push(&aligned_value);
                    std::mem::forget(aligned_value);
                    result.and_then(|_| self.stack.push(&Aligned::new(ValueTag::Func)))
                }
                Value::WeakFunc(wf) => {
                    let aligned_value = Aligned::new(wf);
                    let result = self.stack.push(&aligned_value);
                    std::mem::forget(aligned_value);
                    result.and_then(|_| self.stack.push(&Aligned::new(ValueTag::WeakFunc)))
                }
            }
        }
    }

    #[inline]
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

    #[inline]
    pub fn push_terminator(&mut self) -> Result<(), StackOverflow> {
        unsafe { self.stack.push(&Aligned::new(ValueTag::Terminator)) }
    }
}

impl Drop for ValueStack {
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

    let mut value_stack = ValueStack::new();
    let arc_value = Arc::new(NativeFunction {});
    let weak_value = Arc::downgrade(&arc_value);

    
    value_stack.push_value(Value::Func(arc_value)).unwrap();
    

    std::mem::drop(value_stack);

    assert!(weak_value.upgrade().is_none(), "Weak pointer should not be able to upgrade after stack is dropped");
}