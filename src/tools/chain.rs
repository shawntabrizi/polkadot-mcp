// Chain utility tools: chain_info, block_info, decode_extrinsic, query_storage.
//
// These are general-purpose tools that work on any Substrate chain.
// `query_storage` is the generic escape hatch for advanced users.

// TODO: Implement tools
//
// #[tool(description = "Get chain information: runtime version, current block, \
//     current era, token info. Defaults to relay chain.")]
// async fn chain_info(&self, params: ChainInfoParams) -> Result<CallToolResult>
//
// #[tool(description = "Get block details with decoded extrinsics and events. \
//     Specify block by number or hash.")]
// async fn block_info(&self, params: BlockInfoParams) -> Result<CallToolResult>
//
// #[tool(description = "Decode a hex-encoded extrinsic into human-readable format. \
//     Uses chain metadata for decoding.")]
// async fn decode_extrinsic(&self, params: DecodeParams) -> Result<CallToolResult>
//
// #[tool(description = "Query any storage item on any chain. Generic escape hatch \
//     for advanced queries. Requires chain name, pallet name, storage entry name, \
//     and optional key(s).")]
// async fn query_storage(&self, params: QueryStorageParams) -> Result<CallToolResult>
