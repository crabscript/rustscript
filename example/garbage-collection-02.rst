// Compile untyped: hof type annotation not added yet
fn higher_order(x) {
    fn g(y) {
      x + y
    }

    g
}

let add10 = higher_order(10);

let result = add10(20);

println(result); // 30