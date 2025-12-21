# Reclaim Protocol - Stellar SDK Onchain Integration

A Stellar (Soroban) smart contract implementation for Reclaim Protocol that enables on-chain verification of cryptographic proofs using witness-based epochs.

## Overview

This contract provides a decentralized verification system for Reclaim Protocol proofs on the Stellar network. It manages epochs with configurable witnesses and verifies cryptographic signatures using secp256k1 recovery.

### Key Features

- **Epoch Management**: Create and manage epochs with configurable witnesses
- **Proof Verification**: Verify cryptographic proofs using secp256k1 signature recovery
- **Owner Controls**: Admin-only functions for epoch management
- **Witness System**: Support for multiple witnesses with minimum witness requirements

## Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Stellar CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli)
- A Stellar account with testnet access (for deployment)

## Installation

1. Clone the repository:

```bash
git clone https://github.com/reclaimprotocol/stellar-sdk-onchain-integration.git
cd stellar-sdk-onchain-integration
```

2. Build the contract:

```bash
stellar contract build
```

This will compile the contract and generate the WASM file at `target/wasm32-unknown-unknown/release/reclaim.wasm`.

## Deployment

### Testnet Deployment

1. Deploy the contract to Stellar testnet:

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/reclaim.wasm \
  --source reclaim \
  --network testnet
```

2. Save the contract address from the output:

```bash
export CONTRACT=<paste-contract-address-here>
```

3. Set your account address:

```bash
export ACCOUNT=<paste-your-public-address-here>
```

4. Initialize the contract:

```bash
stellar contract invoke \
  --id $CONTRACT \
  --source reclaim \
  --network testnet \
  -- instantiate \
  --user $ACCOUNT
```

Note: On Mainnet (Public), you might need to set some suitable transaction fee (`--fee 600000`).

### Deployed Contracts

**Mainnet:**
- Contract Address: `CD4M2KHW3ESOV3RUT7KCTC6BX37PIL2Z3BEK47IA74KIMFIFUI3JJDMO`
- Explorer: [View on Stellar Expert](https://steexp.com/contract/CD4M2KHW3ESOV3RUT7KCTC6BX37PIL2Z3BEK47IA74KIMFIFUI3JJDMO)

**Testnet:**
- Contract Address: `CA3EMXR6JOOTNP44T3OAJFMMMGKRRETDJKBLZP2RU3SIY4SDFAH54DU5`
- Explorer: [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CA3EMXR6JOOTNP44T3OAJFMMMGKRRETDJKBLZP2RU3SIY4SDFAH54DU5)

## Contract Functions

### `instantiate(user: Address)`

Initializes the contract with the specified owner address. This function:
- Sets the contract owner
- Initializes the first epoch (epoch 0) with default witness
- Can only be called once

**Parameters:**
- `user`: The address that will become the contract owner

### `add_epoch(witnesses: Vec<Witness>, minimum_witness: u32)`

Adds a new epoch with the specified witnesses. Only the contract owner can call this function.

**Parameters:**
- `witnesses`: A vector of `Witness` structs containing address and host information
- `minimum_witness`: The minimum number of witnesses required for verification

**Witness Structure:**
```rust
pub struct Witness {
    pub address: BytesN<20>,  // 20-byte witness address
    pub host: String,          // Host identifier
}
```

### `verify_proof(message_digest: BytesN<32>, signature: BytesN<64>, recovery_id: u32)`

Verifies a cryptographic proof by:
1. Recovering the public key from the signature using secp256k1
2. Deriving the Ethereum-style address from the public key
3. Checking if the address matches any of the current epoch's witnesses

**Parameters:**
- `message_digest`: 32-byte hash of the message
- `signature`: 64-byte secp256k1 signature
- `recovery_id`: Recovery ID for signature recovery (0-3)

**Returns:**
- `Ok(())` if verification succeeds
- `Err(ReclaimError::SignatureMismatch)` if the recovered address doesn't match any witness

## Error Codes

The contract defines the following error types:

- `OnlyOwner` (1): Function can only be called by the contract owner
- `AlreadyInitialized` (2): Contract has already been initialized
- `HashMismatch` (3): Hash verification failed
- `LengthMismatch` (4): Length validation failed
- `SignatureMismatch` (5): Signature verification failed

## Testing

Run the test suite:

```bash
cd contracts/reclaim
cargo test
```

The test suite includes:
- Contract initialization tests
- Epoch addition tests
- Proof verification tests

## Project Structure

```
stellar-sdk-onchain-integration/
├── contracts/
│   └── reclaim/
│       ├── src/
│       │   ├── lib.rs          # Main contract implementation
│       │   └── test.rs         # Test suite
│       ├── Cargo.toml          # Contract dependencies
│       └── test_snapshots/      # Test snapshots
├── Cargo.toml                  # Workspace configuration
└── README.md                   # This file
```

## Development

### Building for Release

The contract is configured with optimized release settings:

- Optimization level: `z` (size optimization)
- Overflow checks: enabled
- LTO: enabled
- Codegen units: 1

### Contract Types

**Config:**
```rust
pub struct Config {
    pub owner: Address,
    pub current_epoch: u128,
    pub exists: bool,
}
```

**Epoch:**
```rust
pub struct Epoch {
    pub id: u128,
    pub timestamp_start: u64,
    pub timestamp_end: u64,
    pub minimum_witness: u32,
    pub witnesses: Vec<Witness>,
}
```


