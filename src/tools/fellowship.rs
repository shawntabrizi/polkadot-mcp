// Fellowship tools: fellowship_status, fellowship_salary, fellowship_demotion_risk, etc.
//
// All fellowship tools query the Collectives parachain automatically.
// Chain routing: hardcoded to `self.pool.collectives()`.
//
// Key storage items on Collectives chain:
//   - FellowshipCollective.Members[account] → { rank }
//   - CoreFellowship.Member[account] → { is_active, last_promotion, last_proof }
//   - CoreFellowship.Params → { demotion_period[], min_promotion_period }
//   - FellowshipSalary.Claimant[account] → { last_active, status }
//   - FellowshipSalary.Status → { cycle, cycle_start, budget }

// TODO: Implement tools
//
// #[tool(description = "Check fellowship membership status: rank, activity status, \
//     last promotion date, demotion risk, and salary eligibility. Automatically queries \
//     the Polkadot Collectives chain.")]
// async fn fellowship_status(&self, params: FellowshipParams) -> Result<CallToolResult>
//
// #[tool(description = "Check if fellowship salary is claimable this cycle. Returns \
//     the salary amount based on rank, whether you need to register or claim, and \
//     the current cycle status.")]
// async fn fellowship_salary(&self, params: FellowshipParams) -> Result<CallToolResult>
//
// #[tool(description = "Check demotion risk: how many blocks remain before you are \
//     eligible for auto-demotion. Shows what qualifies as activity proof and how to \
//     submit evidence.")]
// async fn fellowship_demotion_risk(&self, params: FellowshipParams) -> Result<CallToolResult>
//
// #[tool(description = "List all fellowship members, optionally filtered by minimum rank. \
//     Shows rank, address, and activity status for each member.")]
// async fn fellowship_members(&self, params: FellowshipMembersParams) -> Result<CallToolResult>
//
// #[tool(description = "Claim fellowship salary. Builds register() + claim() extrinsics. \
//     IMPORTANT: This submits a real transaction. Confirm with the user first.")]
// async fn fellowship_claim_salary(&self, params: FellowshipParams) -> Result<CallToolResult>
