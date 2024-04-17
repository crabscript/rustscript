fn fac(n) {
    if n == 0 {
        println(0);
        return 1;
    }
    
    n * fac(n-1)
}


fn fac_tail(n) {
    fn tail(n, acc) {
        if n == 0 {
            return acc;
        } else {
            return tail(n-1, acc * n);
        }   
    }

    tail(n, 1)
}

let f = fac(5);
let t = fac_tail(5);

println(f);
println(t);
println(f == t);