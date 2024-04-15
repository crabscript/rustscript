// O(n^2) 1+2+..+n calculation with nested loop

let count = 0;
let x = 0;
let n = 10;

loop x < n || x == n {
    let j = 0;
    
    loop j < x {
		    count = count + 1;
        j = j + 1;
    }
    
    x = x + 1;
}

count // expected: 55