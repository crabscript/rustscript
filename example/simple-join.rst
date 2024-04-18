fn func() -> int {
    println(100);
    500
}

println(1);
let t = spawn func();
println(2);

join t

// Expected: 1 2 100 500