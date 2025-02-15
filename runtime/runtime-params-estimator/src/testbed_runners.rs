use near_primitives::types::AccountId;
use near_vm_runner::internal::VMKind;
use std::path::PathBuf;

/// Get account id from its index.
pub fn get_account_id(account_index: usize) -> AccountId {
    AccountId::try_from(format!("near_{}_{}", account_index, account_index)).unwrap()
}

/// Total number of transactions that we need to prepare.
pub fn total_transactions(config: &Config) -> usize {
    config.block_sizes.iter().sum::<usize>() * config.iter_per_block
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasMetric {
    // If we measure gas in number of executed instructions, must run under simulator.
    ICount,
    // If we measure gas in elapsed time.
    Time,
}

/// Configuration which we use to run measurements.
#[derive(Debug, Clone)]
pub struct Config {
    /// How many warm up iterations per block should we run.
    pub warmup_iters_per_block: usize,
    /// How many iterations per block are we going to try.
    pub iter_per_block: usize,
    /// Total active accounts.
    pub active_accounts: usize,
    /// Number of the transactions in the block.
    pub block_sizes: Vec<usize>,
    /// Where state dump is located in case we need to create a testbed.
    pub state_dump_path: PathBuf,
    /// Metric used for counting.
    pub metric: GasMetric,
    /// VMKind used
    pub vm_kind: VMKind,
    /// When non-none, only the specified metrics will be measured.
    pub metrics_to_measure: Option<Vec<String>>,
}
