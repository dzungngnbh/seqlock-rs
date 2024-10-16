# SeqLock - A fast and low latency sequence lock implementation in Rust

# Use case: 
The main use case of this library is in environment where you want to have a fast and low latency lock mechanism. 
Sequence lock favors the writer over reader and is a good choice when you have a high number of readers and low number of writers.
The drawback is that if there is too much write activity or the reader is too slow, they might livelock (and the readers may starve).

# Example
```rust 
// src/seq_lock.rs test_sanity
use seqlock_rs::SeqLock; 
let seq_lock = SeqLock::new(123);

for _ in 0..100 {
    let mut guard = seq_lock.lock();
    *guard += 1;
}

let value = seq_lock.read();
assert_eq!(value, 123 + 100);  
 ```

# TODO
[] Update benchmark for the case of few writers and many readers.