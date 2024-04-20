fn higher_order(x: int) -> fn(int) -> int {
    fn g(y:int) -> int {
      x + y
    }

    g
}

let add10 = higher_order(10);

let result = add10(20);

println(result); // 30