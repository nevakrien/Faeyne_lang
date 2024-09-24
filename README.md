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
the lifetime of the global scope is giving me trouble.
after a lot of fighting with it I got it to free everything while being ALMOST fully safe.

This required a 4 hours refactor to add a lifetime followed by some fairly weird code to get lifetime anotations where they should be.
It is likely I am missing a more elegent way to do it but as of now I have made it where the global scope is leaked and needs to be manualy turned into a box.

I managed to drop it down into 2 static lifetimes in system that need to be fixed. they corespond to 2 closures that are being created.
The issue is that the lifetime of the closure and the lifetime of the returned value are not directly linked...
what we need is 1 struct containing all of the context that runs things using a &'ctx self

we also potentially want to impl Fn EXPLICITLY which would mean that we need to have a similar trait thats used.
that trait could be potentially very benifical as it can be used for debuging as well


