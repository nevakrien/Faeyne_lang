//code is from https://github.com/rust-lang/stacker/blob/master/src/lib.rs 
//we are vendoring it because they add a deoendency on windows.h being on the path for windows
//this is a big ask because it means you are now depending on microsofts toolkits
//and the dependency is not on methods we use
//psm only reallu depends on assembly (but it does use cc to build it)
//and so for the sake of build simplicity we use it

#[macro_use]
extern crate psm;

use psm::stack_pointer;

thread_local! {
    static STACK_LIMIT: Cell<Option<usize>> = Cell::new(0);
}

pub fn set_accptble_growth(size: usize){
	STACK_LIMIT=stack_pointer()+size;
}

#[inline(always)]
fn get_stack_limit() -> Option<usize> {
    STACK_LIMIT.with(|s| s.get())
}

pub fn remaining_stack() -> Option<usize> {
    let current_ptr = stack_pointer();
    get_stack_limit().map(|limit| current_ptr - limit)
}