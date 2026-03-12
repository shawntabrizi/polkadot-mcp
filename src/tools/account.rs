// Account & balance tools: account_info, account_balances, account_locks, unlock_frozen.
//
// These tools work on any Substrate chain (default: relay chain).
// Chain routing: defaults to relay, accepts optional `chain` parameter override.

// TODO: Implement tools
//
// #[tool(description = "Get full account overview: free, reserved, frozen, and \
//     transferable balance, plus lock breakdown with reasons. Defaults to relay chain. \
//     Use 'chain' param for other chains: 'polkadot-asset-hub', 'polkadot-collectives'.")]
// async fn account_info(&self, params: AccountInfoParams) -> Result<CallToolResult>
//
// #[tool(description = "Get balances for multiple accounts at once. Returns free, \
//     reserved, frozen, and transferable balance for each address. Set chain to 'all' \
//     to show balances across relay chain and Asset Hub with totals.")]
// async fn account_balances(&self, params: AccountBalancesParams) -> Result<CallToolResult>
//
// #[tool(description = "Show all balance locks with reasons and unlock conditions. \
//     Covers staking locks, governance conviction locks, vesting schedules, and holds. \
//     Tells you exactly why your balance is frozen and when it unlocks.")]
// async fn account_locks(&self, params: AccountLocksParams) -> Result<CallToolResult>
//
// #[tool(description = "Build transaction(s) to unlock expired balance locks. \
//     Checks for expired conviction voting locks and completed vesting schedules. \
//     IMPORTANT: This submits a real transaction. Confirm with the user first.")]
// async fn unlock_frozen(&self, params: UnlockFrozenParams) -> Result<CallToolResult>
