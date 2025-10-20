//! Smart handles for pool-allocated objects.

mod owned;
mod shared;
mod weak;

pub use owned::{OwnedHandle, PoolInterface};
pub use shared::SharedHandle;
pub use weak::WeakHandle;
