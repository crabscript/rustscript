fn func() -> int {
    println("inside func");
    500
}

println("before spawn func");
let t = spawn func();
println("after spawn func");

join t

// Expected: 
// before spawn func
// after spawn func
// inside func
// 500