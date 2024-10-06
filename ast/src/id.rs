use crate::ast::StringTable;


// IMPORTANT: The first value is assumed not to be a variable.
// If this is broken, then Lambda match statements will break.
// We use NIL_ID as the nil type so that we can perform easy nil checks.

// Core IDs
pub const NIL_ID: u32 = 0;
pub const ERR_ID: u32 = 1;
pub const OK_ID: u32 = 2;

// Added :len with ID 3
pub const LEN_ID: u32 = 3;

// Type IDs
pub const BOOL_ID: u32 = 4;
pub const STRING_ID: u32 = 5;
pub const INT_ID: u32 = 6;
pub const FLOAT_ID: u32 = 7;
pub const ATOM_ID: u32 = 8;
pub const FUNC_ID: u32 = 9;
pub const TYPE_ATOM_ID: u32 = 10;

pub const TO_STRING_ID: u32 = 11;

// Special IDs
pub const UNDERSCORE_ID: u32 = 12;
pub const MAIN_ID: u32 = 13;

// Function IDs
pub const PRINTLN_ID: u32 = 14;
pub const READ_FILE_ID: u32 = 15;
pub const WRITE_FILE_ID: u32 = 16;
pub const DELETE_FILE_ID: u32 = 17;

pub const READ_DIR_ID: u32 = 18;
pub const MAKE_DIR_ID: u32 = 19;
pub const DELETE_DIR_ID: u32 = 20;

pub const STRING_OUT_OF_BOUNDS: u32 = 21;
pub const SELF_ID: u32 = 22;


pub fn preload_table(table: &mut StringTable) {
    assert_eq!(table.get_id(":nil"), NIL_ID);
    assert_eq!(table.get_id(":err"), ERR_ID);
    assert_eq!(table.get_id(":ok"), OK_ID);

    assert_eq!(table.get_id(":len"), LEN_ID);  // Added :len

    assert_eq!(table.get_id(":bool"), BOOL_ID);
    assert_eq!(table.get_id(":string"), STRING_ID);
    assert_eq!(table.get_id(":int"), INT_ID);
    assert_eq!(table.get_id(":float"), FLOAT_ID);
    assert_eq!(table.get_id(":atom"), ATOM_ID);
    assert_eq!(table.get_id(":func"), FUNC_ID);
    assert_eq!(table.get_id(":type"), TYPE_ATOM_ID);
    assert_eq!(table.get_id(":to_string"), TO_STRING_ID);

    assert_eq!(table.get_id("_"), UNDERSCORE_ID);
    assert_eq!(table.get_id("main"), MAIN_ID);
    
    assert_eq!(table.get_id(":println"), PRINTLN_ID);
    assert_eq!(table.get_id(":read_file"), READ_FILE_ID);
    assert_eq!(table.get_id(":write_file"), WRITE_FILE_ID);
    assert_eq!(table.get_id(":delete_file"), DELETE_FILE_ID);

    assert_eq!(table.get_id(":read_dir"), READ_DIR_ID);
    assert_eq!(table.get_id(":make_dir"), MAKE_DIR_ID);
    assert_eq!(table.get_id(":delete_dir"), DELETE_DIR_ID);

    assert_eq!(table.get_id(":string_out_of_bounds"), STRING_OUT_OF_BOUNDS);
    assert_eq!(table.get_id("self"), SELF_ID);
}

#[macro_export]
macro_rules! get_id {
    (":nil") => { NIL_ID };
    (":err") => { ERR_ID };
    (":ok") => { OK_ID };

    (":len") => { LEN_ID };  // Added :len

    (":bool") => { BOOL_ID };
    (":string") => { STRING_ID };
    (":int") => { INT_ID };
    (":float") => { FLOAT_ID };
    (":atom") => { ATOM_ID };
    (":func") => { FUNC_ID };
    (":type") => { TYPE_ATOM_ID };

    ("_") => { UNDERSCORE_ID };
    ("main") => { MAIN_ID };

    (":println") => { PRINTLN_ID };
    (":to_string") => { TO_STRING_ID };
    (":read_file") => { READ_FILE_ID };
    (":write_file") => { WRITE_FILE_ID };
    (":delete_file") => { DELETE_FILE_ID };

    (":read_dir") => { READ_DIR_ID };
    (":make_dir") => { MAKE_DIR_ID };
    (":delete_dir") => { DELETE_DIR_ID };
    
    (":string_out_of_bounds") => { STRING_OUT_OF_BOUNDS };
    ("self") => { SELF_ID };

    ($other:expr) => { // Fallback to the runtime version if it's not predefined
        $other
    };
}