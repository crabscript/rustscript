fn f(x) {
    let z = 3;
    fn g(y) {
        return x + y + z;
    }
   
    g
}

let hof = f(2);

hof(4)