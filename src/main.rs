extern crate tokio_executor;

use tokio;
use tokio_executor::threadpool::{blocking};
use futures::future::poll_fn;
use std::time::Instant;

/// A really slow inefficient function for finding out if a value is prime
fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    let n_sqrt = f64::sqrt(n as f64);
    let n_sqrt = n_sqrt.trunc() as u64;
    (2..=n_sqrt).all(|v| n % v != 0)
}

/// An even more inefficient prime finding algorithm
fn find_nth_prime(n: u64) -> u64 {
    let mut found_primes = 0u64;
    let mut candidate = 1u64;
    while found_primes < n {
        candidate += 1;
        if is_prime(candidate) {
            found_primes += 1;
        }
    }
    candidate
}

async fn prime_output(id: u64, n: u64) {
    // So what's happening here?
    // We want to `spawn` the result of this function. This means it must be a task, which means the
    // future must have `Output=()`.
    // But `poll_fn(blocking(..))` is a `Output=Result<(), BlockingError>` future. The simplest way
    // to convert this to a task is to `await` the result (returning `Result<..>`) and then
    // handling the error. In this demo, we just panic if `blocking` returns an error.
    poll_fn(move |_| {
        blocking(|| {
            let t = Instant::now();
            let val = find_nth_prime(n);
            let t = t.elapsed();
            println!("#{:2}, {:6}th prime = {:12} ({:6.3}s)", id, n, val, t.as_secs_f64());
        })
    }).await.expect("Couldn't block");

}

/// Spawn a search for 20 prime numbers starting with the hardest to find and running down to the
/// easiest to find.
async fn main_fut() {
    let max = 5_000_000u64;
    for i in 0..20 {
        let n = max - 200_000*i;
        tokio::spawn(prime_output(i,n));
    }
}


/// Run a search for 20 prime numbers on 5 "blocking" threads. Since we start with the really hard
/// to find primes, we expect the threads to return in reverse order. But there are only 5 threads
/// looking for primes, so we expect #4 to return first, then #3 and so on. Once #4 is done, it'll
/// get a head start start working on #9 (or possibly any of #5-#9, depending on how the work-
/// stealing/task assignment goes), so the return order from this point is harder to guess up front.
///
/// Here's the output from my PC though:
/// ```text
/// # 4, 4200000th prime =     71480051 (239.881s)
/// # 3, 4400000th prime =     75103493 (250.352s)
/// # 2, 4600000th prime =     78736451 (276.399s)
/// # 1, 4800000th prime =     82376219 (287.379s)
/// # 0, 5000000th prime =     86028121 (312.824s)
/// # 8, 3400000th prime =     57099299 (167.897s)
/// # 6, 3800000th prime =     64268779 (209.286s)
/// # 7, 3600000th prime =     60678089 (183.773s)
/// # 9, 3200000th prime =     53533511 (151.493s)
/// # 5, 4000000th prime =     67867967 (228.976s)
/// #14, 2200000th prime =     35926307 (92.645s)
/// #13, 2400000th prime =     39410867 (98.256s)
/// #12, 2600000th prime =     42920191 (112.429s)
/// #11, 2800000th prime =     46441207 (126.380s)
/// #10, 3000000th prime =     49979687 (147.052s)
/// #17, 1600000th prime =     25582153 (53.941s)
/// #16, 1800000th prime =     29005541 (64.282s)
/// #18, 1400000th prime =     22182343 (46.629s)
/// #15, 2000000th prime =     32452843 (74.454s)
/// #19, 1200000th prime =     18815231 (35.867s)
/// Bye
/// ```
fn main()  {
    let rt = tokio::runtime::Builder::new()
        .blocking_threads(5)
        // Run the work scheduler on one thread so we can really see the effects of using `blocking` above
        .core_threads(1)
        .build()
        .expect("Could not create runtime");
    rt.block_on(main_fut());
    rt.shutdown_on_idle();
    println!("Bye");
}
