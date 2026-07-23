/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

#![doc = include_str!("../README.md")]

use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::sync::Mutex;

/// An OpenMP-style reducer that wraps a global value into a [`Mutex`],
/// providing [shareable, cloneable copies with a local
/// value](Reducer::share); the copies will be reduced into the global value
/// when dropped.
///
/// The global value can be observed with [`peek`](Reducer::peek) if the base
/// type is [`Clone`], whereas [`get`](Reducer::get) consumes self and returns
/// the global value.
///
/// For convenience, the global value and the local value have distinct type
/// parameters `G` and `L`, respectively; the second type defaults to the first
/// one. The reduction function has type parameter `F`, defaulting to
/// `fn(&mut G, &L)`.
///
/// Each shared copy has a reference to the [`Reducer`] it was created from, so
/// you cannot call [`get`](Reducer::get) if there are still shared copies
/// around. For example, this code will not compile:
/// ```compile_fail
/// use openmp_reducer::Reducer;
///
/// let reducer = Reducer::<usize>::new(3, |global, local| *global += *local);
/// let mut shared = reducer.share();
/// // drop(shared); // uncommenting this line would make the code compile
/// assert_eq!(reducer.get(), 3);
///```
///
/// # Thread Safety
///
/// [`Reducer`] is [`Send`] and [`Sync`] when `G: Send` and `F: Send + Sync`.
/// [`SharedReducer`] is [`Send`] when additionally `L: Send`.
///
/// # Examples
///
/// In this example, we manually spawn threads:
///
/// ```rust
/// use openmp_reducer::Reducer;
/// use std::thread;
///
/// let reducer = Reducer::<usize>::new(5, |global, local| *global += *local);
/// std::thread::scope(|s| {
///     for i in 0..3 {
///         let mut shared = reducer.share();
///         s.spawn(move || {
///             *shared.as_mut() += 10;
///         });
///     }
/// });
///
/// // Initial value plus additional values from shared copies
/// assert_eq!(reducer.get(), 35);
/// ```
///
/// You can obtain the same behavior with [Rayon](https://docs.rs/rayon) using
/// methods such as
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// and
/// [`map_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.map_with):
///
/// ```rust
/// use openmp_reducer::Reducer;
/// use rayon::prelude::*;
///
/// let reducer = Reducer::<usize>::new(5, |global, local| *global += *local);
/// (0..1000000).into_par_iter().
///     with_min_len(1000). // optional, might reduce the amount of cloning
///     for_each_with(reducer.share(), |shared, i| {
///         *shared.as_mut() += 1;
///     }
/// );
///
/// // Initial value plus additional values from clones
/// assert_eq!(reducer.get(), 1_000_005);
/// ```
///
/// Note that you have to pass `reducer.share()`, which can be cloned. Also,
/// since
/// [`for_each_with`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with)
/// might perform excessive cloning if jobs are too short, you can use
/// [`with_min_len`](https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.with_min_len)
/// to reduce the amount of cloning.
pub struct Reducer<G, L = G, F = fn(&mut G, &L)> {
    global: Mutex<G>,
    reduce: F,
    _marker: PhantomData<fn(L)>,
}

impl<G: Debug, L, F> Debug for Reducer<G, L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reducer")
            .field("global", &self.global)
            .finish_non_exhaustive()
    }
}

impl<G, L, F: Fn(&mut G, &L)> Reducer<G, L, F> {
    /// Creates a new reducer with a given reduction function.
    ///
    /// The function must reduce the local value (second argument) into the
    /// global value (first argument). For the result to be deterministic, the
    /// global value must be the same regardless of the order in which the local
    /// values are reduced.
    pub const fn new(init: G, reduce: F) -> Self {
        Reducer {
            global: Mutex::new(init),
            reduce,
            _marker: PhantomData,
        }
    }
}

impl<G, L, F> Reducer<G, L, F> {
    /// Consumes self and returns the global value.
    ///
    /// Note that you cannot call this method if there are still [shared
    /// copies](Reducer::share) that have not been dropped.
    ///
    /// If you just need to access the global value without consuming self, and
    /// the base type is [`Clone`], use [`peek`](Reducer::peek).
    ///
    /// If the mutex is poisoned (e.g., due to a panic in the reduction
    /// function), the global value is recovered and returned anyway, mirroring
    /// the behavior of the [`Drop`] implementation of [`SharedReducer`].
    pub fn get(self) -> G {
        self.global.into_inner().unwrap_or_else(|e| e.into_inner())
    }
}

impl<G, L: Default, F: Fn(&mut G, &L)> Reducer<G, L, F> {
    /// Returns a [`SharedReducer`] referencing this [`Reducer`].
    ///
    /// The [`SharedReducer`] will be initialized with the default value of the
    /// local type.
    pub fn share(&self) -> SharedReducer<'_, G, L, F> {
        SharedReducer {
            openmp_reducer: self,
            local: L::default(),
        }
    }
}

impl<G: Clone, L, F> Reducer<G, L, F> {
    /// Returns the current global value.
    ///
    /// Note that this method does not guarantee that all shared copies have
    /// been dropped. If you need that guarantee, use [`get`](Reducer::get).
    ///
    /// If the mutex is poisoned (e.g., due to a panic in the reduction
    /// function), the global value is recovered and returned anyway, mirroring
    /// the behavior of the [`Drop`] implementation of [`SharedReducer`].
    pub fn peek(&self) -> G {
        self.global
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

/// A shareable copy of a [`Reducer`] containing a local value and implementing
/// [`Clone`].
///
/// The local value can be accessed with [`AsRef`] and [`AsMut`]
/// implementations.
///
/// When a [`SharedReducer`] is dropped, the local value will be reduced into
/// the global value. If the mutex is poisoned (e.g., due to a panic in another
/// thread), the reduction will still be performed on the recovered value.
pub struct SharedReducer<'a, G, L = G, F: Fn(&mut G, &L) = fn(&mut G, &L)> {
    openmp_reducer: &'a Reducer<G, L, F>,
    local: L,
}

impl<G: Debug, L: Debug, F: Fn(&mut G, &L)> Debug for SharedReducer<'_, G, L, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedReducer")
            .field("openmp_reducer", &self.openmp_reducer)
            .field("local", &self.local)
            .finish()
    }
}

impl<G, L: Default, F: Fn(&mut G, &L)> Clone for SharedReducer<'_, G, L, F> {
    /// Returns a copy sharing the same global value and
    /// with local value initialized to the default value.
    fn clone(&self) -> Self {
        SharedReducer {
            openmp_reducer: self.openmp_reducer,
            local: L::default(),
        }
    }
}

impl<G: Clone, L, F: Fn(&mut G, &L)> SharedReducer<'_, G, L, F> {
    /// Returns the current global value.
    ///
    /// This method delegates to [`Reducer::peek`].
    pub fn peek(&self) -> G {
        self.openmp_reducer.peek()
    }
}

impl<G, L, F: Fn(&mut G, &L)> Drop for SharedReducer<'_, G, L, F> {
    /// Reduces the local value into the global value.
    fn drop(&mut self) {
        let mut lock = self
            .openmp_reducer
            .global
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        (self.openmp_reducer.reduce)(&mut *lock, &self.local);
    }
}

impl<G, L, F: Fn(&mut G, &L)> AsRef<L> for SharedReducer<'_, G, L, F> {
    /// Returns a reference to the local value.
    fn as_ref(&self) -> &L {
        &self.local
    }
}

impl<G, L, F: Fn(&mut G, &L)> AsMut<L> for SharedReducer<'_, G, L, F> {
    /// Returns a mutable reference to the local value.
    fn as_mut(&mut self) -> &mut L {
        &mut self.local
    }
}
