use crate::ast::*;
use crate::ir;
use codespan::Span;
use crate::ir::{VarScope,StaticVarScope,LazyVal,GcPointer,ScopeRet};
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
			Value::FuncCall(call) =>LazyVal::FuncCall(call.translate(table)),


			Value::String(x) => {
				let r = table.get_string(x).unwrap();
				let s = r[1..r.len()-1].to_string();

				LazyVal::Terminal(ir::Value::String(
					GcPointer::new(s)
				))
			},
			


			Value::BuildIn(op) => LazyVal::Terminal(
				ir::Value::Func(
				ir::FunctionHandle::FFI(
					get_buildin_function(op)
				)
			)),

			Value::Lambda(f) => LazyVal::MakeFunc(f.translate(table)),
			Value::MatchLambda(f) => LazyVal::MakeMatchFunc(f.translate(table)),

			Value::Match(m) => {
				let (var_in,statment) = m.translate(table);
				let var = Box::new(var_in);
				LazyVal::Match{var,statment}
			},
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



impl Translate<ir::Block> for FuncBlock {
	fn translate(self, table: &StringTable) -> ir::Block {
		let mut ans = Vec::with_capacity(self.body.len()+1);
		for s in self.body.into_iter() {
			ans.push(s.translate(table));
		}
		ans.push(ir::Statment::Return (match self.ret {
			None => ir::GenericRet::Local(LazyVal::Terminal(ir::Value::Nil)),
			Some(r) => match r {
				Ret::Imp(x) => ScopeRet::Local(x.translate(table)),
				Ret::Exp(x) => ScopeRet::Unwind(x.translate(table)),
			},
		}));

		ir::Block::new(ans)
	}
}

impl Translate<ir::MatchStatment> for (Vec<MatchArm>,Span) {
	fn translate(self, table: &StringTable) -> ir::MatchStatment {
		let mut conds = Vec::with_capacity(self.0.len());
		let mut vals = Vec::with_capacity(self.0.len());

		for a in self.0.into_iter() {
			conds.push(a.pattern.translate(table));
			vals.push(a.result.translate(table));
		}

		ir::MatchStatment::new(conds,vals,self.1)
	}
} 

impl Translate<(LazyVal,ir::MatchStatment)> for MatchStatment {
	fn translate(self, table: &StringTable) -> (LazyVal,ir::MatchStatment) {
		let var = self.val.translate(table);
		let statment = (self.arms,self.debug_span).translate(table);
		(var, statment)
	}
}

impl Translate<ir::LazyMatch> for MatchLambda {
	fn translate(self, table: &StringTable) -> ir::LazyMatch {
		todo!()
	}
}

impl Translate<ir::LazyFunc> for Lambda {
	fn translate(self, table: &StringTable) -> ir::LazyFunc {
		todo!()
	}
}

impl Translate<ir::Statment> for Statment {
	fn translate(self, table: &StringTable) -> ir::Statment {
		match self {
			Statment::Assign(id,x) => ir::Statment::Assign(id,x.translate(table)),
			Statment::Call(f) => ir::Statment::Call(f.translate(table)),
			Statment::Match(m) => ir::Statment::Match(m.translate(table)),
		}
	}
}

impl Translate<ir::MatchCond> for MatchPattern {
	fn translate(self, table: &StringTable) -> ir::MatchCond {
		todo!()
	}
}

impl Translate<ir::Block> for MatchOut {
	fn translate(self, table: &StringTable) -> ir::Block {
		todo!()
	}
}