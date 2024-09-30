use core::mem;
use std::mem::{MaybeUninit, size_of};

// Aligned data to 16 bytes.
#[repr(align(16))]
struct AlignedData<const STACK_SIZE: usize> {
    data: [MaybeUninit<u8>; STACK_SIZE], // Uninitialized u8 elements
}

// Aligned to 8 bytes for any generic type.
#[derive(Clone, Debug, PartialEq)]
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

    // Method to return the owned inner value (move out).
    pub fn to_inner(self) -> T {
        self.inner
    }

    // Immutable reference to the inner value.
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }

    // Mutable reference to the inner value.
    pub fn inner_mut_ref(&mut self) -> &mut T {
        &mut self.inner
    }
}

// Stack that stores bytes.
pub struct Stack<const STACK_SIZE: usize> {
    len: usize,
    data: AlignedData<STACK_SIZE>,
}

impl<const STACK_SIZE: usize> Default for Stack<STACK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const STACK_SIZE: usize> Stack<STACK_SIZE> {
    pub fn new() -> Self {
        Self {
            len: 0,
            data: AlignedData {
                data: [const { MaybeUninit::uninit() }; STACK_SIZE],
            },
        }
    }

    // Push method takes a reference to Aligned<T> and converts it into an 8-byte slice.
    pub fn push<T: Sized + Clone>(&mut self, aligned: &Aligned<T>) {
        let end = self.len + 8;

        if end <= STACK_SIZE {
            let bytes = aligned.as_u8_slice();

            // Write the bytes into the stack
            for (i, d) in self.data.data[self.len..end].iter_mut().enumerate() {
                d.write(bytes[i]);
            }

            self.len = end;
        } else {
            panic!("Stack overflow");
        }
    }

    // Pop method, which is unsafe because the caller needs to ensure they are reading the correct type.
    pub unsafe fn pop<T: Sized + Clone>(&mut self) -> Option<Aligned<T>> {
        if self.len >= 8 {
            self.len -= 8;
            let start = self.len;

            let mut data: [MaybeUninit<u8>; 8] = [const { MaybeUninit::uninit() }; 8];

            for (i, d) in data.iter_mut().enumerate() {
                d.write(self.data.data[start + i].assume_init());
            }

            let bytes = mem::transmute::<_, [u8; 8]>(data);

            // SAFETY: Transmute the 8 bytes back into the correct type T.
            let value: T = mem::transmute_copy::<[u8; 8], T>(&bytes);

            Some(Aligned::new(value))
        } else {
            None
        }
    }
}

#[test]
fn test_stack() {
    let mut stack: Stack<100> = Stack::new();

    // Create an aligned value with i32 (which is 4 bytes)
    let aligned_value = Aligned::new(42i32);

    // Push the value (by reference)
    stack.push(&aligned_value);

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
}
