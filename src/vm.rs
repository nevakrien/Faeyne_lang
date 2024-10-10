// use crate::value::VarTable;
// use crate::stack::StackRet;
// use crate::stack::StackView;
// use crate::basic_ops::handle_bin;
// use crate::basic_ops::BinOp;
// use crate::value::StringRegistry;
// use crate::value::Registry;
// use crate::reporting::Error;
// use crate::reporting::ErrList;
// use crate::stack::Stack;
// // use crate::value::Scope;
// use ast::ast::StringTable;

// use arrayvec::ArrayVec;

// #[derive(Clone)]
// pub enum Function<'code> {
//     Native(StackView<'code>),
//     FFI,
// }
// pub type FunctionRegistry<'code>=Registry<Function<'code>>;

// pub struct RetData<'code> {
//     ret: StackRet,
//     code:StackView<'code>,
//     // scope:Scope<'ctx>
// }

// pub const MAX_LOCAL_SCOPES: usize = 1000;
// pub const MAX_RECURSION :usize=2_500;

// // #[repr(C)] //want to orgenize by importance
// pub struct Context<'ctx,'code> {
//     // pub pos:usize,
//     code: StackView<'code>,//we need a varible length stack for these...
//     call_stack:  &'ctx mut ArrayVec<RetData<'code>,MAX_RECURSION>,
//     local_call_stack: &'ctx mut ArrayVec<RetData<'code>,MAX_LOCAL_SCOPES>,
//     vars:&'ctx mut VarTable,

//     pub scope: Scope<'ctx>,

//     pub stack: &'ctx mut Stack,
    
//     //pub constants: &'code[IRValue],
//     pub funcs: &'ctx FunctionRegistry<'code>,
//     pub strings: &'ctx StringRegistry,
    
//     pub table: &'ctx StringTable<'code>,//for errors only
// }

// impl<'ctx,'code> Context<'ctx,'code> {
//     /// # Safety
//     ///
//     /// code must never tell us to pop the wrong type from stack.
//     /// as long as code allways pops any non value type on stack that it pushed
//     /// the code is safe
//     pub unsafe fn new(

//         table: &'ctx StringTable<'code>,
//         code: StackView<'code>,//constants: &'code[IRValue],
        
//         stack: &'ctx mut Stack,scope: Scope<'ctx>,
//         call_stack:  &'ctx mut ArrayVec<RetData<'ctx,'code>,MAX_RECURSION>,
//         local_call_stack: &'ctx mut ArrayVec<RetData<'ctx,'code>,MAX_LOCAL_SCOPES>,
        
//         strings: &'ctx StringRegistry,funcs: &'ctx FunctionRegistry<'code>
//     ) -> Self{
        
//         Context{
//             table,code,stack,call_stack,local_call_stack,scope,strings,funcs
//         }
//     }

//     pub fn get_code(&self) -> &StackView<'code>{
//         &self.code
//     }


//     fn pop_to(&mut self,id:u32) -> Result<(),ErrList>{
//         match self.stack.pop_value(){
//             Ok(x) => self.scope.set(id as usize,x)
//                 .map_err(|_| Error::Bug("tried seting a non existent id").to_list()),
            
//             Err(..)  => Err(Error::Bug("over poping").to_list()),
//         }
//     }

//     fn push_from(&mut self,id:u32) -> Result<(),ErrList>{
//         let value = self.scope.get(id as usize)
//             .ok_or_else(|| Error::Bug("tried seting a non existent id").to_list())?;
//         self.stack.push_value(&value).map_err(|_|{Error::StackOverflow.to_list()})?;
//         Ok(())
//     }

//     fn push_constant(&mut self) -> Result<(),ErrList>{
//         let res = unsafe{self.code.pop()};
//         let val = res.ok_or_else(|| Error::Bug("over pop").to_list())?;
//         self.stack.push_value(&val.to_inner()).map_err(|_|{Error::StackOverflow.to_list()})
//     }

//     pub fn curent_var_names(&self) -> Vec<&'code str> {
//         self.scope.table.names.iter()
//         .map(|id| self.table.get_raw_str(*id))
//         .collect()
//     } 

//     fn big_ret(&mut self) -> Result<(),ErrList> {
//         let ret_data = self.call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
//         let value = self.stack.pop_value().map_err(|()| Error::Bug("over pop value stack").to_list())?;

//         self.code = ret_data.code;
//         self.stack.return_to(ret_data.ret);
        

//         self.stack.push_value(&value).map_err(|_|{Error::StackOverflow.to_list()})?;
        

//         self.scope = ret_data.scope;
//         self.local_call_stack.clear();

//         todo!()
//     }

//     fn small_ret(&mut self) -> Result<(),ErrList> {
//         let ret_data = self.local_call_stack.pop().ok_or_else(|| Error::Bug("over pop call stack").to_list())?;
//         let value = self.stack.pop_value().map_err(|()| Error::Bug("over pop value stack").to_list())?;

//         self.code = ret_data.code;
//         self.stack.return_to(ret_data.ret);
        

//         self.stack.push_value(&value).map_err(|_|{Error::StackOverflow.to_list()})?;

//         self.scope = ret_data.scope;

//         todo!()
//     }

//     // fn push_scope(&mut self,code:StackView<'code>,ret:StackRet) -> Result<(),ErrList> {
//     //     let mut ret = RetData{scope:self.scope,code,ret};
//     //     self.scope = ret.scope.add_scope(&[]);
//     //     Ok(())
//     // }

    
//     /// # Safety
//     ///
//     /// this is safe as long as we pop the ops 1 by 1 as the code says
//     /// we are relying on the compilation step making correct code 
//     pub unsafe fn handle_op(&mut self,op:Operation) -> Result<(),ErrList> {
//         match op {
//             BinOp(b) => handle_bin(self,b),
//             PopTo(id) => self.pop_to(id),
//             PushFrom(id) => self.push_from(id),
//             PushConst => self.push_constant(),

//             RetSmall => self.small_ret(),
//             RetBig => self.big_ret(),

//             _ => todo!(),
//         }
//     }

//     //returns true if we should keep going
//     pub fn next_op(&mut self) -> Result<bool,ErrList>{
//         unsafe {
//             let r = self.code.pop();
//             let Some(op) = r else {return Ok(false)};
//             self.handle_op(op.to_inner()).map(|_| true)
//         }
        
//     }

// }



// #[derive(Debug,PartialEq,Clone,Copy)]
// #[repr(u32)]
// pub enum Operation {
//     //type ids end at 8 so we take a safe distance from them to maje the try_from fail on most UB

//     Call = 16,//calls a function args are passed through the stack and a single return value is left at the end (args are consumed)
//     RetBig,//returns out of the function scope. 
//     RetSmall,//returns out of a match block

//     PopTo(u32),
//     PushFrom(u32),
//     PushConst,

//     BinOp(BinOp),

//     Match,//pops a value to match aginst then returns the result (not quite sure about the detais)
//     CaptureClosure,//pops the data off the stack and creates a new function returning it as an IRValue to the stack
// }

// use Operation::*;