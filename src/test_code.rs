#![cfg(test)]

use crate::value::Value;
use crate::translate::compile_source_to_code;

use crate::system::system;

#[test]
fn matrix_mul() {
    let source_code = r#"
def make_matrix(a,b,c,d) {
    match fn {
        0 => match fn {
            0=> a,
            1=> b,
        },
        1 => match fn {
            0=> c,
            1=> d,
        },

        :n=>2,
        :m=>2,
    }
}

def matrix_mul(A,B) {
    match A(:n)==B(:m){
        false => {return :err;},
        true => {}
    };

    make_entry = fn(n,m) -> {
        fn(i,agg) -> {
            match i==A(:n){
                true =>{return  agg;},
                false =>{}
            };

            c = A(i)(m)*B(n)(i);
            self(i-1,c+agg)
        }(A(:n),0)
    };

    make_entry(1,1);
    make_entry(0,0);
    
    _make_row = fn(i,m,row) {
        match i==A(:m){
            true =>{return  row;},
            false =>{}
        };

        row = fn(x) {
            match x==i {
                true => make_entry(m,i),
                false=> row(x) 
            }
        };

        self(i+1,m,row)
    };

    empty_row= match fn {_ => :err_matrix};

    make_row = fn(m) -> {
        _make_row(0,m,empty_row)
    };

    fn(x) -> {
        match x {
            :m => A(:m),
            :n => B(:n),
            _ => make_row(x)
        }
    }
}

#catches bugs related to closures fairly well
def main(system) {
    a = 1;
    b = 1;
    c = 1;
    d = 1;

    matrix = make_matrix(a,b,c,d);
    #matrix(1)(1) |> system(:println)();

    matrix = matrix_mul(matrix,matrix) |> system(:println)();
    #matrix = matrix_mul(matrix,matrix);
    #matrix |> system(:println)();

    #x = matrix(1)(1);
    
    #system(:println)( x )
}"#;
    let code = compile_source_to_code(source_code);

   	 code.run("main", vec![Value::StaticFunc(system)]).unwrap();
}