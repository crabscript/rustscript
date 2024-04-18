// Expected: main thread spawns and program exits after main finishes without waiting

let count = 0;

fn infinite_increment() {
   loop {
       count = count + 1;
   }
}
spawn infinite_increment();
yield;

println(count);