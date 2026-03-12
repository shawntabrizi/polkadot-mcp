# polkadot-mcp — Tool Catalog

All tools accept `network` (polkadot/kusama/westend/paseo) and `chain` parameters unless noted otherwise. Tools marked **[write]** submit transactions and require a signer. Tools marked **[indexer]** use Subscan or similar indexed backend for historical data.

**Status key:** implemented, stub (file exists, not yet implemented), planned (not started)

---

## 1. Chain Utilities

General-purpose tools for querying any chain.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `chain_info` | implemented | any | Chain name, type, token, decimals, SS58 prefix, current block, spec/tx version |
| `block_info` | planned | any | Block details: number, hash, parent, timestamp, extrinsics count, decoded events |
| `extrinsic_info` | planned | any | Look up a specific extrinsic by hash or by block number + index. Shows decoded call, signer, fee, success/failure, events emitted. Answers: "did my transaction go through?" |
| `fee_estimate` | planned | any | Estimate the fee for a given call without submitting. Uses `payment_queryInfo` or dry-run. Answers: "how much will this cost?" |
| `dry_run` | planned | any | Simulate a call against current state and return the execution result + events without submitting. Answers: "what would happen if I did X?" Shows success/failure, dispatch error details, and all events that would be emitted |
| `encode_call` | planned | any | Build hex-encoded call data from pallet + method + params. Inverse of `decode_call`. Needed for constructing multisig proposals, proxy calls, governance preimages, and batch payloads |
| `scheduled_actions` | planned | any | List pending scheduled dispatches: governance enactments, runtime upgrades, etc. Queries `Scheduler.Agenda` and `Scheduler.Lookup` |
| `remark` | planned [write] | any | Store arbitrary data on-chain via `System.remark_with_event`. Used for signaling, attestation, and data anchoring |

---

## 2. Address Utilities

Offline tools for working with SS58 addresses. No chain connection needed.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `ss58_decode` | implemented | none | Decode an SS58 address into its raw public key (hex) and SS58 prefix. Answers: "what prefix/network is this address for?" |
| `ss58_encode` | implemented | none | Encode a raw public key (hex) into an SS58 address with a given prefix. Answers: "what's this account's address on Kusama?" |
| `ss58_convert` | implemented | none | Convert an SS58 address from one prefix to another. Answers: "is this the same account on Polkadot and Kusama?" Shows the address in both formats |
| `ss58_validate` | implemented | none | Validate an SS58 address: check encoding, extract prefix, identify the network it belongs to |

### SS58 prefix reference
| Prefix | Network |
|---|---|
| 0 | Polkadot |
| 2 | Kusama |
| 42 | Generic Substrate / Westend / Paseo |

---

## 3. Runtime Metadata Introspection

These tools parse the on-chain metadata that every Substrate chain exposes. The metadata describes the entire runtime: every pallet, extrinsic, storage item, event, error, and constant — with types and documentation. This is the AI's "map" of what a chain can do.

The `subxt` client already downloads metadata on first connection (~200KB–1MB). These tools just expose it in structured, digestible pieces so the agent can explore any chain without prior knowledge.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `list_pallets` | implemented | any | List all pallets in the runtime with their index. Answers: "what pallets does this chain have?" |
| `pallet_info` | implemented | any | Overview of a single pallet: description (from docs), number of calls/storage/events/errors/constants. Answers: "does this chain have pallet X? what does it do?" |
| `list_calls` | planned | any | List all extrinsics (dispatchable calls) for a pallet with one-line descriptions. Answers: "what can I do with the Staking pallet?" |
| `call_info` | planned | any | Full detail for a specific call: parameters with names and types, documentation, dispatch origin. Answers: "what are the params for `Staking.nominate`?" |
| `list_storage` | implemented | any | List all storage entries for a pallet: name, key types, value type, hashers, modifier (Optional/Default). Answers: "what data does the Staking pallet store?" |
| `storage_info` | planned | any | Full detail for a specific storage entry: key types, value type, default value, documentation. Answers: "what does `Staking.Ledger` contain and what are its keys?" |
| `list_events` | planned | any | List all events for a pallet with field names and descriptions. Answers: "what events does Balances emit?" |
| `list_errors` | planned | any | List all errors for a pallet with descriptions. Answers: "what errors can `ConvictionVoting.vote` return?" |
| `list_constants` | planned | any | List all runtime constants for a pallet with values and descriptions. Answers: "what are the configuration parameters for staking?" |
| `constant_value` | implemented | any | Get the decoded value of a specific runtime constant. Answers: "what is the existential deposit?" / "how long is the unbonding period?" |
| `query_storage` | planned | any | Read any storage item by pallet + entry name + keys. Supports optional `at_block` parameter for historical state. Raw decoded value output. Escape hatch for anything not covered by a dedicated tool |
| `decode_call` | planned | any | Decode a hex-encoded call/extrinsic into human-readable pallet + method + args using chain metadata |
| `type_info` | planned | any | Describe a type by ID from the metadata type registry. Useful for understanding complex nested types returned by other tools |
| `runtime_apis` | planned | any | List available runtime APIs and their methods. Some chains expose custom APIs beyond the standard set |

### Why this matters

Every Substrate chain is different — parachains add custom pallets, runtime upgrades change parameters, and testnets may have different configurations than mainnet. These tools let the AI:

- **Discover capabilities** on unfamiliar chains without hardcoded knowledge
- **Answer "how do I..." questions** by looking up exact call signatures and docs
- **Debug errors** by looking up error variants and their meanings
- **Explain storage** by describing what data a pallet tracks and how to read it
- **Compare chains** — e.g. "does Kusama have the same staking parameters as Polkadot?"

### Implementation notes

All metadata is available through `api.metadata()` after connecting via subxt. Key types:
- `metadata.pallets()` → iterate pallet metadata
- `pallet.calls()` → call variants with fields and docs
- `pallet.storage()` → storage entries with type info
- `pallet.constants()` → constants with encoded values
- `pallet.events()` → event variants
- `pallet.errors()` → error variants
- `metadata.types()` → full type registry for resolving type IDs

---

## 4. Account & Balance

Core account queries. Work on **any chain** (relay or parachain).

| Tool | Status | Chain | Description |
|---|---|---|---|
| `get_balances` | implemented | any | Free, reserved, frozen, transferable balance for an SS58 address |
| `account_locks` | implemented | any | All locks, freezes, and holds with reasons and amounts. Combines `Balances.Locks`, `Balances.Freezes`, `Balances.Holds` |
| `account_transfers` | planned [indexer] | any | Recent transfer history (inbound/outbound) with timestamps, counterparty, amount |
| `transfer` | planned [write] | any | Transfer native token. Builds `Balances.transfer_keep_alive` or `transfer_allow_death`. Dry-runs first |
| `vesting_info` | planned | relay | Vesting schedules: locked amount, per-block release, already vested. Queries `Vesting.Vesting[account]` |
| `vest` | planned [write] | relay | Release vested tokens. Builds `Vesting.vest()` or `Vesting.vest_other(target)` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `System` | `Account` | AccountId | `{ nonce, consumers, providers, sufficients, data: { free, reserved, frozen } }` |
| `Balances` | `Locks` | AccountId | `Vec<{ id, amount, reasons }>` |
| `Balances` | `Freezes` | AccountId | `Vec<{ id, amount }>` |
| `Balances` | `Holds` | AccountId | `Vec<{ id, amount }>` |
| `Vesting` | `Vesting` | AccountId | `Vec<{ locked, per_block, starting_block }>` |

---

## 5. Staking (Relay Chain)

Direct staking (nominating validators) and nomination pools.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `staking_status` | stub | relay | Bonded amount, active vs inactive stake, nominations list, pending unbonds with unlock eras |
| `staking_rewards` | stub [indexer] | relay | Pending unclaimed rewards + historical payouts per era |
| `nomination_pools` | stub | relay | Pool membership: pool ID, your points/stake, pending rewards, pool state, commission |
| `pool_info` | planned | relay | Details on a specific nomination pool by ID: state, member count, roles, points, commission |
| `validators_info` | planned | relay | Current active validator set, era points, commission rates. Useful for nomination decisions |
| `validator_detail` | planned | relay | Single validator details: commission, self-stake, total stake, nominator count, identity, era points history, slash history |
| `pools_list` | planned | relay | Browse nomination pools: list by state (Open/Blocked/Destroying), member count, total stake. Answers: "what pools can I join?" |
| `pool_create` | planned [write] | relay | Create a new nomination pool. Builds `NominationPools.create(amount, root, nominator, bouncer)` |
| `bond` | planned [write] | relay | Bond tokens for staking. Builds `Staking.bond(value, payee)` |
| `nominate` | planned [write] | relay | Set nomination targets. Builds `Staking.nominate(targets)` |
| `unbond` | planned [write] | relay | Unbond tokens. Builds `Staking.unbond(value)`. Shows unlock timeline |
| `withdraw_unbonded` | planned [write] | relay | Withdraw fully unbonded tokens. Builds `Staking.withdraw_unbonded(num_slashing_spans)` |
| `chill` | planned [write] | relay | Stop nominating. Builds `Staking.chill()` |
| `claim_staking_rewards` | stub [write] | relay | Claim pending staking payouts. Builds `Staking.payout_stakers(validator, era)` |
| `pool_join` | planned [write] | relay | Join a nomination pool. Builds `NominationPools.join(amount, pool_id)` |
| `pool_bond_extra` | planned [write] | relay | Add more stake to pool. Builds `NominationPools.bond_extra(extra)` |
| `pool_claim_payout` | planned [write] | relay | Claim pool rewards. Builds `NominationPools.claim_payout()` |
| `pool_unbond` | planned [write] | relay | Unbond from pool. Builds `NominationPools.unbond(member, unbonding_points)` |
| `pool_withdraw` | planned [write] | relay | Withdraw unbonded from pool. Builds `NominationPools.withdraw_unbonded(member, num_slashing_spans)` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Staking` | `Bonded` | StashId | ControllerId |
| `Staking` | `Ledger` | ControllerId | `{ stash, total, active, unlocking }` |
| `Staking` | `Nominators` | AccountId | `{ targets, submitted_in }` |
| `Staking` | `Validators` | AccountId | `{ commission, blocked }` |
| `Staking` | `CurrentEra` | — | EraIndex |
| `Staking` | `ActiveEra` | — | `{ index, start }` |
| `Staking` | `ErasStakers` | (EraIndex, AccountId) | `{ total, own, others }` |
| `Staking` | `ErasValidatorReward` | EraIndex | Balance |
| `NominationPools` | `PoolMembers` | AccountId | `{ pool_id, points, unbonding_eras }` |
| `NominationPools` | `BondedPools` | PoolId | `{ commission, member_counter, points, roles, state }` |
| `NominationPools` | `RewardPools` | PoolId | `{ last_recorded_reward_counter, ... }` |
| `NominationPools` | `CounterForPoolMembers` | — | u32 |

---

## 6. Governance — OpenGov (Relay Chain)

Referendum lifecycle, conviction voting, and delegation.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `referenda_active` | stub | relay | List all active referenda: index, track, tally (aye/nay/support %), status (preparing/deciding/confirming), time remaining |
| `referendum_detail` | stub | relay | Full details for a referendum by index: track, origin, proposal call data (decoded), tally, timeline, deposits |
| `my_votes` | stub | relay | All active votes across all tracks with conviction, balance, lock expiry. Shows delegations too |
| `delegation_status` | stub | relay | Current delegation per track: delegate address, conviction, balance |
| `preimage_info` | planned | relay | Look up and decode a preimage by hash. Shows the decoded call data of a governance proposal. Answers: "what does this referendum actually propose?" Queries `Preimage.PreimageFor` and `Preimage.StatusFor` |
| `vote` | stub [write] | relay | Vote on a referendum. Supports Standard (aye/nay with conviction), Split (aye + nay amounts), and SplitAbstain (aye + nay + abstain amounts). Builds `ConvictionVoting.vote()` |
| `delegate` | stub [write] | relay | Delegate voting power on one or all tracks. Builds `ConvictionVoting.delegate()` |
| `undelegate` | planned [write] | relay | Remove delegation on a track. Builds `ConvictionVoting.undelegate(class)` |
| `unlock_votes` | planned [write] | relay | Unlock expired conviction voting locks. Builds `ConvictionVoting.unlock(class, target)` |
| `remove_vote` | planned [write] | relay | Remove a vote on a specific referendum. Builds `ConvictionVoting.remove_vote(class, index)` |

### Tracks reference
| ID | Name | Example use |
|---|---|---|
| 0 | Root | Runtime upgrades |
| 1 | Whitelisted Caller | Fellowship-approved fast-track |
| 2 | Wish For Change | Non-binding signals |
| 10 | Staking Admin | Staking parameter changes |
| 11 | Treasurer | Treasury spends |
| 12 | Lease Admin | Parachain slot management |
| 13 | Fellowship Admin | Fellowship parameter changes |
| 14 | General Admin | General administrative changes |
| 15 | Auction Admin | Auction parameters |
| 20 | Referendum Canceller | Cancel non-malicious referenda (refunds deposit) |
| 21 | Referendum Killer | Kill malicious referenda (slashes deposit) |
| 30 | Small Tipper | Treasury tips ≤ 250 DOT |
| 31 | Big Tipper | Treasury tips ≤ 1,000 DOT |
| 32 | Small Spender | Treasury spend ≤ 10,000 DOT |
| 33 | Medium Spender | Treasury spend ≤ 100,000 DOT |
| 34 | Big Spender | Treasury spend ≤ 1,000,000 DOT |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Referenda` | `ReferendumInfoFor` | ReferendumIndex | `Ongoing{...}` or terminal state |
| `Referenda` | `ReferendumCount` | — | u32 |
| `ConvictionVoting` | `VotingFor` | (AccountId, TrackId) | `Casting{votes}` or `Delegating{target}` |
| `ConvictionVoting` | `ClassLocksFor` | AccountId | `Vec<(Class, Balance)>` |
| `Preimage` | `StatusFor` | Hash | `Requested { deposit, count, len }` or `Unrequested { deposit, len }` |
| `Preimage` | `PreimageFor` | (Hash, Len) | Raw bytes (decoded via metadata) |

### Conviction table
| Level | Multiplier | Lock period (DOT) |
|---|---|---|
| None | 0.1x | No lock |
| 1 | 1x | 7 days |
| 2 | 2x | 14 days |
| 3 | 3x | 28 days |
| 4 | 4x | 56 days |
| 5 | 5x | 112 days |
| 6 | 6x | 224 days |

---

## 7. Treasury (Relay Chain)

Treasury spend tracking. Most treasury actions happen via governance referenda, but these tools help users understand current treasury state.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `treasury_info` | planned | relay | Treasury balance, next spend period, pending proposals count, approved spends |
| `treasury_spends` | planned | relay | List active/approved treasury spend proposals with amounts, beneficiaries, and status |
| `bounties_list` | planned | relay | List open bounties: index, description, value, curator, status |
| `bounty_detail` | planned | relay | Details on a specific bounty: curator, value, child bounties, status, claims |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Treasury` | `Proposals` | ProposalIndex | `{ proposer, value, beneficiary, bond }` |
| `Treasury` | `Approvals` | — | `Vec<ProposalIndex>` |
| `Bounties` | `Bounties` | BountyIndex | `{ proposer, value, fee, curator_deposit, status }` |
| `Bounties` | `BountyCount` | — | u32 |
| `ChildBounties` | `ChildBounties` | (ParentIndex, ChildIndex) | `{ value, status }` |

---

## 8. Fellowship (Collectives Chain)

Polkadot Technical Fellowship: rank, salary, demotion, evidence. Queries the **Collectives parachain** (not available on Kusama).

| Tool | Status | Chain | Description |
|---|---|---|---|
| `fellowship_status` | stub | collectives | Rank, activity status, last promotion/proof dates, demotion risk, salary eligibility |
| `fellowship_salary` | stub | collectives | Whether salary is claimable this cycle, amount by rank, register/claim instructions |
| `fellowship_demotion_risk` | stub | collectives | Blocks until demotion-eligible. What qualifies as activity proof |
| `fellowship_members` | stub | collectives | List all members with rank. Optionally filter by minimum rank |
| `fellowship_claim_salary` | stub [write] | collectives | Claim salary. Builds `FellowshipSalary.register()` + `FellowshipSalary.claim()` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `FellowshipCollective` | `Members` | AccountId | `MemberRecord { rank }` |
| `FellowshipCollective` | `MemberCount` | Rank | u32 |
| `CoreFellowship` | `Member` | AccountId | `{ is_active, last_promotion, last_proof }` |
| `CoreFellowship` | `MemberEvidence` | AccountId | `(Wish, Evidence)` |
| `CoreFellowship` | `Params` | — | `{ demotion_period[], min_promotion_period, offboard_timeout }` |
| `FellowshipSalary` | `Claimant` | AccountId | `{ last_active, status }` |
| `FellowshipSalary` | `Status` | — | `{ cycle, cycle_start, budget }` |

**Demotion rule:** `current_block - last_proof > demotion_period[rank]` → eligible via `CoreFellowship.bump(who)`

**Salary rule:** Claimable if `last_active != current_cycle && status != Attempted`. Call `register()` then `claim()`.

---

## 9. Identity (People Chain)

On-chain identity management. Lives on the **People parachain**.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `identity_of` | planned | people | Look up on-chain identity for an address: display name, email, web, twitter/X, legal name, riot/matrix, verification status |
| `identity_registrars` | planned | people | List active registrars: index, account, fee, fields they verify |
| `set_identity` | planned [write] | people | Set your on-chain identity fields. Builds `Identity.set_identity(info)` |
| `request_judgement` | planned [write] | people | Request verification from a registrar. Builds `Identity.request_judgement(reg_index, max_fee)` |
| `clear_identity` | planned [write] | people | Remove on-chain identity and recover deposit. Builds `Identity.clear_identity()` |
| `set_subs` | planned [write] | people | Set sub-accounts for your identity. Builds `Identity.set_subs(subs)` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Identity` | `IdentityOf` | AccountId | `(Registration { judgements, deposit, info }, Option<Username>)` |
| `Identity` | `SuperOf` | AccountId | `(SuperAccountId, Data)` |
| `Identity` | `SubsOf` | AccountId | `(deposit, Vec<AccountId>)` |
| `Identity` | `Registrars` | — | `Vec<Option<RegistrarInfo { account, fee, fields }>>` |

### Identity fields
`display`, `legal`, `web`, `email`, `pgp_fingerprint`, `image`, `twitter`, `github`, `discord`

---

## 10. Proxy (Any Chain)

Proxy account management. Available on relay and parachains.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `proxy_list` | planned | any | List all proxies for an address: delegate, proxy type, delay. Queries `Proxy.Proxies[account]` |
| `add_proxy` | planned [write] | any | Add a proxy. Builds `Proxy.add_proxy(delegate, proxy_type, delay)` |
| `remove_proxy` | planned [write] | any | Remove a proxy. Builds `Proxy.remove_proxy(delegate, proxy_type, delay)` |

### Proxy types
`Any`, `NonTransfer`, `Governance`, `Staking`, `IdentityJudgement`, `CancelProxy`, `Auction`, `NominationPools`

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Proxy` | `Proxies` | AccountId | `(Vec<{ delegate, proxy_type, delay }>, deposit)` |
| `Proxy` | `Announcements` | AccountId | `(Vec<{ real, call_hash, height }>, deposit)` |

---

## 11. Multisig (Any Chain)

Multi-signature operations.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `multisig_info` | planned | any | Pending multisig calls for an address. Queries `Multisig.Multisigs` |
| `create_multisig_call` | planned [write] | any | Initiate or approve a multisig call. Builds `Multisig.as_multi()` or `Multisig.approve_as_multi()` |
| `cancel_multisig` | planned [write] | any | Cancel a pending multisig. Builds `Multisig.cancel_as_multi()` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Multisig` | `Multisigs` | (AccountId, CallHash) | `{ when: { height, index }, deposit, depositor, approvals }` |

---

## 12. Assets & NFTs (Asset Hub)

Fungible assets, foreign assets, and NFTs on Asset Hub.

### Fungible assets (local)

Non-native tokens registered on Asset Hub (USDT, USDC, etc.).

| Tool | Status | Chain | Description |
|---|---|---|---|
| `assets_list` | planned | asset-hub | List registered assets with metadata: ID, name, symbol, decimals, supply, admin, status |
| `asset_balance` | planned | asset-hub | Balance of a specific asset for an address. Queries `Assets.Account[asset_id, account]` |
| `asset_info` | planned | asset-hub | Detailed metadata for a specific asset: admin, issuer, freezer, supply, deposit, status |
| `asset_transfer` | planned [write] | asset-hub | Transfer an asset. Builds `Assets.transfer(id, target, amount)` |

### Foreign assets

Tokens originating from other chains, received via XCM. Tracked in the `ForeignAssets` pallet.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `foreign_assets_list` | planned | asset-hub | List foreign assets with metadata and multilocation origin |
| `foreign_asset_balance` | planned | asset-hub | Balance of a foreign asset for an address |

### NFTs

Non-fungible tokens via the `Nfts` pallet on Asset Hub.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `nft_collections` | planned | asset-hub | List NFT collections: ID, owner, item count, metadata |
| `nft_items` | planned | asset-hub | List items in a collection with owner and metadata |
| `nfts_owned` | planned | asset-hub | List all NFTs owned by an address across collections |
| `nft_transfer` | planned [write] | asset-hub | Transfer an NFT. Builds `Nfts.transfer(collection, item, dest)` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Assets` | `Asset` | AssetId | `{ owner, issuer, admin, freezer, supply, deposit, min_balance, ... }` |
| `Assets` | `Account` | (AssetId, AccountId) | `{ balance, status, reason, extra }` |
| `Assets` | `Metadata` | AssetId | `{ deposit, name, symbol, decimals, is_frozen }` |
| `ForeignAssets` | `Asset` | MultiLocation | `{ owner, issuer, admin, freezer, supply, ... }` |
| `ForeignAssets` | `Account` | (MultiLocation, AccountId) | `{ balance, status, ... }` |
| `Nfts` | `Collection` | CollectionId | `{ owner, items, item_metadatas, attributes }` |
| `Nfts` | `Item` | (CollectionId, ItemId) | `{ owner, approvals, deposit }` |
| `Nfts` | `CollectionMetadataOf` | CollectionId | `{ deposit, data }` |
| `Nfts` | `ItemMetadataOf` | (CollectionId, ItemId) | `{ deposit, data }` |

### Common assets (Polkadot Asset Hub)
| ID | Symbol | Name |
|---|---|---|
| 1984 | USDT | Tether USD |
| 1337 | USDC | USD Coin |

---

## 13. Cross-Chain Transfers (XCM)

Teleport or reserve-transfer assets between chains. Primarily between relay and Asset Hub.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `xcm_transfer` | planned [write] | any | Transfer native tokens between relay ↔ parachains. Builds `XcmPallet.limited_teleport_assets()` or `PolkadotXcm.limited_reserve_transfer_assets()` |
| `xcm_fee_estimate` | planned | any | Estimate fees for a cross-chain transfer without submitting |

### Common flows
| From | To | Method |
|---|---|---|
| Relay → Asset Hub | DOT teleport | `XcmPallet.limited_teleport_assets` on relay |
| Asset Hub → Relay | DOT teleport | `PolkadotXcm.limited_teleport_assets` on Asset Hub |
| Relay → People | DOT teleport | `XcmPallet.limited_teleport_assets` on relay |
| Asset Hub → Asset Hub (other network) | Bridge transfer | Via Bridge Hub |

---

## 14. Parachains (Relay Chain)

Information about registered parachains and their status.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `parachains_list` | planned | relay | List all registered parachains with their IDs, lifecycle status (onboarding/active/offboarding), and head data hash |
| `parachain_info` | planned | relay | Details for a specific parachain by ID: lifecycle, head data, current code hash, upgrade schedule |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Paras` | `Parachains` | — | `Vec<ParaId>` |
| `Paras` | `ParaLifecycles` | ParaId | `Onboarding`, `Parathread`, `Parachain`, `UpgradingParathread`, `DowngradingParachain`, `OffboardingParathread`, `OffboardingParachain` |
| `Paras` | `Heads` | ParaId | HeadData |
| `Paras` | `CurrentCodeHash` | ParaId | ValidationCodeHash |
| `Paras` | `FutureCodeUpgrades` | ParaId | BlockNumber |
| `Registrar` | `Paras` | ParaId | `{ manager, deposit, locked }` |

---

## 15. Coretime (Coretime Chain)

Coretime (blockspace) purchases and management for parachains.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `coretime_status` | planned | coretime | Current sale status: price, cores available, region begin/end, sale phase |
| `coretime_regions` | planned | coretime | List owned coretime regions for an address |
| `coretime_purchase` | planned [write] | coretime | Purchase bulk coretime. Builds `Broker.purchase(price_limit)` |
| `coretime_renew` | planned [write] | coretime | Renew existing coretime. Builds `Broker.renew(core)` |
| `coretime_on_demand` | planned [write] | relay | Place on-demand coretime order. Builds `OnDemandAssignmentProvider.place_order_allow_death(max_amount, para_id)` |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Broker` | `Status` | — | `{ core_count, private_pool_size, system_pool_size, last_committed_timeslice, last_timeslice }` |
| `Broker` | `SaleInfo` | — | `{ sale_start, leadin_length, price, region_begin, region_end, ... }` |
| `Broker` | `Regions` | RegionId | `{ end, owner, paid }` |
| `Broker` | `Configuration` | — | `{ advance_notice, interlude_length, leadin_length, region_length, ... }` |

---

## 16. Asset Conversion (Asset Hub DEX)

On-chain DEX on Asset Hub via the `AssetConversion` pallet. Enables swaps between native token and registered assets.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `swap_pools` | planned | asset-hub | List liquidity pools with reserves and trading pairs. Answers: "what can I swap DOT for on Asset Hub?" |
| `swap_quote` | planned | asset-hub | Get a quote for swapping amount of asset A → asset B. Shows expected output and price impact |
| `swap` | planned [write] | asset-hub | Execute a swap. Builds `AssetConversion.swap_exact_tokens_for_tokens()` or `swap_tokens_for_exact_tokens()`. Dry-runs first |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `AssetConversion` | `Pools` | (Asset1, Asset2) | `{ lp_token }` |
| `AssetConversion` | `NextPoolAssetId` | — | AssetId |

---

## 17. Staking Economics (Relay Chain)

Network-level staking statistics and economics.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `era_info` | planned | relay | Current era, session index, session progress, time until next era, era duration. Essential context for staking |
| `staking_rate` | planned | relay | Total staked vs total issuance, ideal staking rate, current inflation rate, validator count |
| `track_info` | planned | relay | Governance track parameters: decision period, confirmation period, min approval curve, min support curve, max deciding. Differs per network |

### Storage reference
| Pallet | Entry | Key | Returns |
|---|---|---|---|
| `Staking` | `CurrentEra` | — | EraIndex |
| `Staking` | `ActiveEra` | — | `{ index, start }` |
| `Staking` | `ErasTotalStake` | EraIndex | Balance |
| `Staking` | `CounterForValidators` | — | u32 |
| `Staking` | `CounterForNominators` | — | u32 |
| `Session` | `CurrentIndex` | — | SessionIndex |
| `Balances` | `TotalIssuance` | — | Balance |

---

## 18. Utility / Batch

Combine multiple calls into one transaction. Used internally by other tools but also exposed directly.

| Tool | Status | Chain | Description |
|---|---|---|---|
| `batch_calls` | planned [write] | any | Batch multiple extrinsics into one transaction. Builds `Utility.batch_all(calls)`. Useful for "claim all rewards" or "unlock all locks" |

---

## Implementation Priority

Based on what users ask most about and what provides the most value:

### Phase 1 — Foundation + Metadata (current)
- `chain_info` ✅
- `get_balances` ✅
- **Address utilities** — offline, no chain needed: `ss58_decode`, `ss58_encode`, `ss58_convert`, `ss58_validate`
- **Metadata introspection** — high-leverage, lets the agent self-serve on any chain:
  - `list_pallets`, `pallet_info`
  - `list_calls`, `call_info`
  - `list_storage`, `storage_info`
  - `list_constants`, `constant_value`
  - `list_events`, `list_errors`
- `query_storage`, `decode_call`, `encode_call`
- `extrinsic_info`, `fee_estimate`, `dry_run`

### Phase 2 — Read-heavy user flows
- `staking_status`, `nomination_pools`, `pool_info`, `validator_detail`, `era_info`
- `referenda_active`, `referendum_detail`, `my_votes`, `preimage_info`, `track_info`
- `fellowship_status`, `fellowship_members`
- `identity_of`
- `account_locks`, `vesting_info`
- `proxy_list`
- `type_info`, `runtime_apis`

### Phase 3 — More reads + indexer integration
- `staking_rewards`, `account_transfers` (Subscan)
- `staking_rate`
- `treasury_info`, `treasury_spends`, `bounties_list`
- `assets_list`, `asset_balance`, `foreign_assets_list`, `foreign_asset_balance`
- `nft_collections`, `nft_items`, `nfts_owned`
- `swap_pools`, `swap_quote`
- `coretime_status`, `coretime_regions`
- `parachains_list`, `parachain_info`
- `pools_list`, `validators_info`
- `delegation_status`
- `fellowship_salary`, `fellowship_demotion_risk`
- `scheduled_actions`

### Phase 4 — Write transactions
- `vote` (Standard + Split + SplitAbstain), `delegate`, `undelegate`, `unlock_votes`
- `transfer`, `vest`, `remark`
- `bond`, `nominate`, `unbond`, `chill`, `claim_staking_rewards`
- `pool_join`, `pool_claim_payout`, `pool_unbond`, `pool_create`
- `fellowship_claim_salary`
- `set_identity`, `request_judgement`
- `xcm_transfer`
- `swap`
- `batch_calls`

### Phase 5 — Advanced
- `add_proxy`, `remove_proxy`
- `asset_transfer`, `nft_transfer`
- `coretime_purchase`, `coretime_renew`, `coretime_on_demand`
- `multisig_info`, `create_multisig_call`
- `block_info`, `bounty_detail`
- `xcm_fee_estimate`
