fn loop_and_print(x: int) {
    let count = 0;
    
    loop {
        if count > 10 || count  == 10 {
            break;
        }
        
        println(x);
        
        count = count + 1;
    }
}

let thread_id_1 = spawn loop_and_print(1);
let thread_id_2 = spawn loop_and_print(2);
let thread_id_3 = spawn loop_and_print(3);

join thread_id_3;
join thread_id_2;
join thread_id_1;

println(true);
