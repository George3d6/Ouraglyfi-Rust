use std::sync::atomic::{AtomicUsize, Ordering};

use std::vec::Vec;
use std::cell::UnsafeCell;

use std::mem::replace;
use std::mem::zeroed;

struct ScspFixedLockFreeQueueInternal<T> {
    size    : usize,
    buffer  : Vec<T>,
    r_pos   : AtomicUsize,
    w_pos   : AtomicUsize,
}

impl<T> Default for ScspFixedLockFreeQueueInternal<T> {
    fn default() -> ScspFixedLockFreeQueueInternal<T> {
        ScspFixedLockFreeQueueInternal {
            size  : 100,
            buffer: Vec::new(),
            r_pos :  AtomicUsize::new(0),
            w_pos :  AtomicUsize::new(0),
        }
    }
}

pub struct ScspFixedLockFreeQueue<T> {
    internal_queue: UnsafeCell<ScspFixedLockFreeQueueInternal<T>>,
}

impl<T> ScspFixedLockFreeQueue<T> {

    pub fn new(size: usize) -> ScspFixedLockFreeQueue<T> {
        let mut new_queue : ScspFixedLockFreeQueueInternal<T> = ScspFixedLockFreeQueueInternal{size : size, ..Default::default()};
        new_queue.buffer.reserve(new_queue.size);
        unsafe {
            for i in 0..size {
                new_queue.buffer.push(zeroed());
            }
        }
        return ScspFixedLockFreeQueue{internal_queue : UnsafeCell::new(new_queue)};
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

    pub fn dequeue(&self) -> Option<T> {
        let mut data = self.get_self();
        let old_w_pos = data.w_pos.load(Ordering::Relaxed);
        if data.r_pos.load(Ordering::Relaxed) == old_w_pos {
            return None;
        }
        unsafe {
            let read = replace(&mut data.buffer[data.r_pos.load(Ordering::Relaxed)], zeroed());
            //No longer need to be in the unsafe block here but *shrug*
            data.r_pos.store(self.next_pos(data.r_pos.load(Ordering::Relaxed)), Ordering::Relaxed);
            return Some(read);
        }
    }

    pub fn enqueue(&self, write: T) -> bool {
        let mut data = self.get_self();
        let w_next = self.next_pos(data.w_pos.load(Ordering::Relaxed));
        if data.r_pos.load(Ordering::Relaxed) == w_next {
            return false;
        }
        replace(&mut data.buffer[data.w_pos.load(Ordering::Relaxed)], write);

        data.w_pos.store(w_next, Ordering::Relaxed);
        return true;
    }
}

unsafe impl<T> Sync for ScspFixedLockFreeQueue<T> {}

unsafe impl<T> Send for ScspFixedLockFreeQueue<T> {}
