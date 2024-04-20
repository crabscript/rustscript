fn f(x: int) -> fn(int) -> int {
    let z = 3;
    fn g(y: int) -> int {
        return x + y + z;
    }
   
    g
}

// try uncommenting this line to get type error when compiling
// let hof : fn(int) -> bool = f(2); 

let hof : fn(int) -> int = f(2);

// Expected: 9
hof(4)