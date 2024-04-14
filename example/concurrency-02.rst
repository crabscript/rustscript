let count = 0;

fn infinite_increment() {
   loop {
       count = count + 1;
   }
}
spawn infinite_increment();

yield;