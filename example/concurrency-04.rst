// Expected: count = 3000 on each run

let count = 0;
let sem = sem_create();

fn increment(times: int) {
  let i = 0;
  loop i < times {
    wait sem;
    count = count + 1; // critical section
    post sem;
    i = i + 1;
  }
}

println("Spawning 3 threads");

let tid_1 = spawn increment(1000);
let tid_2 = spawn increment(1000);
let tid_3 = spawn increment(1000);

println("Joining 3 threads");

join tid_3;
join tid_2;
join tid_1;

count