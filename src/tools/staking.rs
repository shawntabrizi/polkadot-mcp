// Staking tools: staking_status, staking_rewards, nomination_pools, claim_rewards.
//
// All staking tools query the relay chain automatically.
// Chain routing: hardcoded to `self.pool.relay()`.
//
// Key storage items on relay chain:
//   - Staking.Bonded[stash] → controller
//   - Staking.Ledger[controller] → { stash, total, active, unlocking }
//   - Staking.Nominators[account] → { targets, submitted_in }
//   - Staking.CurrentEra → EraIndex
//   - NominationPools.PoolMembers[account] → { pool_id, points, unbonding_eras }
//   - NominationPools.BondedPools[pool_id] → { commission, member_counter, points, roles, state }
//   - NominationPools.RewardPools[pool_id] → reward tracking

// TODO: Implement tools
//
// #[tool(description = "Check staking status: bonded amount, active nominations, \
//     pending unbonds with unlock times, and nomination pool membership if any.")]
// async fn staking_status(&self, params: StakingParams) -> Result<CallToolResult>
//
// #[tool(description = "Check staking rewards: pending unclaimed payouts and \
//     historical reward summary. Uses on-chain data for pending and Subscan API \
//     for historical rewards.")]
// async fn staking_rewards(&self, params: StakingRewardsParams) -> Result<CallToolResult>
//
// #[tool(description = "Check nomination pool membership: your stake, pending rewards, \
//     pool state, member count, and commission rate.")]
// async fn nomination_pools(&self, params: StakingParams) -> Result<CallToolResult>
//
// #[tool(description = "Claim pending staking rewards or nomination pool rewards. \
//     IMPORTANT: This submits a real transaction. Confirm with the user first.")]
// async fn claim_rewards(&self, params: ClaimRewardsParams) -> Result<CallToolResult>
