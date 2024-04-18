fn fac_tail(n: int) -> int {
    fn tail(n: int, acc: int) -> int {
        if n == 0 {
            return acc;
        } else {
            return tail(n-1, acc * n);
        }   
    }

    tail(n, 1)
}

let res : int = fac_tail(4);
res // 24