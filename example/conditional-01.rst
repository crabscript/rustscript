let y = true;
let x : int = if y { 5; 2 } else { 3 };

if y {
    200;
    x = x + 40;
}

if !y {
    200
} else {
    x
}
