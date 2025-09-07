# Icarus SDK Feature Implementation Roadmap

## Overview
This document outlines the implementation plan for enhancing Icarus SDK with powerful ICP features to create the most capable MCP-blockchain integration. Each feature includes implementation steps, testing requirements, and success criteria.

## Core Principles
- **Idiomatic Rust**: Clean, efficient, and maintainable code following Rust best practices
- **Developer Experience**: Simple APIs that abstract complexity
- **Test-Driven**: Comprehensive tests before moving to next feature
- **CLI Integration**: Seamless interaction through `icarus` CLI
- **MCP-First**: All features designed to enhance MCP tool capabilities

---

## Phase 1: Foundation Features ✅ Priority: IMMEDIATE

### 1.1 HTTP Outcalls Module ✅ COMPLETED
**Goal**: Enable MCP tools to fetch external data from any API

#### Implementation Tasks
- [x] Create `icarus-canister/src/http.rs` module
  - [x] Implement `get(url: &str) -> Result<String, String>`
  - [x] Implement `post_json(url: &str, body: Value) -> Result<String, String>`
  - [x] Add retry logic with exponential backoff
  - [x] Add request timeout handling (default 30s)
  - [x] Implement response size limits (2MB default)

- [x] Create helper macros in `icarus-canister/src/macros.rs`
  ```rust
  #[macro_export]
  macro_rules! http_get {
      ($url:expr) => { ... }
  }
  ```

- [x] Add to canister prelude
  - [x] Export http module in `lib.rs`
  - [x] Add to prelude for easy access

#### Testing Requirements
- [x] Unit tests for URL validation
- [x] Integration test: Fetch from httpbin.org
- [x] Test retry logic with failing endpoint
- [x] Test timeout handling
- [x] Test response size limits
- [x] Example canister using HTTP outcalls

#### CLI Integration
- [x] Add `icarus test-http <canister-id>` command
- [x] Show HTTP outcall costs in deployment info

#### Success Criteria
- Developers can fetch any HTTP API with one line of code
- Automatic retry on transient failures
- Clear error messages for debugging

---

### 1.2 Timers Module ✅ COMPLETED
**Goal**: Enable autonomous, scheduled operations in MCP tools

#### Implementation Tasks
- [x] Create `icarus-canister/src/timers.rs` module
  - [x] Implement `schedule_once(seconds: u64, task: impl FnOnce())`
  - [x] Implement `schedule_periodic(seconds: u64, task: impl Fn())`
  - [x] Add timer management (start, stop, list active)
  - [x] Create `timer_once!` and `timer_periodic!` macros

- [x] Add timer lifecycle management
  - [x] Timer registry with automatic tracking
  - [x] Cancel individual or all timers
  - [x] Max timers per canister limit (100)

#### Testing Requirements
- [x] Unit test timer scheduling
- [x] Test timer registry and management
- [x] Test maximum timer limits
- [x] Helper function for exponential backoff

#### CLI Integration
- [x] Decided not to add CLI commands (unnecessary complexity)
- [x] Timer functions available directly in canister code

#### Success Criteria
- Developers can schedule tasks with simple macro
- Timers persist across upgrades
- Clear visibility into active timers

---

## Phase 2: Chain Fusion Integration 🔗 Priority: HIGH

### 2.1 Bitcoin Integration
**Goal**: Enable MCP tools to interact with Bitcoin directly

#### Implementation Tasks
- [ ] Create `icarus-canister/src/chain_fusion/bitcoin.rs`
  - [ ] Implement `get_balance(address: String) -> Result<u64, String>`
  - [ ] Implement `get_utxos(address: String) -> Result<Vec<Utxo>, String>`
  - [ ] Implement `send_transaction(tx: Transaction) -> Result<String, String>`
  - [ ] Add P2PKH address validation
  - [ ] Add fee estimation

- [ ] Create Bitcoin types in `icarus-canister/src/chain_fusion/types.rs`
  - [ ] `BitcoinAddress` with validation
  - [ ] `Satoshi` amount type
  - [ ] `Transaction` builder

#### Testing Requirements
- [ ] Mock Bitcoin integration for tests
- [ ] Test address validation
- [ ] Test UTXO management
- [ ] Test transaction building
- [ ] Example: Bitcoin wallet canister

#### CLI Integration
- [ ] Add `icarus bitcoin balance <address>` command
- [ ] Add `icarus bitcoin send` interactive command

---

### 2.2 Ethereum Integration
**Goal**: Enable MCP tools to interact with Ethereum and EVM chains

#### Implementation Tasks
- [ ] Create `icarus-canister/src/chain_fusion/ethereum.rs`
  - [ ] Implement `get_balance(address: String) -> Result<U256, String>`
  - [ ] Implement `call_contract(address: String, data: Vec<u8>) -> Result<Vec<u8>, String>`
  - [ ] Implement `send_transaction(tx: Transaction) -> Result<String, String>`
  - [ ] Add EIP-55 address validation
  - [ ] Add gas estimation

- [ ] Add EVM RPC support
  - [ ] Support multiple EVM chains (ETH, Polygon, BSC)
  - [ ] Chain ID management
  - [ ] Nonce management

#### Testing Requirements
- [ ] Mock Ethereum integration
- [ ] Test address validation
- [ ] Test contract calls
- [ ] Test gas estimation
- [ ] Example: ERC-20 token interaction

---

### 2.3 Chain Key Tokens (ckBTC, ckETH)
**Goal**: Native handling of wrapped Bitcoin and Ethereum on ICP

#### Implementation Tasks
- [ ] Create `icarus-canister/src/chain_fusion/tokens.rs`
  - [ ] Implement ckBTC minting/burning
  - [ ] Implement ckETH minting/burning
  - [ ] Add balance tracking
  - [ ] Add transfer functions

#### Testing Requirements
- [ ] Test token minting flow
- [ ] Test token burning flow
- [ ] Test balance tracking
- [ ] Example: Cross-chain DEX canister

---

## Phase 3: Authentication 🔐 Priority: MEDIUM

### 3.1 Internet Identity Integration
**Goal**: Seamless, passwordless authentication for MCP tools

#### Implementation Tasks
- [ ] Create `icarus-canister/src/auth/identity.rs`
  - [ ] Implement `authenticate() -> Result<Principal, String>`
  - [ ] Implement `get_delegation()`
  - [ ] Add session management
  - [ ] Add principal validation

- [ ] Create auth helpers
  - [ ] `require_auth` attribute macro
  - [ ] Session storage with timeout
  - [ ] Multi-device support

#### Testing Requirements
- [ ] Mock Internet Identity for tests
- [ ] Test authentication flow
- [ ] Test session expiry
- [ ] Test delegation chain
- [ ] Example: Authenticated notes canister

#### CLI Integration
- [ ] Add `icarus auth login` command
- [ ] Add `icarus auth status` command
- [ ] Store auth tokens securely

---

## Phase 4: Privacy Features 🔒 Priority: MEDIUM

### 4.1 vetKeys Integration
**Goal**: Enable private data storage on public blockchain

#### Implementation Tasks
- [ ] Create `icarus-canister/src/privacy/vetkeys.rs`
  - [ ] Implement `encrypt(data: &[u8], owner: Principal) -> Result<Vec<u8>, String>`
  - [ ] Implement `decrypt(data: &[u8], owner: Principal) -> Result<Vec<u8>, String>`
  - [ ] Add key derivation
  - [ ] Add access control lists

- [ ] Create privacy helpers
  - [ ] `#[private]` field attribute
  - [ ] Automatic encryption/decryption
  - [ ] Shared secret support

#### Testing Requirements
- [ ] Test encryption/decryption
- [ ] Test access control
- [ ] Test key derivation
- [ ] Example: Private messaging canister

---

## Phase 5: AI Integration 🤖 Priority: FUTURE

### 5.1 LLM Canister Integration
**Goal**: On-chain AI model execution for MCP tools

#### Implementation Tasks
- [ ] Create `icarus-canister/src/ai/llm.rs`
  - [ ] Implement `complete(prompt: String) -> Result<String, String>`
  - [ ] Add model selection
  - [ ] Add token limits
  - [ ] Add streaming responses

- [ ] Create AI helpers
  - [ ] Prompt templates
  - [ ] Response parsing
  - [ ] Cost estimation

#### Testing Requirements
- [ ] Mock LLM responses
- [ ] Test prompt handling
- [ ] Test token limits
- [ ] Example: AI assistant canister

---

### 5.2 On-chain Model Execution
**Goal**: Run AI models directly in canisters

#### Implementation Tasks
- [ ] Create `icarus-canister/src/ai/models.rs`
  - [ ] Implement model loading
  - [ ] Add inference execution
  - [ ] Support ONNX format
  - [ ] Add result caching

#### Testing Requirements
- [ ] Test model loading
- [ ] Test inference accuracy
- [ ] Test performance limits
- [ ] Example: Image classifier canister

---

## Testing Strategy

### For Each Feature
1. **Unit Tests**: Test individual functions in isolation
2. **Integration Tests**: Test feature within canister context
3. **E2E Tests**: Test via CLI commands
4. **Example Canister**: Working example demonstrating feature
5. **Documentation**: API docs and usage guide

### Test Coverage Requirements
- Minimum 80% code coverage
- All error paths tested
- All edge cases covered
- Performance benchmarks included

---

## Documentation Requirements

### For Each Feature
- [ ] API documentation with examples
- [ ] Integration guide for developers
- [ ] CLI command documentation
- [ ] Example canister with comments
- [ ] Cost analysis (cycles usage)

---

## Release Checklist

### Before Moving to Next Feature
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Example canister working
- [ ] CLI commands tested
- [ ] No compiler warnings
- [ ] Code review completed
- [ ] Version bumped appropriately

---

## Success Metrics

### Developer Experience
- Feature can be used in <5 lines of code
- Clear error messages with solutions
- Comprehensive examples available
- IDE autocomplete works properly

### User Experience
- CLI commands intuitive and fast
- Operations complete in <5 seconds
- Clear progress feedback
- Helpful error recovery

### Code Quality
- No `unsafe` code without justification
- All `Result` types handled properly
- Memory usage optimized
- Idiomatic Rust patterns used

---

## Implementation Order

1. **Week 1**: HTTP Outcalls + Timers (Foundation)
2. **Week 2**: Bitcoin Integration (High impact)
3. **Week 3**: Ethereum Integration (Expand reach)
4. **Week 4**: Internet Identity (Better UX)
5. **Week 5**: vetKeys (Privacy)
6. **Future**: AI Integration (Innovation)

---

## Notes

- Each feature should be independently useful
- Maintain backwards compatibility
- Keep CLI commands consistent
- Focus on developer ergonomics
- Prioritize security and correctness