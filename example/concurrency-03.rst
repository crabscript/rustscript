let count = 0;

fn increment(times: int) {
  let i = 0;
  loop i < times {
    count = count + 1;
    i = i + 1;
  }
}

let tid_1 = spawn increment(1000);
let tid_2 = spawn increment(1000);
let tid_3 = spawn increment(1000);

join tid_3;
join tid_2;
join tid_1;

count