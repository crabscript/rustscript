fn higher_order(x) {
  return y => x + y;
}

const add10 = higher_order(10);

const result = add10(20);

println(result); // 30