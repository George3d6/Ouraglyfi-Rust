# Ouraglyfi

This is a micro-library used for implementing various lock free patterns in a concurrent environment.
It has a [sister implementation](https://git.cerebralab.com/george/ouraglyfi_cpp) written in C++, so if you prefer that language consider checking it out

## FixedQueue
This is a fixed size, lock free, wait free, thread safe queue that allows for lock free dequeuing and enqueuing on a FIFO basis.
When construction the queue you can decide on the size (determined at runtime), it's in mutli-consumer, multi-producer mode by default.
Woip on understanding if rust's macro system will allow me to enable/disable those features in an easy way


### Usage:

```
use ouraglyfi::FixedQueue;

let queue_sp = Arc::new(FixedQueue::new(500));

let queue_clone = queue_sp.clone();
thread::spawn(move || {
    match queue_clone.enqueue(String::from("My value")) {
        ouraglyfi::ReturnCode::Done => //Value was successfully added to the queue
        ouraglyfi::ReturnCode::Full  => //The queue is full, can't enqueue, if this is problematic try increasing the size at inception
        ouraglyfi::ReturnCode::Busy => //Someone else is writing, if this is not desired only read from a single thread
        //If you don't mind waiting to read spin with a while, individuals reads 'should' be quick
    }
});

let queue_clone = queue_sp.clone();
thread::spawn(move || {
    let read = String::new();
    match queue_clone.dequeue(read) {
        ouraglyfi::ReturnCode::Done => //read value contains the value read from the queue
        ouraglyfi::ReturnCode::Empty  => //The queue is empty
        ouraglyfi::ReturnCode::Busy => //Someone else is reading, if this is not desired only read from a single thread
        //If you don't mind waiting to read spin with a while, individuals reads 'should' be quick
    }
});
```
