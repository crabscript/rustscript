let count = 0;
let x = 0;

loop x < 10 || x == 10 {
    let j = 0;
    
    
    loop {
        if !(j < x) {
                break;
        }
		    
        count = count + 1;
        j = j + 1;
        
        if j == 5 {
            break;
        }
    }
    
    x = x + 1;
    
    if x == 7 {
        break;
    }
}

count // expected: 20