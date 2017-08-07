use std::sync::atomic::AtomicUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::Ordering::SeqCst;

use std::vec::Vec;
use std::cell::UnsafeCell;

use std::mem::replace;
use std::mem::swap;
use std::mem::zeroed;

#[derive(PartialEq)]
pub enum ReturnCode {
    Done  = 0,
    Full  = 1,
    Empty = 2,
    Busy  = 3,
}

struct ScspFixedLockFreeQueueInternal<T> {
    size    : usize,
    buffer  : Vec<T>,
    read_position   : AtomicUsize,
    write_position   : AtomicUsize,
    reading : AtomicBool,
    writing : AtomicBool,
}

impl<T> Default for ScspFixedLockFreeQueueInternal<T> {
    fn default() -> ScspFixedLockFreeQueueInternal<T> {
        ScspFixedLockFreeQueueInternal {
            size  : 100,
            buffer: Vec::new(),
            read_position :  AtomicUsize::new(0),
            write_position :  AtomicUsize::new(0),
            reading : AtomicBool::new(false),
            writing : AtomicBool::new(false),
        }
    }
}

pub struct FixedQueue<T> {
    internal_queue: UnsafeCell<ScspFixedLockFreeQueueInternal<T>>,
}

impl<T> FixedQueue<T> {

    pub fn new(size: usize) -> FixedQueue<T> {
        let mut new_queue : ScspFixedLockFreeQueueInternal<T> = ScspFixedLockFreeQueueInternal{size : size, ..Default::default()};
        new_queue.buffer.reserve(new_queue.size);
        unsafe {
            for _ in 0..size {
                new_queue.buffer.push(zeroed());
            }
        }
        return FixedQueue{internal_queue : UnsafeCell::new(new_queue)};
    }

    pub fn dequeue(&self, mut read: &mut T) -> ReturnCode {
        let mut data = self.get_self();
        let can_read = match data.reading.compare_exchange(false, true, SeqCst, SeqCst) {
            Ok(_) => false,
            Err(_) => true,
        };
        if can_read {
            return ReturnCode::Busy;
        }
        let old_write_position = data.write_position.load(Relaxed);
        if data.read_position.load(Relaxed) == old_write_position {
            data.reading.store(false, Relaxed);
            return ReturnCode::Empty;
        }
        swap(&mut data.buffer[data.read_position.load(Relaxed)], &mut read);
        data.read_position.store(self.next_pos(data.read_position.load(Relaxed)), Relaxed);
        data.reading.store(false, Relaxed);
        return ReturnCode::Done;
    }

    pub fn enqueue(&self, write: T) -> ReturnCode {
        let mut data = self.get_self();
        let can_write = match data.writing.compare_exchange(false, true, SeqCst, SeqCst) {
            Ok(_) => false,
            Err(_) => true,
        };
        if can_write {
            return ReturnCode::Busy;
        }
        let w_next = self.next_pos(data.write_position.load(Relaxed));
        if data.read_position.load(Relaxed) == w_next {
            data.writing.store(false, Relaxed);
            return ReturnCode::Full;
        }
        replace(&mut data.buffer[data.write_position.load(Relaxed)], write);

        data.write_position.store(w_next, Relaxed);
        data.writing.store(false, Relaxed);
        return ReturnCode::Done;
    }

    fn next_pos(&self, pos: usize) -> usize {
        let data = self.get_self();
        if pos == (data.size - 1) {
            return 0;
        }
        return pos + 1;
    }

    fn get_self(&self) -> &mut ScspFixedLockFreeQueueInternal<T> {
        unsafe { return &mut *self.internal_queue.get();}
    }

}
unsafe impl<T> Sync for FixedQueue<T> {}


#[cfg(test)]
mod tests {
    use super::*;

    use std::string::String;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn smoke() {
        let nr_elements: i64 = 1000 * 1600;
        let nr_reader_threads: i64 = 20;
        let nr_writer_threads: i64 = 10;

        let queue_sp = Arc::new(FixedQueue::new(500));

        let mut reader_threads = vec![];
        let mut writer_threads = vec![];

        for i in 0i64..nr_writer_threads {
            let queue = queue_sp.clone();
            let t = thread::spawn(move || {
                let mut total: i64 = 0;
                for n in 0..nr_elements/nr_writer_threads {
                    total += n;
                    while ReturnCode::Done != queue.enqueue(n.to_string()) {}
                }
                return total;
            });
            writer_threads.push(t);
        }

        for _ in 0i64..nr_reader_threads {
            let queue = queue_sp.clone();
            let t = thread::spawn(move || {
                let mut total: i64 = 0;
                for _ in 0..nr_elements/nr_reader_threads {
                    let mut read = String::new();
                    while ReturnCode::Done != queue.dequeue(&mut read) {}
                    let nr = read.parse::<i64>().unwrap();
                    assert!(nr <= nr_elements);
                    total += nr;
                }
                return total;
            });
            reader_threads.push(t);
        }

        let mut total_expected: i64 = 0;
        for t in writer_threads {
            total_expected += t.join().unwrap();
        }

        let mut total_got: i64 = 0;
        for t in reader_threads {
            total_got += t.join().unwrap();
        }

        println!("Expected total values to sum to: {}, when dequeued they summed up to: {}", total_got, total_expected);
        assert!(total_expected == total_got);
    }
}
