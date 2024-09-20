use crate::ast::*;
use crate::ir;
use crate::ir::{VarScope,StaticVarScope,LazyVal};
use crate::basic_ops::get_buildin_function;

pub trait Translate<Output> {
    fn translate(self, table: &StringTable) -> Output;
}

impl Translate<LazyVal> for FValue {
	fn translate(self, table: &StringTable) -> LazyVal {
		let val : Value = self.into();
		val.translate(table)
	}
}

impl Translate<LazyVal> for Value {
	fn translate(self, table: &StringTable) -> LazyVal {
		match self {
			Value::Variable(id) => LazyVal::Ref(id),
			Value::Int(x) => LazyVal::Terminal(ir::Value::Int(x)),
			Value::Float(x) => LazyVal::Terminal(ir::Value::Float(x)),
			Value::Bool(x) => LazyVal::Terminal(ir::Value::Bool(x)),
			Value::Atom(x) => LazyVal::Terminal(ir::Value::Atom(x)),

			Value::Nil => LazyVal::Terminal(ir::Value::Nil),
			Value::BuildIn(op) => LazyVal::Terminal(
				ir::Value::Func(
				ir::FunctionHandle::FFI(
					get_buildin_function(op)
				)
			)),

			Value::FuncCall(call) =>LazyVal::FuncCall(call.translate(table)),
			// Value::Match(m) => LazyVal::Match(
			// 		ir::Match{
			// 		var: Box::new(call.name.translate(table)),
			// 		statment : ir:MatchStatment
			// 	}
			// ),

			_ => todo!()
		}
	}
}

impl Translate<ir::Call> for FunctionCall {
	fn translate(self, table: &StringTable) -> ir::Call {
		ir::Call{
			called:Box::new(self.name.translate(table)),
			args:self.args.into_iter().map(|x| x.translate(table)).collect(),
			debug_span: self.debug_span
		}
	}
}

impl Translate<ir::Statment> for Statment {
	fn translate(self, table: &StringTable) -> ir::Statment {
		match self {
			Statment::Assign(id,x) => ir::Statment::Assign(id,x.translate(table)),
			Statment::Call(f) => ir::Statment::Call(f.translate(table)),
			_=> todo!()
		}
	}
}