// Expected: output interleaves between 500..599 and 0..99
// e.g 500, 0, 501, 1, ...

fn func() {
    let x = 0;
    loop x < 100 {
        println(x);
        x = x + 1;
        yield;
    }
}

let t = spawn func();

let x = 500;
loop x < 600 {
    println(x);
    x = x + 1;
    yield;
}

join t;