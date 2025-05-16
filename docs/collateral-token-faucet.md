# Collateral Token Faucet

The Collateral Token Faucet is a utility program that provides test tokens for the Signals Breakout Programs prediction markets. This document explains how to use the faucet program for development and testing.

## Overview

The Collateral Token Faucet program allows users to mint test collateral tokens that can be used for betting in prediction markets during development and testing. The program simplifies testing by providing an easy way to obtain tokens without needing to set up external token sources.

## Key Features

- Mint test collateral tokens on demand
- Automatic token account creation if not exists
- Simple API for integration with testing scripts
- PDA-based authority for secure token minting

## Program Architecture

### Program ID

```
DDFXv1hETR8pQSpNbzCxTX7jm1Hr57V4oihDGosXQfgC
```

### Accounts

1. **Faucet PDA**:

   - PDA derived from the seed "collateral_faucet"
   - Acts as the mint authority for the collateral token

2. **Collateral Mint**:

   - The token mint that represents the collateral token
   - Must have the Faucet PDA set as mint authority

3. **Receiver**:
   - The token account that will receive the newly minted tokens
   - Created automatically if it doesn't exist

## Instructions

### `initialize`

Initializes the faucet program.

```typescript
initialize(): Promise<void>
```

This instruction is primarily used for deployment and does not require any parameters.

### `mintCollateralTokens`

Mints collateral tokens to a specified receiver.

```typescript
mintCollateralTokens(amount: number | BN): Promise<void>
```

Parameters:

- `amount`: The amount of tokens to mint (in raw units, accounting for decimals)

Required accounts:

- `mint`: The collateral token mint account
- `faucet_pda`: The PDA that acts as mint authority
- `receiver`: The token account to receive the minted tokens
- `user`: The signer of the transaction (will pay for transaction fees)
- Standard program accounts (`token_program`, `system_program`, etc.)

## Usage Examples

### Basic Usage

```typescript
import { Connection, PublicKey, Keypair } from "@solana/web3.js";
import { Program, Provider, BN } from "@project-serum/anchor";
import * as anchor from "@project-serum/anchor";
import idl from "./idl/collateral_token_faucet.json";

// Setup connection and provider
const connection = new Connection("http://localhost:8899", "confirmed");
const wallet = new anchor.Wallet(Keypair.generate());
const provider = new anchor.AnchorProvider(connection, wallet, {});

// Initialize program
const programId = new PublicKey("DDFXv1hETR8pQSpNbzCxTX7jm1Hr57V4oihDGosXQfgC");
const program = new Program(idl, programId, provider);

// Define mint address
const COLLATERAL_MINT = new PublicKey("YourCollateralMintAddress");

// Mint tokens (e.g., 1000 tokens with 6 decimals = 1,000,000,000 raw units)
await program.methods
  .mintCollateralTokens(new BN(1_000_000_000))
  .accounts({
    mint: COLLATERAL_MINT,
    receiver: await findAssociatedTokenAddress(
      wallet.publicKey,
      COLLATERAL_MINT
    ),
    user: wallet.publicKey,
  })
  .rpc();

console.log("Successfully minted tokens!");
```

### Integration with Tests

```typescript
import { mintTestTokens } from "./test-utils";

describe("Range Bet Program Tests", () => {
  before(async () => {
    // Mint test tokens to users before running tests
    await mintTestTokens(user1, 1000); // 1000 tokens
    await mintTestTokens(user2, 2000); // 2000 tokens
  });

  it("should allow betting on a range", async () => {
    // User has tokens for betting
    // Test betting functionality
  });
});
```

## Integration with Range Bet Program

The Collateral Token Faucet is designed to work seamlessly with the Range Bet Program:

1. **Token Flow**:

   - Users obtain test tokens from the faucet
   - These tokens are used as collateral for placing bets in prediction markets
   - When users win bets, they receive rewards in the same token

2. **Account Structure**:

   - Both programs operate on the same collateral token mint
   - The Faucet PDA holds mint authority for test environments
   - The Range Bet Program uses token accounts for vault storage and user positions

3. **Development Workflow**:
   ```
   ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
   │  Deploy Faucet  │────▶│  Mint Tokens    │────▶│  Create Market  │
   └─────────────────┘     └─────────────────┘     └─────────────────┘
                                                           │
                                                           ▼
   ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
   │ Claim Rewards   │◀────│  Close Market   │◀────│   Place Bets    │
   └─────────────────┘     └─────────────────┘     └─────────────────┘
   ```

For full documentation of the Range Bet Program, see the [Architecture Documentation](./architecture.md) and [API Reference](./api-reference.md).

## Deployment

### Local Development

Deploy to a local validator:

```bash
anchor deploy --program-name collateral_token_faucet
```

### Devnet Deployment

Deploy to Solana devnet:

```bash
solana config set --url devnet
anchor deploy --program-name collateral_token_faucet
```

## Important Notes

1. The faucet program is intended for **development and testing environments only**.
2. In production, real tokens should be acquired through legitimate channels.
3. The mint authority (Faucet PDA) should be revoked in production deployments.
4. For security, access to the faucet can be restricted in testnet/devnet environments if needed.

## Related Documentation

- [Main README](../README.md) - Overview of the entire Signals Breakout Programs project
- [System Architecture](./architecture.md) - How the faucet integrates with the overall system
- [API Reference](./api-reference.md) - Complete API details for all programs
