use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::ptr;
use core::sync::atomic::Ordering;
use std::thread;

use crossbeam_epoch::{Guard, Owned, Shared};

use super::base::{ELIM_DELAY, ElimStack, Stack, get_random_elim_index};

// 槽位状态保存在指针低位的 tag 中。请求节点至少按指针对齐，因此有两个空闲低位可用。
const EMPTY: usize = 0;
const PUSH_REQUEST: usize = 1;
const POP_REQUEST: usize = 2;
const ACKNOWLEDGED: usize = 3;

/// 从已经成功配对的 push 请求中取出值。
///
/// # Safety
///
/// 调用方必须已经通过 CAS 独占这次配对，保证只有当前线程会读取其中的 `T`。
/// 节点里的值使用 `ManuallyDrop<T>` 保存，因此稍后回收节点时不会再次析构 `T`。
///
// 尝试 push 到 Treiber stack
// │
// ├── 成功：结束
// │
// └── CAS 失败：随机选择一个 slot
//     │
//     ├── POP_REQUEST
//     │   ├── CAS 成功：与 pop 配对，结束
//     │   └── CAS 失败：返回 Err(req)，外层重新尝试
//     │
//     ├── EMPTY
//     │   ├── 发布 PUSH_REQUEST 失败：立即返回 Err(req)
//     │   └── 发布 PUSH_REQUEST 成功
//     │       ├── 等待 10ms
//     │       └── 尝试撤回
//     │           ├── 撤回成功：没有配对，返回 Err(req)
//     │           └── 撤回失败：已被 pop 确认，回收节点，结束
//     │
//     └── PUSH_REQUEST / ACKNOWLEDGED
//         └── 不能配对，立即返回 Err(req)
unsafe fn take_value<T, R>(request: Shared<'_, R>) -> T
where
    R: Deref<Target = ManuallyDrop<T>>,
{
    // SAFETY: 上述约束保证 `request` 有效，且其中的值只会被移动一次。
    let value = unsafe { ptr::read(request.deref().deref()) };
    ManuallyDrop::into_inner(value)
}

impl<T, S: Stack<T>> Stack<T> for ElimStack<T, S> {
    type PushReq = S::PushReq;

    fn try_push(
        &self,
        req: Owned<Self::PushReq>,
        guard: &Guard,
    ) -> Result<(), Owned<Self::PushReq>> {
        let Err(req) = self.inner.try_push(req, guard) else {
            return Ok(());
        };

        let index = get_random_elim_index();
        // SAFETY: `get_random_elim_index` 的返回值始终小于槽位数组长度。
        let slot_ref = unsafe { self.slots.get_unchecked(index) };
        let slot = slot_ref.load(Ordering::Acquire, guard);

        match slot.tag() {
            POP_REQUEST => {
                // 已有 pop 在等待：直接把当前节点交给它，并把状态改为“已确认”。
                // 等待的 pop 线程负责取值、清空槽位以及回收节点。
                match slot_ref.compare_exchange(
                    slot,
                    req.with_tag(ACKNOWLEDGED),
                    Ordering::Release,
                    Ordering::Relaxed,
                    guard,
                ) {
                    Ok(_) => Ok(()),
                    // 配对失败时所有权仍在 `error.new` 中；清除 tag 后交还给外层重试。
                    Err(error) => Err(error.new.with_tag(EMPTY)),
                }
            }
            EMPTY => {
                // 没有可配对的 pop，把 push 请求发布到槽位中，短暂等待一个 pop。
                let published = match slot_ref.compare_exchange(
                    slot,
                    req.with_tag(PUSH_REQUEST),
                    Ordering::Release,
                    Ordering::Relaxed,
                    guard,
                ) {
                    Ok(published) => published,
                    Err(error) => return Err(error.new.with_tag(EMPTY)),
                };

                thread::sleep(ELIM_DELAY);

                // 尝试撤回自己的请求。成功说明无人配对；失败则只能是某个 pop
                // 已把同一指针的 tag 从 PUSH_REQUEST 改成 ACKNOWLEDGED。
                match slot_ref.compare_exchange(
                    published,
                    Shared::null(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                    guard,
                ) {
                    Ok(_) => {
                        // SAFETY: 撤回 CAS 成功后，槽位不再引用该节点，所有权重新归当前线程。
                        let req = unsafe { published.into_owned() }.with_tag(EMPTY);
                        Err(req)
                    }
                    Err(error) => {
                        debug_assert_eq!(error.current, published.with_tag(ACKNOWLEDGED));

                        // 配对方已经取走 T。先让槽位重新可用，再延迟回收只剩外壳的节点；
                        // epoch guard 会保证配对线程结束访问前节点不会真正释放。
                        slot_ref.store(Shared::null(), Ordering::Release);
                        // SAFETY: 当前线程是这个 push 请求的等待方，确认配对后负责回收节点；
                        // 此后没有线程能再次取得该节点的所有权。
                        unsafe { guard.defer_destroy(published.with_tag(EMPTY)) };
                        Ok(())
                    }
                }
            }
            // 同方向请求或尚未被等待方清理的确认状态都不能与当前 push 配对。
            PUSH_REQUEST | ACKNOWLEDGED => Err(req),
            _ => unreachable!("槽位 tag 只能使用两个低位"),
        }
    }

    // ElimStack::try_pop()
    // │
    // ├── 尝试底层 Treiber stack
    // │   │
    // │   ├── 成功弹出值
    // │   │   └── Ok(Some(value))
    // │   │
    // │   ├── 栈为空
    // │   │   └── Ok(None)
    // │   │
    // │   └── CAS 冲突
    // │       └── 进入 elimination
    // │
    // └── 随机选择一个 slot
    //     │
    //     ├── PUSH_REQUEST
    //     │   │
    //     │   ├── CAS 改为 ACKNOWLEDGED 成功
    //     │   │   ├── 取出 push 的值
    //     │   │   └── Ok(Some(value))
    //     │   │
    //     │   └── CAS 失败
    //     │       └── Err(())
    //     │
    //     ├── EMPTY
    //     │   │
    //     │   ├── 发布 POP_REQUEST 失败
    //     │   │   └── Err(())
    //     │   │
    //     │   └── 发布成功
    //     │       ├── 等待 10ms
    //     │       └── 尝试撤回
    //     │           │
    //     │           ├── 撤回成功
    //     │           │   └── 没有 push，Err(())
    //     │           │
    //     │           └── 撤回失败
    //     │               ├── push 已配对
    //     │               ├── 取出值
    //     │               ├── 清空 slot
    //     │               ├── 延迟回收节点
    //     │               └── Ok(Some(value))
    //     │
    //     ├── POP_REQUEST
    //     │   └── 同方向，不能配对，Err(())
    //     │
    //     └── ACKNOWLEDGED
    //         └── 属于上一组配对，不能使用，Err(())
    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()> {
        if let Ok(result) = self.inner.try_pop(guard) {
            return Ok(result);
        }

        let index = get_random_elim_index();
        // SAFETY: `get_random_elim_index` 的返回值始终小于槽位数组长度。
        let slot_ref = unsafe { self.slots.get_unchecked(index) };
        let slot = slot_ref.load(Ordering::Acquire, guard);

        match slot.tag() {
            PUSH_REQUEST => {
                // 已有 push 在等待。CAS 成功后，本线程是唯一有权取出 T 的 pop；
                // 发布 push 的线程看到 ACKNOWLEDGED 后会清空槽位并回收节点。
                if slot_ref
                    .compare_exchange(
                        slot,
                        slot.with_tag(ACKNOWLEDGED),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                        guard,
                    )
                    .is_err()
                {
                    return Err(());
                }

                // SAFETY: 上面的 CAS 成功，使当前线程独占这次 push/pop 配对。
                Ok(Some(unsafe { take_value(slot) }))
            }
            EMPTY => {
                // 没有可配对的 push，发布一个不携带指针的 pop 请求并短暂等待。
                let pop_request = Shared::<Self::PushReq>::null().with_tag(POP_REQUEST);
                if slot_ref
                    .compare_exchange(
                        slot,
                        pop_request,
                        Ordering::Release,
                        Ordering::Relaxed,
                        guard,
                    )
                    .is_err()
                {
                    return Err(());
                }

                thread::sleep(ELIM_DELAY);

                // 若撤回成功，说明等待期间没有 push 到来；若失败，则槽位中已经是
                // push 留下的“节点指针 + ACKNOWLEDGED”。
                match slot_ref.compare_exchange(
                    pop_request,
                    Shared::null(),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                    guard,
                ) {
                    Ok(_) => Err(()),
                    Err(error) => {
                        let acknowledged = error.current;
                        debug_assert_eq!(acknowledged.tag(), ACKNOWLEDGED);
                        debug_assert!(!acknowledged.is_null());

                        // 当前 pop 是请求的等待方，所以由它取值、清空槽位并回收节点。
                        // SAFETY: push 只有成功把 POP_REQUEST 改成 ACKNOWLEDGED 才会返回，
                        // 因而当前线程独占该值。
                        let value = unsafe { take_value(acknowledged) };
                        slot_ref.store(Shared::null(), Ordering::Release);
                        // SAFETY: 节点已从槽位移除，且 T 已移入 `value`；epoch 负责推迟释放，
                        // 直到可能观察过该指针的线程都离开临界区。
                        unsafe { guard.defer_destroy(acknowledged.with_tag(EMPTY)) };
                        Ok(Some(value))
                    }
                }
            }
            // pop 不能和另一个 pop 配对；ACKNOWLEDGED 要留给原等待方清理。
            POP_REQUEST | ACKNOWLEDGED => Err(()),
            _ => unreachable!("槽位 tag 只能使用两个低位"),
        }
    }

    fn is_empty(&self, guard: &Guard) -> bool {
        self.inner.is_empty(guard)
    }
}
