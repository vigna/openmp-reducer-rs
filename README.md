# OpenMP-Style Reducers

[![crates.io](https://img.shields.io/crates/v/openmp-reducer.svg)](https://crates.io/crates/openmp-reducer)
[![docs.rs](https://docs.rs/openmp-reducer/badge.svg)](https://docs.rs/openmp-reducer)
[![rustc](https://img.shields.io/badge/rustc-1.85+-red.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![CI](https://github.com/vigna/openmp-reducer-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/vigna/openmp-reducer-rs/actions)
![license](https://img.shields.io/crates/l/openmp-reducer)
[![downloads](https://img.shields.io/crates/d/openmp-reducer)](https://crates.io/crates/openmp-reducer)

Sometimes, multiple threads need to update a shared value so frequently that the
overhead of an [atomic] or a [`Mutex`] becomes a bottleneck. If, however, only
the cumulative result of the update is important, then it is possible to use a
*reducer*. This (quite natural) idea comes from [OpenMP]: a reducer offers a
local value to each thread, and has a reduction function that merges a local
value into the global value. As long as the result of multiple reductions is
independent from the order of the reductions, the final global value is
deterministic. This pattern works particularly well with methods such as
[Rayon]'s [`for_each_with`], [`map_with`], and so on.

Local updates can be arbitrary: it is the responsibility of the caller that
they are compatible with the reduction function. Moreover, in some cases the
global result is just “mostly” (e.g., floats) independent from the order of the
reductions, and the programmer is responsible for keeping the differences due to
nondeterminism within acceptable limits.

Note that [Cilk] has *reducer hyperobjects*, which are somewhat more general as
they guarantee a deterministic result even if the reduction order is
relevant: they fuel the inner workings of [Rayon]. However, [Rayon] does not
expose them—you access them through a functional (or *pull*) interface (e.g.,
via the [`fold`] method). If you need an imperative (or *push*) interface (e.g.,
direct mutation methods) and your reduction function satisfies the constraints
above you can use a [`Reducer`] instead.

## Acknowledgments

This software has been partially supported by project SERICS (PE00000014) under
the NRRP MUR program funded by the EU - NGEU. Views and opinions expressed are
however those of the authors only and do not necessarily reflect those of the
European Union or the Italian MUR. Neither the European Union nor the Italian
MUR can be held responsible for them.

[`for_each_with`]: <https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.for_each_with>
[`map_with`]: <https://docs.rs/rayon/latest/rayon/iter/trait.ParallelIterator.html#method.map_with>
[`Mutex`]: <https://doc.rust-lang.org/std/sync/struct.Mutex.html>
[`Reducer`]: <https://docs.rs/openmp-reducer/latest/openmp_reducer/struct.Reducer.html>
[atomic]: <https://doc.rust-lang.org/std/sync/atomic/>
[Rayon]: <https://docs.rs/rayon/latest/rayon/>
[Cilk]: <https://www.opencilk.org/>
[`fold`]: <https://docs.rs/rayon/1.10.0/rayon/iter/trait.ParallelIterator.html#method.fold>
[OpenMP]: <https://www.openmp.org/>
