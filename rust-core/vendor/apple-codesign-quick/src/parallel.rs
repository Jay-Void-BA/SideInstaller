//! Keep the native signer parallel; WASI Preview 1 has no worker threads.
#[cfg(not(target_os = "wasi"))]
pub use rayon::{join, prelude};

#[cfg(target_os = "wasi")]
pub fn join<A, B, R, S>(a: A, b: B) -> (R, S)
where
    A: FnOnce() -> R,
    B: FnOnce() -> S,
{
    (a(), b())
}

#[cfg(target_os = "wasi")]
pub mod prelude {
    pub trait IntoParallelIterator: IntoIterator + Sized {
        fn into_par_iter(self) -> Self::IntoIter {
            self.into_iter()
        }
    }
    impl<T: IntoIterator> IntoParallelIterator for T {}

    pub trait ParallelSlice<T> {
        fn par_iter(&self) -> std::slice::Iter<'_, T>;
        fn par_chunks_mut(&mut self, size: usize) -> std::slice::ChunksMut<'_, T>;
    }
    impl<T> ParallelSlice<T> for [T] {
        fn par_iter(&self) -> std::slice::Iter<'_, T> {
            self.iter()
        }
        fn par_chunks_mut(&mut self, size: usize) -> std::slice::ChunksMut<'_, T> {
            self.chunks_mut(size)
        }
    }
}
