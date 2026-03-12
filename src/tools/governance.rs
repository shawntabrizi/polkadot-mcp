// Governance (OpenGov) tools: referenda_active, referendum_detail, vote, delegate, my_votes.
//
// All governance tools query the relay chain automatically.
// Chain routing: hardcoded to `self.pool.relay()`.
//
// Key storage items on relay chain:
//   - Referenda.ReferendumInfoFor[index] → Ongoing{...} or terminal state
//   - Referenda.ReferendumCount → u32
//   - ConvictionVoting.VotingFor[account][track] → Casting{votes} or Delegating{...}
//   - ConvictionVoting.ClassLocksFor[account] → Vec<(class, balance)>
//
// Tracks (classes): 0=Root, 1=WhitelistedCaller, 2=WishForChange,
//   10=StakingAdmin, 11=Treasurer, 12=LeaseAdmin, 13=FellowshipAdmin,
//   14=GeneralAdmin, 15=AuctionAdmin, 20=ReferendumCanceller,
//   21=ReferendumKiller, 30=SmallTipper, 31=BigTipper,
//   32=SmallSpender, 33=MediumSpender, 34=BigSpender

// TODO: Implement tools
//
// #[tool(description = "List all active governance referenda. Shows referendum index, \
//     track name, current tally (aye/nay percentages), status (preparing/deciding/confirming), \
//     and time remaining. Optionally filter by track.")]
// async fn referenda_active(&self, params: ReferendaActiveParams) -> Result<CallToolResult>
//
// #[tool(description = "Get full details on a specific referendum by index. Shows track, \
//     origin, decoded proposal call data, tally, timeline, deposits, and current status.")]
// async fn referendum_detail(&self, params: ReferendumDetailParams) -> Result<CallToolResult>
//
// #[tool(description = "Show all your active votes across all governance tracks. Includes \
//     vote direction, conviction, locked balance, and when conviction locks expire. \
//     Also shows any active delegations.")]
// async fn my_votes(&self, params: MyVotesParams) -> Result<CallToolResult>
//
// #[tool(description = "Show current vote delegations by track. Shows who you delegate to, \
//     conviction level, and delegated balance for each track.")]
// async fn delegation_status(&self, params: DelegationParams) -> Result<CallToolResult>
//
// #[tool(description = "Vote on a governance referendum. Specify aye/nay, conviction (0-6), \
//     and balance. IMPORTANT: This submits a real transaction. Confirm vote details with \
//     the user first.")]
// async fn vote(&self, params: VoteParams) -> Result<CallToolResult>
//
// #[tool(description = "Delegate your votes on one or all governance tracks to another account. \
//     IMPORTANT: This submits a real transaction. Confirm with the user first.")]
// async fn delegate(&self, params: DelegateParams) -> Result<CallToolResult>
