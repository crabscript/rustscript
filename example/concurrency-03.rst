let count = 0;

fn increment(times: int) {
  let i = 0;
  while i < times {
    count = count + 1;
    i = i + 1;
  }
}

let tid_1 = spawn increment(l00);
let tid_2 = spawn increment(100);
let tid_3 = spawn increment(100);

join tid_3;
join tid_2;
join tid_1;

count