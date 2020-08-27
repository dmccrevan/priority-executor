mod lib;
use lib::*;
use std::thread;
use std::time;

fn main() {
    futures::executor::block_on(async {
        let handle = spawn(
            async {
                println!("Running task 1...");
                thread::sleep(time::Duration::from_secs(10));
                2
            },
            1,
        );
        let handle2 = spawn(
            async {
                println!("Running task 2...");
                thread::sleep(time::Duration::from_secs(10));
                3
            },
            2,
        );
        let handle3 = spawn(
            async {
                println!("Running task 3...");
                thread::sleep(time::Duration::from_secs(10));
                4
            },
            1,
        );

        let (x, y, z) = futures::join!(handle, handle2, handle3);
        println!("x: {}\ny: {}\nz: {}", x, y, z);
    });
}
