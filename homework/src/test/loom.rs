//! Re-exports loom if `feature = "check-loom"`. Otherwise, std.
//! 如果 `feature = "check-loom"`，则重新出口迫在眉睫。否则，使用 std。

#[cfg(not(feature = "check-loom"))]
pub use std::*;

#[cfg(feature = "check-loom")]
pub use loom::*;

/// Run `f` with `loom::model` if compiled with `check-loom` feature.
/// 如果使用了 `check-loom` 功能编译，请使用 `loom::model` 运行 `f`。
pub fn model<F: Fn() + Sync + Send + 'static>(f: F) {
    cfg_if::cfg_if! {
        if #[cfg(feature = "check-loom")] {
            loom::model(f)
        } else {
            f()
        }
    }
}
