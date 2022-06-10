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
/// Similar to [`AsyncIterator`](core::async_iter::AsyncIterator), but infinite.
///
/// It's expected that [`Notifier`]s can be polled indefinitely without causing
/// panics or undefined behavior.  They are not required to continue sending
/// events indefinitely, though.
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
    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<Self::Event>;

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
    ///     fn poll_next(self: Pin<&mut Self>, _: &mut Exec<'_>) -> Poll<Self::Event> {
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
        Self: Sized + Unpin,
    {
        EventFuture(self)
    }

    /// Transform produced [`Self::Event`]s with a function.
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized + Unpin,
    {
        let noti = self;

        Map { noti, f }
    }
}

impl<N: Notifier + ?Sized> Notifier for Box<N> {
    type Event = N::Event;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<N::Event> {
        Pin::new(&mut self.get_mut()).poll_next(e)
    }
}

impl<N: Notifier + ?Sized, P> Notifier for Pin<P>
where
    P: core::ops::DerefMut<Target = N> + Unpin,
{
    type Event = N::Event;

    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<Self::Event> {
        Pin::get_mut(self).as_mut().poll_next(e)
    }
}

impl<N: Notifier + Unpin + ?Sized> Notifier for &mut N {
    type Event = N::Event;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<N::Event> {
        Pin::new(&mut **self).poll_next(e)
    }
}

impl<N: Notifier + Unpin> Notifier for [N] {
    type Event = (usize, N::Event);

    #[inline]
    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<Self::Event> {
        for (i, this) in self.get_mut().iter_mut().enumerate() {
            if let Ready(value) = Pin::new(this).poll_next(e) {
                return Ready((i, value));
            }
        }

        Pending
    }
}

#[derive(Debug)]
pub struct EventFuture<'a, N: Notifier + Unpin>(&'a mut N);

impl<N: Notifier + Unpin> Future for EventFuture<'_, N> {
    type Output = N::Event;

    #[inline]
    fn poll(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.get_mut().0).poll_next(e)
    }
}

/// A [`Notifier`] created from a function returning [`Poll`].
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
#[derive(Debug)]
pub struct Noti<T, F: FnMut(&mut Exec<'_>) -> Poll<T> + Unpin>(F);

impl<T, F: FnMut(&mut Exec<'_>) -> Poll<T> + Unpin> Noti<T, F> {
    /// Create a new [`Notifier`] from a function returning [`Poll`].
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<T, F: FnMut(&mut Exec<'_>) -> Poll<T> + Unpin> Notifier for Noti<T, F> {
    type Event = T;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<T> {
        self.get_mut().0(e)
    }
}

/// Trait for "fusing" a [`Future`] (conversion to a [`Notifier`]).
///
/// <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/styles/a11y-dark.min.css">
/// <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.5.1/highlight.min.js"></script>
/// <script>hljs.highlightAll();</script>
/// <style> code.hljs { background-color: #000B; } </style>
pub trait Fuse: Sized {
    /// Fuse the [`Future`]
    fn fuse(self) -> Option<Self>;
}

impl<F: Future> Fuse for F {
    fn fuse(self) -> Option<Self> {
        self.into()
    }
}

impl<F: Future> Notifier for Option<F> {
    type Event = F::Output;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<F::Output> {
        let mut this = self;
        if let Some(f) = this.as_mut().as_pin_mut() {
            let output = f.poll(e);
            if output.is_ready() {
                this.set(None);
            }
            output
        } else {
            Pending
        }
    }
}

pub trait Rep<F: Future>: Unpin {
    fn poll(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<F::Output>;
    fn set(self: Pin<&mut Self>, future: F);
}

impl<F: Future> Rep<F> for Pin<Box<F>> {
    #[inline]
    fn poll(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<F::Output> {
        Future::poll(self, e)
    }

    #[inline]
    fn set(self: Pin<&mut Self>, f: F) {
        Pin::set(self.get_mut(), f);
    }
}

impl<F: Future + Unpin> Rep<F> for F {
    #[inline]
    fn poll(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<F::Output> {
        Future::poll(self, e)
    }

    #[inline]
    fn set(mut self: Pin<&mut Self>, f: F) {
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

impl<F: Future, L: FnMut() -> F + Unpin, S: Rep<F>> Notifier for Loop<F, L, S> {
    type Event = F::Output;

    #[inline]
    fn poll_next(self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<F::Output> {
        let this = self.get_mut();
        let poll = Pin::new(&mut this.0).poll(e);

        if poll.is_ready() {
            Pin::new(&mut this.0).set(this.1());
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

impl<N: Notifier + Unpin, F, E> Notifier for Map<N, F>
where
    F: FnMut(N::Event) -> E + Unpin,
{
    type Event = E;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, e: &mut Exec<'_>) -> Poll<E> {
        Pin::new(&mut self.noti).poll_next(e).map(&mut self.f)
    }
}
