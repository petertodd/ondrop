//! Do something on drop.

use core::mem::ManuallyDrop;
use core::ptr;

/// Calls the wrapped closure when dropped.
///
/// That's it.
///
/// # Examples
///
/// Test that `Box` does in fact drop its contents:
///
/// ```
/// # use ondrop::OnDrop;
/// let mut drops = 0;
/// let boxed = Box::new(OnDrop::new(|| {
///     drops += 1;
/// }));
///
/// drop(boxed);
///
/// assert_eq!(drops, 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OnDrop<F: FnOnce()>(ManuallyDrop<F>);

impl<F: FnOnce()> OnDrop<F> {
    /// Creates a new `OnDrop` from a closure.
    pub fn new(f: F) -> Self {
        Self(ManuallyDrop::new(f))
    }

    /// Unwraps the closure without calling it.
    ///
    /// # Examples
    /// ```
    /// # use ondrop::OnDrop;
    /// let dropper = OnDrop::new(|| panic!());
    /// dropper.into_inner(); // no panic
    /// ```
    pub fn into_inner(self) -> F {
        let this = ManuallyDrop::new(self);
        unsafe { ptr::read(&*this.0) }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let f: F = core::ptr::read(&*self.0);
            f()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::Cell;
    use dropcheck::{DropCheck, DropToken};

    #[test]
    /// Make sure the closure is deallocated once and only once.
    fn drops_closure_once() {
        let check = DropCheck::new();
        let (token, state) = check.pair();

        let mut dst = Cell::new(None);
        let ondrop = OnDrop::new(|| {
            dst.set(Some(token));
        });

        assert!(state.is_not_dropped());
        ondrop.into_inner();
        assert!(state.is_dropped());
        assert!(dst.take().is_none());

        let (token, state) = check.pair();
        let ondrop = OnDrop::new(|| {
            dst.set(Some(token));
        });

        assert!(state.is_not_dropped());
        drop(ondrop);
        assert!(state.is_not_dropped());
        assert!(dst.take().is_some());
    }
}
