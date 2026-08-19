pub(crate) mod disk_io;
pub mod lifecycle;
/// C1-C4 lifecycle policy suite. A sibling module rather than a child of
/// `lifecycle::tests` deliberately: it drives ONLY the public API, so it cannot
/// quietly depend on internals the product path does not use.
#[cfg(test)]
mod lifecycle_policy_tests;
mod signing;
pub mod state_lock;
pub mod updater_state;

pub use signing::{check_patch_signature, hash_file};
pub use state_lock::StateLock;
pub use updater_state::UpdaterState;

/// The public interface for talking about patches to the Cache.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PatchInfo {
    pub path: std::path::PathBuf,
    pub number: usize,
}
