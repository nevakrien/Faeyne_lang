use core::ptr;
use core::mem;
use std::mem::{MaybeUninit, size_of};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;

// Aligned to 8 bytes for any generic type.
#[derive(Copy,Clone, Debug, PartialEq)]
#[repr(align(8))]
pub struct Aligned<T: Sized + Clone> {
    inner: T, // Private field
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
    pub fn as_u8_slice(&self) -> [u8; 8] {
        let size_of_t = size_of::<T>();

        // Create a buffer of 8 bytes initialized to 0 (for padding).
        let mut buffer: [u8; 8] = [0; 8];

        // SAFETY: Convert the reference to the inner value into a raw byte slice.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &self.inner as *const T as *const u8,
                size_of_t,
            )
        };

        // Copy the bytes into the buffer (it will only copy size_of_t bytes, rest remains 0).
        buffer[..size_of_t].copy_from_slice(bytes);

        buffer
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

    #[inline]
    pub fn push<T: Sized + Clone>(&mut self, aligned: &Aligned<T>) -> Result<(), ()> {
        let end = self.len + 8;

        if end <= self.capacity {
            let bytes = aligned.as_u8_slice();

            // Write the bytes into the stack
            unsafe {
                for (i, byte) in bytes.iter().enumerate() {
                    self.data.as_ptr().add(self.len + i).write(MaybeUninit::new(*byte));
                }
            }

            self.len = end;
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn push_grow<T: Sized + Clone>(&mut self, aligned: &Aligned<T>) {
        loop {
            match self.push(aligned) {
                Ok(_) => break,
                Err(_) => self.ensure_capacity(8),
            }
        }
    }

    // SAFETY: The caller must ensure that the data being popped is correctly aligned and matches the expected type.
    #[inline]
    pub unsafe fn pop<T: Sized + Clone>(&mut self) -> Option<Aligned<T>> {
        if self.len >= 8 {
            self.len -= 8;
            let start = self.len;

            let mut data: [MaybeUninit<u8>; 8] = [MaybeUninit::uninit(); 8];

            for i in 0..8 {
                data[i] = self.data.as_ptr().add(start + i).read();
            }

            let bytes = mem::transmute::<_, [u8; 8]>(data);

            // SAFETY: Transmute the 8 bytes back into the correct type T.
            let value: T = mem::transmute_copy::<[u8; 8], T>(&bytes);

            Some(Aligned::new(value))
        } else {
            None
        }
    }

    // SAFETY: The caller must ensure that the alignment of the pushed data is correct.
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

    // SAFETY: The caller must ensure that the alignment and size are correct when reading the data.
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
    stack.push(&aligned_value).unwrap();

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
        stack.push_grow(&aligned_value);
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