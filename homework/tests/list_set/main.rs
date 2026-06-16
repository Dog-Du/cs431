// optimistic_fine_grained on thread santizer has very unstable performance on gg.
// 在线程检测器上，optimistic_fine_grained 在 gg 上的性能非常不稳定。
#![feature(cfg_sanitize)]

mod fine_grained;
mod optimistic_fine_grained;
