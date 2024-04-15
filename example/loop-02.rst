let n : int = 10; // Calculate the 10th (0 idx) Fibonacci number = 55
let fib_prev : int = 0;
let fib_current : int = 1;
let fib_next : int = 0; 

let i = 1; // Start from the 1st Fibonacci number

loop i < n {
    fib_next = fib_prev + fib_current;
    fib_prev = fib_current;
    fib_current = fib_next;
    i = i + 1;
}

if n == 0 {
    fib_prev
} else {
    fib_current
}