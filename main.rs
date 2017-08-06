/*
*  A single producer single consumer lock free queue of fixed size (defined at compile time)
*/
mod scsp_queue;
use scsp_queue::ScspFixedLockFreeQueue;

use std::string::String;
use std::thread;
use std::sync::Arc;



//Testing here for now
fn main() {
    //Whilst the fight against the evil compiler rages on this is a temporary solution
    let nr_elements: i64 = 500000;
    let queue_sp: Arc<ScspFixedLockFreeQueue<String>> = Arc::new(ScspFixedLockFreeQueue::new(100));

    let queue = queue_sp.clone();
    let t1 = thread::spawn(move || {
        for n in 0..nr_elements {
            while !queue.enqueue(n.to_string()) {}
        }
    });

    let queue = queue_sp.clone();
    let t2 = thread::spawn(move || {
        for n in 0..nr_elements {
            let mut doit = true;
            while doit {
                let result = queue.dequeue();
                match result {
                    Some(ele) => {
                        doit = false;
                        let nr = ele.parse::<i64>().unwrap();
                        println!("{}", nr);
                        assert!(nr <= nr_elements);
                    }
                    None => {}
                }
            }
        }
    });

    t1.join();
    t2.join();
}
