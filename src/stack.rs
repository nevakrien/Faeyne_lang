#![allow(clippy::result_unit_err)]

use core::slice;
use core::ptr;
use core::mem;
use std::mem::{MaybeUninit, size_of};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

// Aligned to 8 bytes for any generic type.
#[derive(Copy,Clone, Debug, PartialEq)]
#[repr(align(8))]
pub struct Aligned<T: Sized + Clone> {
    pub inner: T,
}

impl<T: Sized + Clone> Aligned<T> {
    // Constructor ensures that T is less than or equal to 8 bytes in size.
    pub fn new(value: T) -> Self {
        assert!(
            size_of::<T>() <= 8,
            "T must be smaller than or equal to 8 bytes in size!"
        );
        Aligned { inner: value }
    }

    // Method that returns an 8-byte slice of the inner value, padded with zeros if necessary.
    pub fn as_u8_slice(&self) -> &[MaybeUninit<u8>; 8] {
        // let size_of_t = size_of::<T>();

        // // Create a buffer of 8 bytes initialized to 0 (for padding).
        // let mut buffer: [u8; 8] = [0; 8];

        // // SAFETY: Convert the reference to the inner value into a raw byte slice.
        // let bytes = unsafe {
        //     std::slice::from_raw_parts(
        //         &self.inner as *const T as *const u8,
        //         size_of_t,
        //     )
        // };

        // // Copy the bytes into the buffer (it will only copy size_of_t bytes, rest remains 0).
        // buffer[..size_of_t].copy_from_slice(bytes);

        // buffer

        unsafe {
            let ptr=self as *const _ as *const [MaybeUninit<u8>; 8];
            &*ptr
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

// Stack that stores bytes using a manually allocated aligned buffer.
pub struct Stack {
    len: usize,
    capacity: usize, //this being dynamic adds a semi significant overhead. it makes push pop take 2x longer
    data: NonNull<MaybeUninit<u8>>, // Pointer to aligned memory
}

static ALIGN :usize= 16;

impl Stack {
    pub fn with_capacity(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, ALIGN).expect("Invalid layout");
        let data = unsafe { alloc(layout) as *mut MaybeUninit<u8> };
        let data = NonNull::new(data).expect("Failed to allocate memory");
        Self { len: 0, capacity, data }
    }

    #[inline(always)]
    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub fn ensure_capacity(&mut self, additional: usize) {
        let required_capacity = self.len + additional;
        if required_capacity > self.capacity {
            let new_capacity = self.capacity.max(1) * 2;
            let new_capacity = new_capacity.max(required_capacity);
            let layout = Layout::from_size_align(new_capacity, ALIGN).expect("Invalid layout");
            unsafe {
                let new_data = alloc(layout) as *mut MaybeUninit<u8>;
                let new_data = NonNull::new(new_data).expect("Failed to allocate memory");
                ptr::copy_nonoverlapping(self.data.as_ptr(), new_data.as_ptr(), self.len);
                dealloc(self.data.as_ptr() as *mut u8, Layout::from_size_align(self.capacity, ALIGN).expect("Invalid layout"));
                self.data = new_data;
                self.capacity = new_capacity;
            }
        }
    }

    pub fn shrink_to_fit(&mut self) {
        let size = self.len.max(1);
        let layout = Layout::from_size_align(size, ALIGN).expect("Invalid layout");
        unsafe {
            let new_data = alloc(layout) as *mut MaybeUninit<u8>;
            let new_data = NonNull::new(new_data).expect("Failed to allocate memory");
            ptr::copy_nonoverlapping(self.data.as_ptr(), new_data.as_ptr(), self.len);
            dealloc(self.data.as_ptr() as *mut u8, Layout::from_size_align(self.capacity, ALIGN).expect("Invalid layout"));
            self.data = new_data;
            self.capacity = size;
        }
    }

    /// # Safety
    ///
    /// When not using ValueStack this is perfectly safe.
    /// However when using ValueStack pushing a type thats untagged can break the invariance
    #[inline]
    pub unsafe fn push<T: Sized + Clone>(&mut self, aligned: &Aligned<T>) -> Result<(), ()> {
        let end = self.len + 8;

        if end <= self.capacity {
            let bytes = aligned.as_u8_slice();

            // Write the bytes into the stack
            unsafe {
                let data_ptr = self.data.as_ptr().add(self.len); //as *mut u8;
                ptr::copy_nonoverlapping(bytes.as_ptr(), data_ptr, bytes.len());
            }

            self.len = end;
            Ok(())
        } else {
            Err(())
        }
    }

    /// # Safety
    ///
    /// same as push
    #[inline]
    pub unsafe fn push_grow<T: Sized + Clone>(&mut self, aligned: &Aligned<T>) {
        loop {
            match self.push(aligned) {
                Ok(_) => break,
                Err(_) => self.ensure_capacity(8),
            }
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    #[inline]
    pub unsafe fn pop<T: Sized + Clone>(&mut self) -> Option<Aligned<T>> {
        if self.len >= 8 {
            self.len -= 8;
            let start = self.len;

            let ptr = self.data.as_ptr().add(start) as *const Aligned<T>;

            Some((&*ptr).clone())
        } else {
            None
        }
    }


    /// # Safety
    ///
    /// The caller must ensure that the alignment of the pushed data is correct.
    #[inline]
    pub unsafe fn push_raw(&mut self, bytes: &[u8]) -> Result<(), ()> {
        let end = self.len + bytes.len();

        if end <= self.capacity {
            for (i, byte) in bytes.iter().enumerate() {
                self.data.as_ptr().add(self.len + i).write(MaybeUninit::new(*byte));
            }
            self.len = end;
            Ok(())
        } else {
            Err(())
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the alignment of the pushed data is correct.
    #[inline]
    pub unsafe fn push_raw_grow<T: Sized + Clone>(&mut self,  bytes: &[u8]) {
        loop {
            match self.push_raw(bytes) {
                Ok(_) => break,
                Err(_) => self.ensure_capacity(8),
            }
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the alignment and size are correct when reading the data.
    #[inline]
    pub unsafe fn pop_raw(&mut self, size: usize) -> Option<Vec<u8>> {
        if self.len >= size {
            self.len -= size;
            let start = self.len;

            let mut bytes = Vec::with_capacity(size);
            for i in 0..size {
                bytes.push(self.data.as_ptr().add(start + i).read().assume_init());
            }

            Some(bytes)
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// same as push
    #[inline]
    pub unsafe fn push_stack_view(&mut self, stack_view: &StackView) -> Result<(), ()> {
        self.push(&Aligned::new(stack_view.idx))?;
        self.push(&Aligned::new(stack_view.data.len()))?;
        self.push(&Aligned::new(stack_view.data.as_ptr())) //this makes it basically impossible for us to get the wrong tag by mistake

    }

    /// # Safety
    ///
    /// same as push
    #[inline]
    pub unsafe fn push_stack_view_grow(&mut self, stack_view: &StackView) {
        loop {
            match self.push_stack_view(stack_view) {
                Ok(_) => break,
                Err(_) => self.ensure_capacity(mem::size_of::<StackView>()),
            }
        }
    }

    
}

impl Drop for Stack {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, ALIGN).expect("Invalid layout");
        unsafe {
            dealloc(self.data.as_ptr() as *mut u8, layout);
        }
    }
}



#[test]
fn test_stack() {
    let mut stack = Stack::with_capacity(100);

    // Create an aligned value with i32 (which is 4 bytes)
    let aligned_value = Aligned::new(42i32);

    // Push the value (by reference)
    unsafe{stack.push(&aligned_value).unwrap();}

    // Pop the value back (unsafe because we assume we know the type)
    let value: Option<Aligned<i32>> = unsafe { stack.pop() };

    // Compare with the original i32 value inside Aligned.
    assert_eq!(value, Some(aligned_value));

    if let Some(ref val) = value {
        println!("Popped value: {}", val.inner_ref());
    }

    // Test to_inner method
    if let Some(popped_value) = value {
        let inner_value = popped_value.to_inner();
        assert_eq!(inner_value, 42i32);
        println!("Moved out inner value: {}", inner_value);
    }

    // Test unsafe push_raw and pop_raw
    let raw_data: [u8; 4] = [1, 2, 3, 4];
    unsafe {
        stack.push_raw(&raw_data).unwrap();
        let popped_raw = stack.pop_raw(4).expect("Failed to pop raw data");
        assert_eq!(popped_raw, raw_data);
    }

    // Ensure no mixed alignment issues by pushing raw and then not using pop for aligned types
    let raw_data_2: [u8; 8] = [5, 6, 7, 8, 9, 10, 11, 12];
    unsafe {
        stack.push_raw(&raw_data_2).unwrap();
        let popped_raw_2 = stack.pop_raw(8).expect("Failed to pop raw data 2");
        assert_eq!(popped_raw_2, raw_data_2);
    }

    // Test push_grow to force resizing
    for _ in 0..20 {
        unsafe{stack.push_grow(&aligned_value);}
    }

    // Pop the value back (unsafe because we assume we know the type)
    for _ in 0..20 {
        let value: Option<Aligned<i32>> = unsafe { stack.pop() };
        assert_eq!(value, Some(aligned_value));
    }

    // Test shrink_to_fit
    stack.shrink_to_fit();
    assert_eq!(stack.get_capacity(), stack.len.max(1));
}

#[derive(Clone)]
pub struct StackView<'a> {
    idx: isize,
    pub data:&'a [u8]
}

impl<'a> StackView<'a> {
    #[inline]
    pub fn from_stack(s:&'a Stack) -> Self {
        let data = unsafe{ 
            //cant be making a mut ref to the data at any point
            //so we make sure we are working with const

            let r : *const MaybeUninit<u8> =s.data.as_ptr();
            let slice = std::slice::from_raw_parts(r,s.len);
            &*(slice as *const [MaybeUninit<u8>] as *const [u8])
        };
        StackView{data,idx:(data.len()-1) as isize}
    }

    /// # Safety
    ///
    /// the index must be pointing to an aligned value
    /// also note that pop/peak will be called with type assumbtions
    /// so this function shares respobsibility
    pub unsafe fn set_index(&mut self,idx:isize) {
        self.idx=idx;
    }

    pub fn get_index(&self) -> isize {
        self.idx
    }

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    #[inline]
    pub unsafe fn pop<T: Sized + Clone>(&mut self) -> Option<Aligned<T>> {
        if self.idx >= 7 {
            self.idx -= 8;

            let start = (self.idx + 1) as usize;

            let ptr = self.data.as_ptr().add(start) as *const Aligned<T>;

            Some((*ptr).clone())
        } else {
            None
        }
    }


    /// # Safety
    ///
    /// The caller must ensure that the alignment and size are correct when reading the data.
    #[inline]
    pub unsafe fn pop_raw(&mut self, size: usize) -> Option<Vec<u8>> {
        if (self.idx+1) as usize>= size {
            self.idx -= size as isize;
            let start = self.idx as usize;

            let bytes = &self.data[start..start+size];

            Some(bytes.to_vec())
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    #[inline]
    pub unsafe fn peak<T: Sized + Clone>(&mut self) -> Option<Aligned<T>> {
        let ans = self.pop();
        self.idx+=8;
        ans
    }
}

pub trait PopStack{
    /// # Safety
    ///
    /// The caller must ensure that the alignment and size are correct when reading the data.
     unsafe fn pop_raw(&mut self, size: usize) -> Option<Vec<u8>>;

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    unsafe fn pop<T: Sized + Clone>(&mut self) -> Option<Aligned<T>>; 

    /// # Safety
    ///
    /// The caller must ensure that the data being popped matches the expected type.
    #[inline]
    unsafe fn pop_stack_view(&mut self) -> Option<StackView> {
        let ptr :*const u8= self.pop()?.to_inner();        
        let len :usize= self.pop()?.to_inner();
        let idx :isize= self.pop()?.to_inner();

        // #[cfg(miri)]//miri will error here because it cant detect ptr is a valid existing pointer...
        // panic!();

        let data = slice::from_raw_parts(ptr,len);

        Some(StackView{idx,data})
    }
    
}

impl PopStack for Stack {

    unsafe fn pop_raw(&mut self, i: usize) -> Option<Vec<u8>> { self.pop_raw(i) }
    unsafe fn pop<T>(&mut self) -> Option<Aligned<T>> where T: Clone { self.pop() }
}

impl PopStack for StackView<'_> {

    unsafe fn pop_raw(&mut self, i: usize) -> Option<Vec<u8>> { self.pop_raw(i) }
    unsafe fn pop<T>(&mut self) -> Option<Aligned<T>> where T: Clone { self.pop() }
}

#[test]
fn test_stack_view() {
    let mut stack = Stack::with_capacity(200);

    // Create some aligned values
    let aligned_value_1 = Aligned::new(10i32);
    let aligned_value_2 = Aligned::new(20i32);
    let aligned_value_3 = Aligned::new(30u32);

    // Push the aligned values to the stack
    unsafe{
        stack.push(&aligned_value_1).unwrap();
        stack.push(&aligned_value_2).unwrap();
        stack.push(&aligned_value_3).unwrap();
    }
    

    // Create a `StackView` from the `Stack`
    let mut stack_view = StackView::from_stack(&stack);

    // Peek the last value in the stack
    let peak_value: Option<Aligned<u32>> = unsafe { stack_view.peak() };
    assert_eq!(peak_value, Some(aligned_value_3));

    // Pop the values and verify they match what was pushed
    let pop_value_3: Option<Aligned<u32>> = unsafe { stack_view.pop() };
    assert_eq!(pop_value_3, Some(aligned_value_3));

    let pop_value_2: Option<Aligned<i32>> = unsafe { stack_view.pop() };
    assert_eq!(pop_value_2, Some(aligned_value_2));

    let pop_value_1: Option<Aligned<i32>> = unsafe { stack_view.peak() };
    assert_eq!(pop_value_1, Some(aligned_value_1));

    let pop_value_1: Option<Aligned<i32>> = unsafe { stack_view.pop() };
    assert_eq!(pop_value_1, Some(aligned_value_1));

    // Ensure there are no more items to pop
    let pop_value_none: Option<Aligned<i32>> = unsafe { stack_view.pop() };
    assert_eq!(pop_value_none, None);

    // Push some raw data and create multiple `StackView` instances
    let raw_data: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    unsafe {
        stack.push_raw(&raw_data).unwrap();
    }

    let mut stack_view_1 = StackView::from_stack(&stack);
    let mut stack_view_2 = StackView::from_stack(&stack);

    // Both views should be able to pop the same value without modifying the stack
    let pop_value_1: Option<Aligned<[u8; 8]>> = unsafe { stack_view_1.pop() };
    assert_eq!(pop_value_1.map(|aligned| aligned.to_inner()), Some(raw_data));

    let pop_value_2: Option<Aligned<[u8; 8]>> = unsafe { stack_view_2.peak() };
    assert_eq!(pop_value_2.map(|aligned| aligned.to_inner()), Some(raw_data));
}

#[test]
// #[cfg(not(miri))]
fn test_stack_view_push_pop() {
    let mut stack1 = Stack::with_capacity(300);
    let mut stack2 = Stack::with_capacity(300);

    // Create an aligned value
    let aligned_value_1 = Aligned::new(42i32);
    unsafe{stack1.push_grow(&aligned_value_1);}

    {
        // Create a stack view from stack1
        let stack_view = StackView::from_stack(&stack1);

        // Push the stack view onto stack2
        unsafe{stack2.push_stack_view_grow(&stack_view);}
    } // End the borrow of `stack1` by `stack_view` here

    // Pop the stack view back from stack2
    let popped_view = unsafe { stack2.pop_stack_view() };
    assert!(popped_view.is_some());
    // Ensure the data length matches what was originally pushed
    assert_eq!(popped_view.unwrap().data.len(), 8); // Replace the direct data comparison with a length check
}

#[test]
// #[cfg(not(miri))]
fn test_push_pop_valid_pointer() {
    let mut stack = Stack::with_capacity(100);

    // Create some random data on the heap
    let value = Box::new(12345);
    let ptr = Box::into_raw(value);

    unsafe{stack.push_grow(&Aligned::new(ptr));}
    let p :*const usize= unsafe{stack.pop().unwrap().to_inner()};
    assert_eq!(p,ptr);

    unsafe{assert_eq!(12345,*ptr)};
    unsafe{assert_eq!(12345,*p)};

    unsafe{
        let _ = Box::from_raw(ptr);
    }
}
