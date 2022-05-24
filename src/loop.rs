// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{prelude::*, AsyncIterator};

pub trait Stateful<S, T>: Unpin {
    fn state(&mut self) -> &mut S;

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>>;
}

#[derive(Debug)]
pub struct Never<'a, S>(&'a mut S);

impl<S, T> Stateful<S, T> for Never<'_, S> {
    fn state(&mut self) -> &mut S {
        self.0
    }

    fn poll(&mut self, _cx: &mut Context<'_>) -> Poll<Poll<T>> {
        Pending
    }
}

/// Composable asynchronous event loop.
///
/// # Selecting on Futures:
/// Select first completed future.
///
/// ```rust
#[doc = include_str!("../examples/slices.rs")]
/// ```
///
/// # Task spawning
/// Spawns tasks in a [`Vec`], and removes them as they complete.
///
/// ```rust
#[doc = include_str!("../examples/tasks.rs")]
/// ```
///
#[derive(Debug)]
pub struct Loop<S: Unpin, T, F: Stateful<S, T>> {
    other: F,
    _phantom: core::marker::PhantomData<(S, T)>,
}

impl<'a, S: Unpin, T> Loop<S, T, Never<'a, S>> {
    /// Create an empty event loop.
    pub fn new(state: &'a mut S) -> Self {
        let other = Never(state);
        let _phantom = core::marker::PhantomData;

        Loop { other, _phantom }
    }
}

impl<S: Unpin, T, F: Stateful<S, T>> Loop<S, T, F> {
    /// Register an event handler.
    pub fn on<I>(
        self,
        past: I,
        then: fn(&mut S, Option<I::Item>) -> Poll<T>,
    ) -> Loop<S, T, impl Stateful<S, T>>
    where
        I: AsyncIterator + Unpin,
    {
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Join { other, past, then };

        Loop { other, _phantom }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Loop<S, T, F> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let this = self.get_mut();
        while let Ready(output) = Pin::new(&mut this.other).poll(cx) {
            if let Ready(output) = output {
                return Ready(output);
            }
        }

        Pending
    }
}

struct Join<S, T, O, F: Stateful<S, T>, I: AsyncIterator<Item = O>> {
    other: F,
    past: I,
    then: fn(&mut S, Option<O>) -> Poll<T>,
}

impl<S, T, O, F, I> Stateful<S, T> for Join<S, T, O, F, I>
where
    F: Stateful<S, T>,
    I: AsyncIterator<Item = O> + Unpin,
{
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Poll<T>> {
        if let Ready(output) = Pin::new(&mut self.past)
            .poll_next(cx)
            .map(|output| (self.then)(self.other.state(), output))
        {
            Ready(output)
        } else {
            self.other.poll(cx)
        }
    }
}