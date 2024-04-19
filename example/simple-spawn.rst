fn func() {
    let j = 0;
    loop j < 100 {
        println("in func");
        j = j + 1;
    }
}



let tid = spawn func();

let i = 0;
loop i < 100 {
    println("in main");
    i = i + 1;
}

// Uncomment to ensure func finishes before main
//join tid;
