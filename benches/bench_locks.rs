use criterion::{criterion_group, criterion_main, Criterion};
use seqlock_rs::SeqLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

trait MutexTest<T> {
    fn new(v: T) -> Self;
    fn m_lock<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut T) -> R;
}

impl MutexTest<f64> for SeqLock<f64> {
    fn new(v: f64) -> Self {
        SeqLock::new(v)
    }

    fn m_lock<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut f64) -> R,
    {
        let mut guard = self.lock();
        let result = f(&mut guard);
        drop(guard);
        result
    }
}

impl MutexTest<f64> for Mutex<f64> {
    fn new(v: f64) -> Self {
        Mutex::new(v)
    }

    fn m_lock<F, R>(&self, f: F) -> R
        where
            F: FnOnce(&mut f64) -> R,
    {
        let mut guard = self.lock().unwrap();
        let result = f(&mut guard);
        drop(guard);
        result
    }
}

/// Bench lock
///
/// Returns the number of iterations each thread completed
fn bench_lock<M: MutexTest<f64> + Send + Sync + 'static>(
    lock: M,
    num_threads: usize,
    running_time: u64, // in seconds
    work_per_critical_section: usize,
    work_between_critical_sections: usize,
) -> Vec<usize> {
    let lock = Arc::new(lock);
    let barrier = Arc::new(Barrier::new(num_threads));
    let keep_going = Arc::new(AtomicBool::new(true));

    let mut threads = vec![];
    for _ in 0..num_threads {
        let lock = lock.clone();
        let barrier = barrier.clone();
        let keep_going = keep_going.clone();

        threads.push(thread::spawn(move || {
            let mut local_value = 0.0;
            let mut value = 1.0;
            let mut iterations = 0usize;
            barrier.wait();

            while keep_going.load(Relaxed) {
                lock.m_lock(|shared_value| {
                    // let mut shared_value = lock.lock();
                    for _ in 0..work_per_critical_section {
                        *shared_value += value;
                        *shared_value *= 1.01;
                        value = *shared_value;
                    }
                });

                for _ in 0..work_between_critical_sections {
                    local_value += value;
                    local_value *= 1.01;
                    value = local_value;
                }
                iterations += 1;
                // debug!("here: {}", iterations);
            }

            // debug!("done 2");
            (iterations, value)
        }));
    }

    // each test sleep 5second
    thread::sleep(std::time::Duration::from_secs(running_time));
    keep_going.store(false, Relaxed);

    // debug!("done");
    threads.into_iter().map(|x| x.join().unwrap().0).collect()
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("locks");
    group.sample_size(10);

    let threads = 16;
    let running_time = 1;
    let work_per_critical_section = 1000;
    let work_between_critical_sections = 10;

    let mut seq_lock_iterations = 0;
    group.bench_function("seq_lock", |b| {
        b.iter(|| {
            let iterations = bench_lock(
                SeqLock::new(0.0),
                threads,
                running_time,
                work_per_critical_section,
                work_between_critical_sections,
            );
            seq_lock_iterations += iterations.iter().sum::<usize>();
        })
    });

    let mut mutex_iterations = 0;
    group.bench_function("mutex_lock", |b| {
        b.iter(|| {
            let iterations = bench_lock(
                Mutex::new(0.0),
                threads,
                running_time,
                work_per_critical_section,
                work_between_critical_sections,
            );
            mutex_iterations += iterations.iter().sum::<usize>();
        })
    });

    println!("seq_lock iterations on avg: {}", seq_lock_iterations / 10);
    println!("mutex_lock iterations on avg: {}", seq_lock_iterations / 10);

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
