#!/bin/bash

set -e

sysroot=$(rustc --print sysroot)
core="$sysroot/lib/rustlib/src/rust/library/core"

test -d $core

if [ -f $core/Cargo.toml.turbo-bak ]; then
    mv $core/Cargo.toml.turbo-bak $core/Cargo.toml
fi
if [ -f $core/src/task/wake.rs.turbo-bak ]; then
    mv $core/src/task/wake.rs.turbo-bak $core/src/task/wake.rs
fi

cp $core/Cargo.toml $core/Cargo.toml.turbo-bak
cp $core/src/task/wake.rs $core/src/task/wake.rs.turbo-bak 

patch $core/Cargo.toml <<"EOF"
*** Cargo.toml.turbo-bak	2023-03-07 00:26:08.627557642 +0100
--- Cargo.toml	2023-03-07 00:26:54.764578210 +0100
***************
*** 33,35 ****
--- 33,36 ----
  # Make `RefCell` store additional debugging information, which is printed out when
  # a borrow error occurs
  debug_refcell = []
+ turbowakers = []
EOF

patch $core/src/task/wake.rs <<"EOF"
diff --git a/wake.rs.turbo-bak b/wake.rs
index 8fc942d..92fcfce 100644
--- a/wake.rs.turbo-bak
+++ b/wake.rs
@@ -368,6 +368,9 @@ impl<'a> ContextBuilder<'a> {
 #[cfg_attr(not(doc), repr(transparent))] // work around https://github.com/rust-lang/rust/issues/66401
 #[stable(feature = "futures_api", since = "1.36.0")]
 pub struct Waker {
+    #[cfg(feature = "turbowakers")]
+    ptr: crate::ptr::NonNull<()>,
+    #[cfg(not(feature = "turbowakers"))]
     waker: RawWaker,
 }
 
@@ -378,6 +381,9 @@ unsafe impl Send for Waker {}
 #[stable(feature = "futures_api", since = "1.36.0")]
 unsafe impl Sync for Waker {}
 
+#[cfg(not(feature = "turbowakers"))]
+mod waker {
+use super::*;
 impl Waker {
     /// Wake up the task associated with this `Waker`.
     ///
@@ -545,6 +551,120 @@ impl fmt::Debug for Waker {
     }
 }
 
+}
+
+
+#[cfg(feature = "turbowakers")]
+mod waker {
+    use crate::ptr::NonNull;
+
+    use super::*;
+    extern "Rust" {
+        fn _turbo_wake(ptr: NonNull<()>);
+    }
+
+    impl Waker {
+        /// Wake up the task associated with this `Waker`.
+        ///
+        /// As long as the executor keeps running and the task is not finished, it is
+        /// guaranteed that each invocation of [`wake()`](Self::wake) (or
+        /// [`wake_by_ref()`](Self::wake_by_ref)) will be followed by at least one
+        /// [`poll()`] of the task to which this `Waker` belongs. This makes
+        /// it possible to temporarily yield to other tasks while running potentially
+        /// unbounded processing loops.
+        ///
+        /// Note that the above implies that multiple wake-ups may be coalesced into a
+        /// single [`poll()`] invocation by the runtime.
+        ///
+        /// Also note that yielding to competing tasks is not guaranteed: it is the
+        /// executor’s choice which task to run and the executor may choose to run the
+        /// current task again.
+        ///
+        /// [`poll()`]: crate::future::Future::poll
+        #[inline]
+        #[stable(feature = "futures_api", since = "1.36.0")]
+        pub fn wake(self) {
+            unsafe { _turbo_wake(self.ptr) }
+        }
+
+        /// Wake up the task associated with this `Waker` without consuming the `Waker`.
+        ///
+        /// This is similar to [`wake()`](Self::wake), but may be slightly less efficient in
+        /// the case where an owned `Waker` is available. This method should be preferred to
+        /// calling `waker.clone().wake()`.
+        #[inline]
+        #[stable(feature = "futures_api", since = "1.36.0")]
+        pub fn wake_by_ref(&self) {
+            unsafe { _turbo_wake(self.ptr) }
+        }
+
+        /// Returns `true` if this `Waker` and another `Waker` would awake the same task.
+        ///
+        /// This function works on a best-effort basis, and may return false even
+        /// when the `Waker`s would awaken the same task. However, if this function
+        /// returns `true`, it is guaranteed that the `Waker`s will awaken the same task.
+        ///
+        /// This function is primarily used for optimization purposes.
+        #[inline]
+        #[must_use]
+        #[stable(feature = "futures_api", since = "1.36.0")]
+        pub fn will_wake(&self, other: &Waker) -> bool {
+            self.ptr == other.ptr
+        }
+
+        /// Creates a new `Waker` from [`RawWaker`].
+        ///
+        /// The behavior of the returned `Waker` is undefined if the contract defined
+        /// in [`RawWaker`]'s and [`RawWakerVTable`]'s documentation is not upheld.
+        /// Therefore this method is unsafe.
+        #[inline]
+        #[must_use]
+        #[stable(feature = "futures_api", since = "1.36.0")]
+        #[rustc_const_unstable(feature = "const_waker", issue = "102012")]
+        pub const unsafe fn from_raw(waker: RawWaker) -> Waker {
+            panic!("Waker::from_raw is unavailable due to enabling turbowakers.");
+        }
+
+        /// Get a reference to the underlying [`RawWaker`].
+        #[inline]
+        #[must_use]
+        #[unstable(feature = "waker_getters", issue = "87021")]
+        pub fn as_raw(&self) -> &RawWaker {
+            panic!("Waker::as_raw is unavailable due to enabling turbowakers.");
+        }
+
+        #[inline]
+        #[must_use]
+        #[stable(feature = "futures_api", since = "1.36.0")]
+        #[rustc_const_unstable(feature = "const_waker", issue = "102012")]
+        pub const unsafe fn from_turbo_ptr(ptr: NonNull<()>) -> Waker {
+            Self { ptr }
+        }
+
+        #[inline]
+        #[must_use]
+        #[stable(feature = "futures_api", since = "1.36.0")]
+        pub fn as_turbo_ptr(&self) -> NonNull<()> {
+            self.ptr
+        }
+    }
+
+    #[stable(feature = "futures_api", since = "1.36.0")]
+    impl Clone for Waker {
+        #[inline]
+        fn clone(&self) -> Self {
+            Self { ptr: self.ptr }
+        }
+    }
+
+    #[stable(feature = "futures_api", since = "1.36.0")]
+    impl fmt::Debug for Waker {
+        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+            f.debug_struct("Waker").field("data", &self.ptr).finish()
+        }
+    }
+}
+
 /// A `LocalWaker` is analogous to a [`Waker`], but it does not implement [`Send`] or [`Sync`].
 ///
 /// This handle encapsulates a [`RawWaker`] instance, which defines the
EOF
