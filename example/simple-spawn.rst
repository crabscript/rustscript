fn func() {
    let j = 500;
    loop j < 600 {
        println(j);
        j = j + 1;
    }
}



let tid = spawn func();

let i = 0;
loop i < 100 {
    println(i);
    i = i + 1;
}

// Uncomment to ensure func finishes before main
//join tid;
