# Faeyne_lang
an attempt at making a functional languge heavily inspired by elixir but with monads


Faeyne stands for: 
"Functions are everything you need EXCEPTION"


The core idea is we only need piecewise functions for achiving basically everything.
Type Exceptions are also present to make things more argonomics
IO monad (and others) are implemented as follows

```
def main(system) {
	input = system(:input)
	x = pure_func(input)
	io_func(x,system(:printer))
}
```

this allows u to pass a diffrent function. for instance if you want to supress printing for "io_func" you can do


```io_func(x,fn(str) -> {})```


since we dont have arrays pattern matches would have to do. you can use lammda functions as a place holder and there is special syntax for it


```
arr = match fn {0=>a,1=>b,2=>c};
```


In the future we will add pattern matching to allow you to know what patterns a "match fn" or "def" function will accept as inputs. This could potentially let you check for the length of an array by checking the pattern match
You can also pass the length explicitly with an atom like so


```
arr = match fn {0=>a,:len => 1};
```

and then update like so 

```
def append(arr,x){
	new_len = arr(:len)+1;
	fn (k) -> {
		match k {
			:len => new_len,
			new_len => x,
			_ => arr(k)
		} 
	}
}
```

in the future with new pattern matching this could look like so

```
def append(arr,x){
	new_len = arr(:len)+1;
	match fn {
		:len => new_len,
		new_len => x,
		(k) | k<len => arr(k)
	} 
}

```

I am hoping that the JIT can compile this as a modifications in some cases. This is one of the reasons we will opt into Reffrence Counting for our GC. Because the languge is pure and strict making a refrence cycle is impossible.

NOTE: if you make an extension to :system (idk who will but saying anyway)
Then if that extension allows mutating functuions. It is responsible for its own GC which can be very tricky.

if at any point you want to crash doing something like
```
x = 1 + "error message";
```
will work and should give line information.

# dev notes

right now we are assuming that all functions can be defined with some sort of closure including static scope ones. 
this is wrong and we will need to rework the entire scope structure to acomedate recursion and zero copy global scope. 

because we want to allow modules global scope should be passed to each kid when making a closure this operation would take O(num scopes) which is fine since thats the worse case for every lookup anyway. 

It could be benifical in the future to fix scoping to colapse long scope chains when they are created so that lookup is more effishent.