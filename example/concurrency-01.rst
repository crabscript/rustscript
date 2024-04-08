fn loop_and_print() {
    let x = 0;
    loop {
        if x == 10 {
            break;
        }
        x += 1;
        println(x);
    }
}

println("Creating threads");

let thread_id_1 = spawn loop_and_print;
let thread_id_2 = spawn loop_and_print;
let thread_id_3 = spawn loop_and_print;

join thread_id_3;
join thread_id_2;
join thread_id_1;
