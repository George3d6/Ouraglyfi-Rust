# Ouraglyfi

This is a micro-library used for implementing various lock free patterns in a concurrent environment.
It has a [sister implementation](https://git.cerebralab.com/george/ouraglyfi_cpp) written in C++, so if you prefer that language consider checking it out

## FixedQueue
This is a fixed size, lock free, wait free, thread safe queue that allows for lock free dequeuing and enqueuing on a FIFO basis.
When construction the queue you can decide on the size (determined at runtime), it's in mutli-consumer, multi-producer mode by default.
Woip on understanding if rust's macro system will allow me to enable/disable those features in an easy way
