//! The single seam for order-preserving parallelism.
//!
//! `map_ordered` maps each item to a result and returns them in input order.
//! Native builds run the map across a Rayon pool; wasm builds run it serially.
//! Because the seam is order-preserving, a caller that partitions work into
//! ordered segments (chapters) gets byte-identical output regardless of thread
//! count or target — only wall-clock changes.
//!
//! Callers throttle native concurrency through Rayon's own pool (a global
//! `ThreadPoolBuilder`, or a scoped `pool.install(..)`); this seam intentionally
//! exposes no thread-count knob of its own. There is deliberately only this one
//! helper — book-level reconciliation is small, order-sensitive, and stays
//! serial, so no `par_reduce`/`fold` seam exists until an actual use needs it.

/// Map `f` over `items`, preserving input order in the result.
#[cfg(not(target_arch = "wasm32"))]
pub fn map_ordered<T, R, F>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    use rayon::prelude::*;
    items.par_iter().map(f).collect()
}

/// Map `f` over `items`, preserving input order in the result.
#[cfg(target_arch = "wasm32")]
pub fn map_ordered<T, R, F>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    items.iter().map(f).collect()
}
