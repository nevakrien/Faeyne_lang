// Sketch for transitioning a tree walk interpreter to a bytecode interpreter with a focus on Value representation and stack integration

use crate::stack::{Aligned, Stack};

// Enum for value types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    Nil = 0,
    BoolTrue = 1,
    BoolFalse = 2,
    Atom = 3,
    String = 4,
    Int = 5,
    Float = 6,
    Func = 7,
}

// C-style representation for Value with u32 + u32 (data and tag)
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Value {
    tag_data: u64, // This u64 will be split into two u32 fields
}

impl Value {
    // Create a new Value for Nil
    pub fn new_nil() -> Self {
        Value { tag_data: ValueType::Nil as u64 }
    }

    // Create a new Value for Bool
    pub fn new_bool(val: bool) -> Self {
        let tag = if val { ValueType::BoolTrue as u64 } else { ValueType::BoolFalse as u64 };
        Value { tag_data: tag }
    }

    // Create a new Value for Atom
    pub fn new_atom(id: u32) -> Self {
        let packed_data = (id as u64) << 32 | ValueType::Atom as u64;
        Value { tag_data: packed_data }
    }

    // Create a new Value for String
    pub fn new_string(id: u32) -> Self {
        let packed_data = (id as u64) << 32 | ValueType::String as u64;
        Value { tag_data: packed_data }
    }

    // Create a new Value for Int
    pub fn new_int(val: i32) -> Self {
        let packed_data = (val as u32 as u64) << 32 | ValueType::Int as u64;
        Value { tag_data: packed_data }
    }

    // Create a new Value for Float
    pub fn new_float(val: f32) -> Self {
        let packed_data = (val.to_bits() as u64) << 32 | ValueType::Float as u64;
        Value { tag_data: packed_data }
    }

    // Create a new Value for Func
    pub fn new_func(id: u32) -> Self {
        let packed_data = (id as u64) << 32 | ValueType::Func as u64;
        Value { tag_data: packed_data }
    }

    // Method to interpret Value from the stored tag and data
    pub fn interpret(&self) -> Result<InterpretedValue, &'static str> {
        let tag = (self.tag_data & 0xFFFFFFFF) as u32;
        let data = (self.tag_data >> 32) as u32;
        match tag {
            0 => Ok(InterpretedValue::Nil),
            1 => Ok(InterpretedValue::Bool(true)),
            2 => Ok(InterpretedValue::Bool(false)),
            3 => Ok(InterpretedValue::Atom(data)),
            4 => Ok(InterpretedValue::String(data)),
            5 => Ok(InterpretedValue::Int(data as i32)),
            6 => Ok(InterpretedValue::Float(f32::from_bits(data))),
            7 => Ok(InterpretedValue::Func(data)),
            _ => Err("Invalid tag value"),
        }
    }
}

// Enum to represent interpreted values
#[derive(Debug, Clone, PartialEq)]
pub enum InterpretedValue {
    Nil,
    Bool(bool),
    Atom(u32),
    String(u32),
    Int(i32),
    Float(f32),
    Func(u32),
}

// Trait for specialized Stack operations for Value
pub trait ValueStack {
    fn push_value(&mut self, value: &Value);
    fn pop_value(&mut self) -> Result<InterpretedValue, &'static str>;
}

impl<const STACK_SIZE: usize> ValueStack for Stack<STACK_SIZE> {
    fn push_value(&mut self, value: &Value) {
        let aligned = Aligned::new(*value);
        self.push(&aligned);
    }

    fn pop_value(&mut self) -> Result<InterpretedValue, &'static str> {
        unsafe {
            match self.pop::<Value>() {
                Some(aligned) => aligned.to_inner().interpret(),
                None => Err("Failed to pop value from stack"),
            }
        }
    }
}

#[test]
fn test_value_stack() {
    let mut stack: Stack<100> = Stack::new();

    // Push multiple items onto the stack
    let value_nil = Value::new_nil();
    let value_bool = Value::new_bool(true);
    let value_atom = Value::new_atom(123);
    let value_string = Value::new_string(456);
    let value_func = Value::new_func(789);

    stack.push_value(&value_nil);
    stack.push_value(&value_bool);
    stack.push_value(&value_atom);
    stack.push_value(&value_string);
    stack.push_value(&value_func);

    // Pop items one by one and check them
    let popped_value = stack.pop_value().expect("Failed to pop Value::Func");
    assert_eq!(popped_value, InterpretedValue::Func(789));

    let popped_value = stack.pop_value().expect("Failed to pop Value::String");
    assert_eq!(popped_value, InterpretedValue::String(456));

    // Push more items after popping some
    let value_int = Value::new_int(42);
    stack.push_value(&value_int);

    let popped_value = stack.pop_value().expect("Failed to pop Value::Int");
    assert_eq!(popped_value, InterpretedValue::Int(42));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Atom");
    assert_eq!(popped_value, InterpretedValue::Atom(123));

    // Push and pop again to ensure stack consistency
    stack.push_value(&value_bool);
    let popped_value = stack.pop_value().expect("Failed to pop Value::Bool");
    assert_eq!(popped_value, InterpretedValue::Bool(true));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Bool");
    assert_eq!(popped_value, InterpretedValue::Bool(true));

    let popped_value = stack.pop_value().expect("Failed to pop Value::Nil");
    assert_eq!(popped_value, InterpretedValue::Nil);

    // Pushing raw data and then using pop_value would be unsafe due to potential alignment issues
    // Therefore, we avoid doing so in this test to ensure proper safety practices are followed.
}
