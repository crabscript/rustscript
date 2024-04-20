// Expected: count != 3000 on each run, though we want 3000

let count = 0;

fn increment(times: int) {
  let i = 0;
  loop i < times {
    let tmp = count;
    yield;
    count = tmp + 1;
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