/*
 * SPDX-FileCopyrightText: 2025 Tommaso Fontana
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use openmp_reducer::Reducer;
use std::fmt::Write;

#[test]
fn test_share_and_reduce() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared0 = reducer.share();
        let mut shared1 = shared0.clone();

        *shared0.as_mut() = 5;
        *shared1.as_mut() = 10;
    }

    assert_eq!(reducer.get(), 18);
}

#[test]
fn test_get() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
        assert_eq!(*shared.as_ref(), 5);
    }
    assert_eq!(reducer.get(), 8);
}

#[test]
fn test_two_types() {
    let reducer = Reducer::<i64, i32>::new(3, |global, local| *global += *local as i64);
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
        assert_eq!(*shared.as_ref(), 5);
    }
    assert_eq!(reducer.get(), 8);
}

#[test]
fn test_peek_count() {
    let reducer = Reducer::new(3, |global, local| *global += *local);
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
        assert_eq!(reducer.peek(), 3);
        assert_eq!(shared.peek(), 3);
    }
    assert_eq!(reducer.peek(), 8);
}

#[test]
fn test_concurrent() {
    let reducer = Reducer::<usize>::new(0, |global, local| *global += *local);
    std::thread::scope(|s| {
        for _ in 0..10 {
            let mut shared = reducer.share();
            s.spawn(move || {
                for _ in 0..1000 {
                    *shared.as_mut() += 1;
                }
            });
        }
    });
    assert_eq!(reducer.get(), 10_000);
}

#[test]
fn test_poisoned() {
    let reducer = Reducer::<usize>::new(3, |global, local| {
        if *local == 999 {
            panic!("poisoning the mutex");
        }
        *global += *local;
    });
    // A panic in the reduction function during drop poisons the mutex.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut shared = reducer.share();
        *shared.as_mut() = 999;
    }));
    assert!(result.is_err());
    // Drop recovers from poisoning and still reduces.
    {
        let mut shared = reducer.share();
        *shared.as_mut() = 5;
    }
    // peek and get recover from poisoning as well.
    assert_eq!(reducer.peek(), 8);
    assert_eq!(reducer.get(), 8);
}

#[test]
fn test_debug() {
    let reducer = Reducer::<i32>::new(42, |global, local| *global += *local);
    let shared = reducer.share();
    let mut buf = String::new();
    write!(&mut buf, "{:?}", reducer).unwrap();
    assert!(buf.contains("Reducer"));
    assert!(buf.contains("42"));
    buf.clear();
    write!(&mut buf, "{:?}", shared).unwrap();
    assert!(buf.contains("SharedReducer"));
    assert!(buf.contains("local"));
}
