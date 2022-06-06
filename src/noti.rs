// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::prelude::*;

/// Trait for asynchronous event notification.
///
/// Similar to [`AsyncIterator`](core::async_iter::AsyncIterator), but infinite
/// and takes `&mut Self` instead of `Pin<&mut Self>`.
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub trait Notifier {
    /// The event produced by this notifier
    type Event;

    /// Get the next event from this notifier, registering a wakeup when not
    /// ready.
    ///
    /// # Return Value
    ///  - `Poll::Pending` - Not ready yet
    ///  - `Poll::Ready(val)` - Ready with next value
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<Self::Event>;

    /// Get the next [`Self::Event`]
    ///
    /// # Usage
    /// ```rust
    /// use pasts::{Notifier, prelude::*};
    ///
    /// struct MyAsyncIter;
    ///
    /// impl Notifier for MyAsyncIter {
    ///     type Event = Option<u32>;
    ///
    ///     fn poll_next(&mut self, _cx: &mut TaskCx<'_>) -> Poll<Self::Event> {
    ///         Ready(Some(1))
    ///     }
    /// }
    ///
    /// async fn run() {
    ///     let mut count = 0;
    ///     let mut async_iter = MyAsyncIter;
    ///     let mut iterations = 0;
    ///     while let Some(i) = async_iter.next().await {
    ///         count += i;
    ///         iterations += 1;
    ///         if iterations == 3 {
    ///             break;
    ///         }
    ///     }
    ///     assert_eq!(count, 3);
    /// }
    ///
    /// pasts::Executor::default().spawn(Box::pin(run()));
    /// ```
    #[inline]
    fn next(&mut self) -> EventFuture<'_, Self>
    where
        Self: Sized,
    {
        EventFuture(self)
    }

    /// Transform produced [`Self::Event`]s with a function.
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        let noti = self;

        Map { noti, f }
    }
}

impl<T: core::ops::DerefMut<Target = N>, N: Notifier + ?Sized> Notifier for T {
    type Event = N::Event;

    #[inline]
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<N::Event> {
        (**self).poll_next(cx)
    }
}

impl<N: Notifier> Notifier for [N] {
    type Event = (usize, N::Event);

    #[inline]
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<Self::Event> {
        for (i, this) in self.iter_mut().enumerate() {
            if let Ready(value) = this.poll_next(cx) {
                return Ready((i, value));
            }
        }

        Pending
    }
}

#[derive(Debug)]
pub struct EventFuture<'a, N: Notifier>(&'a mut N);

impl<N: Notifier> Future for EventFuture<'_, N> {
    type Output = N::Event;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<Self::Output> {
        self.get_mut().0.poll_next(cx)
    }
}

/// A [`Notifier`] created from a function returning [`Poll`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Noti<T, F: FnMut(&mut TaskCx<'_>) -> Poll<T>>(F);

impl<T, F: FnMut(&mut TaskCx<'_>) -> Poll<T>> Noti<T, F> {
    /// Create a new [`Notifier`] from a function returning [`Poll`].
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<T, F: FnMut(&mut TaskCx<'_>) -> Poll<T>> Notifier for Noti<T, F> {
    type Event = T;

    #[inline]
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<T> {
        self.0(cx)
    }
}

/// A fused [`Future`].
///
/// A fused future is guaranteed to return [`Pending`] after the first
/// [`Ready`].
///
/// Fused [`Future`]s also implement [`Notifier`], sending a single event upon
/// completion.
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Fuse<F: Future + Unpin>(Option<F>);

impl<F: Future + Unpin> From<F> for Fuse<F> {
    fn from(other: F) -> Self {
        Self(other.into())
    }
}

impl<F: Future + Unpin> Future for Fuse<F> {
    type Output = F::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        self.get_mut().poll_next(cx)
    }
}

impl<F: Future + Unpin> Notifier for Fuse<F> {
    type Event = F::Output;

    #[inline]
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        if let Some(ref mut future) = self.0 {
            future.poll(cx)
        } else {
            self.0 = None;
            Pending
        }
    }
}

pub trait Looper<F: Future>: Unpin {
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output>;
    fn set(&mut self, future: F);
}

impl<F: Future> Looper<F> for Pin<Box<F>> {
    #[inline]
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        Pin::new(self).poll(cx)
    }

    #[inline]
    fn set(&mut self, f: F) {
        self.set(f);
    }
}

impl<F: Future + Unpin> Looper<F> for F {
    #[inline]
    fn poll(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        Pin::new(self).poll(cx)
    }

    #[inline]
    fn set(&mut self, f: F) {
        *self = f;
    }
}

/// A [`Notifier`] created from a function returning [`Future`]s.
///
/// A repeating [`Task`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Loop<F: Future, L: FnMut() -> F, S>(S, L);

impl<F: Future + Unpin, L: FnMut() -> F> Loop<F, L, F> {
    /// Create a fused [`Notifier`] from an [`Unpin`] [`Future`]
    pub fn new(mut looper: L) -> Self {
        Self(looper(), looper)
    }
}

impl<F: Future, L: FnMut() -> F> Loop<F, L, Pin<Box<F>>> {
    /// Create a fused [`Notifier`] from a `!Unpin` [`Future`]
    ///
    /// Requires non-ZST allocator.
    pub fn pin(mut looper: L) -> Self {
        Self(Box::pin(looper()), looper)
    }
}

impl<F: Future, L: FnMut() -> F, S: Looper<F>> Notifier for Loop<F, L, S> {
    type Event = F::Output;

    #[inline]
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<F::Output> {
        let poll = Pin::new(&mut self.0).poll(cx);

        if poll.is_ready() {
            self.0.set(self.1());
        }

        poll
    }
}

/// A notifier returned from [`Notifier::map()`].
#[derive(Debug)]
pub struct Map<N, F> {
    noti: N,
    f: F,
}

impl<E, N: Notifier, F: FnMut(N::Event) -> E> Notifier for Map<N, F> {
    type Event = E;

    #[inline]
    fn poll_next(&mut self, cx: &mut TaskCx<'_>) -> Poll<E> {
        self.noti.poll_next(cx).map(&mut self.f)
    }
}
