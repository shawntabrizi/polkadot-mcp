# Loop Build Order

Tools to implement, in order. Each iteration: implement ONE tool, add tests, commit.

Skip any tool already marked `implemented` in TOOLS.md.

## Batch 1 — Metadata listing tools (follow `pallet_info` pattern in metadata.rs)

These all use `api.metadata()` and iterate pallet metadata. No storage queries needed.
Use `decode::type_to_string()` for type rendering.

1. `list_calls` — List all calls for a pallet with param names and types. File: `src/tools/metadata.rs`
2. `call_info` — Full detail for a specific call: params with types, docs. File: `src/tools/metadata.rs`
3. `list_storage` — List storage entries with key/value types. File: `src/tools/metadata.rs`
4. `storage_info` — Full detail for a storage entry: key types, value type, docs. File: `src/tools/metadata.rs`
5. `list_events` — List events with field names. File: `src/tools/metadata.rs`
6. `list_errors` — List errors with descriptions. File: `src/tools/metadata.rs`
7. `list_constants` — List constants with values and types. File: `src/tools/metadata.rs`

## Batch 2 — Generic chain tools (new patterns)

8. `query_storage` — Read any storage item by pallet + entry + keys. Use `subxt_backend::fetch_storage`. Accept keys as JSON array of strings. File: `src/tools/metadata.rs` or `src/tools/chain.rs`
9. `decode_call` — Decode hex call data using metadata. Use `subxt::ext::scale_value::scale::decode_as_type`. File: `src/tools/chain.rs`
10. `encode_call` — Build hex call data from pallet + method + params. Use `subxt::dynamic::tx()`. File: `src/tools/chain.rs`

## Batch 3 — Phase 2 read-heavy tools (follow `get_balances` / `account_locks` pattern)

These fetch specific storage entries and format domain-specific output.

11. `era_info` — Current era, session, time to next era. Storage: `Staking.CurrentEra`, `Staking.ActiveEra`, `Session.CurrentIndex`. File: `src/tools/staking.rs`
12. `staking_status` — Bonded amount, nominations, unbonds. Storage: `Staking.Bonded`, `Staking.Ledger`, `Staking.Nominators`. File: `src/tools/staking.rs`
13. `nomination_pools` — Pool membership details. Storage: `NominationPools.PoolMembers`, `NominationPools.BondedPools`. File: `src/tools/staking.rs`
14. `pool_info` — Single pool details. Storage: `NominationPools.BondedPools`. File: `src/tools/staking.rs`
15. `vesting_info` — Vesting schedules. Storage: `Vesting.Vesting`. File: `src/tools/account.rs`
16. `proxy_list` — List proxies. Storage: `Proxy.Proxies`. File: new `src/tools/proxy.rs`
17. `identity_of` — On-chain identity. Storage: `Identity.IdentityOf`. Chain: people. File: new `src/tools/identity.rs`
18. `referenda_active` — Active referenda. Storage: iterate `Referenda.ReferendumInfoFor`. Use `subxt_backend::fetch_storage_iter`. File: `src/tools/governance.rs`
19. `referendum_detail` — Single referendum. Storage: `Referenda.ReferendumInfoFor[index]`. File: `src/tools/governance.rs`
20. `my_votes` — Votes across tracks. Storage: iterate `ConvictionVoting.VotingFor`. File: `src/tools/governance.rs`
21. `fellowship_status` — Rank + activity. Storage: `FellowshipCollective.Members`, `CoreFellowship.Member`. Chain: collectives. File: `src/tools/fellowship.rs`
22. `fellowship_members` — List members. Storage: iterate `FellowshipCollective.Members`. Chain: collectives. File: `src/tools/fellowship.rs`
23. `track_info` — Governance track params. These are constants in the runtime. File: `src/tools/governance.rs`

## Batch 4 — Phase 3 tools (more reads)

24. `treasury_info` — Treasury state. File: new `src/tools/treasury.rs`
25. `treasury_spends` — Active spends. File: `src/tools/treasury.rs`
26. `staking_rate` — Staking economics. File: `src/tools/staking.rs`
27. `assets_list` — Registered assets on Asset Hub. File: new `src/tools/assets.rs`
28. `asset_balance` — Balance of specific asset. File: `src/tools/assets.rs`
29. `parachains_list` — Registered parachains. File: new `src/tools/parachains.rs`
30. `parachain_info` — Single parachain details. File: `src/tools/parachains.rs`
31. `coretime_status` — Current sale status. File: new `src/tools/coretime.rs`
32. `pools_list` — Browse nomination pools. File: `src/tools/staking.rs`
33. `validators_info` — Active validator set. File: `src/tools/staking.rs`
34. `fellowship_salary` — Salary claimable status. File: `src/tools/fellowship.rs`
35. `fellowship_demotion_risk` — Demotion timeline. File: `src/tools/fellowship.rs`
36. `delegation_status` — Vote delegations. File: `src/tools/governance.rs`
37. `multisig_info` — Pending multisig calls. File: new `src/tools/multisig.rs`
38. `scheduled_actions` — Pending scheduler dispatches. File: `src/tools/chain.rs`

## Patterns reference

- **Metadata listing**: See `list_pallets`, `pallet_info` in `src/tools/metadata.rs`
- **Type rendering**: Use `decode::type_to_string(type_id, metadata.types())`
- **Constant decoding**: See `constant_value` in `src/tools/metadata.rs` — uses `subxt::dynamic::constant()`
- **Storage query (single)**: See `get_balances` in `src/tools/account.rs` — uses `subxt_backend::fetch_storage()`
- **Storage query (multi-field)**: See `account_locks` in `src/tools/account.rs` — queries 3 storage entries, formats each
- **Storage iteration**: Use `subxt_backend::fetch_storage_iter()` with a limit
- **Value formatting**: Use `decode::format_value()` for generic output, or domain-specific formatting
- **SS58 offline**: See `src/tools/ss58.rs` — no server/chain needed
- **Error handling**: Return `error_result()` for user errors, `?` for internal errors
- **Testing**: Unit tests in the tool file, integration tests against Westend in `src/integration_tests.rs`
