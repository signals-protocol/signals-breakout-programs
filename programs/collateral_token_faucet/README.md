# Collateral Token Faucet

A utility program for the Signals Breakout Programs that provides test collateral tokens for development and testing environments.

## Overview

This program implements a simple faucet that can mint collateral tokens to any user. It uses a PDA (Program Derived Address) as the mint authority and allows users to request tokens with a single instruction.

## Technical Implementation

### Program Structure

```
collateral_token_faucet/
├── Cargo.toml         # Dependencies and program configuration
├── Xargo.toml         # Rust cross-compilation settings
└── src/
    └── lib.rs         # Program logic
```

### Core Components

1. **PDA Mint Authority**

   - The program uses a PDA derived from the seed "collateral_faucet" as the mint authority
   - This PDA signs for token minting operations using CPI (Cross-Program Invocation)

2. **Token Account Creation**

   - The program supports automatic creation of token accounts if they don't exist
   - Uses the Associated Token Program for standardized token account derivation

3. **Instruction Processing**
   - Simple validation of input parameters
   - Proper error handling for edge cases
   - Safe arithmetic operations

## Instructions

### initialize

Simple initialization function that logs the program ID. This instruction doesn't modify any state and serves primarily as a deployment check.

### mintCollateralTokens

The main instruction that mints tokens to a specified recipient:

```rust
pub fn mint_collateral_tokens(ctx: Context<MintCollateralTokens>, amount: u64) -> Result<()> {
    // Log information
    msg!("Minting {} tokens to receiver account.", amount);

    // Create CPI context with signer seeds
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.receiver.to_account_info(),
        authority: ctx.accounts.faucet_pda.to_account_info(),
    };

    let seeds = &[b"collateral_faucet".as_ref(), &[ctx.bumps.faucet_pda]];
    let signer_seeds = &[&seeds[..]];

    // Execute token mint operation
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        ),
        amount,
    )?;

    msg!("Tokens minted successfully.");
    Ok(())
}
```

## Account Validation

The `MintCollateralTokens` context performs these validations:

```rust
#[derive(Accounts)]
pub struct MintCollateralTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"collateral_faucet".as_ref()],
        bump
    )]
    /// CHECK: This PDA only serves as a signing authority and doesn't store data
    pub faucet_pda: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub receiver: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}
```

## Setup Requirements

Before using the faucet, ensure:

1. The collateral token mint has been created
2. The faucet PDA has been set as the mint authority
3. The program has been deployed to the target environment

## Usage Guidelines

- **Local Development**: Use freely for testing and development
- **Testnet/Devnet**: Restrict access or implement rate limiting if needed
- **Mainnet**: Do not deploy the faucet to production environments

## Integration with Range Bet Program

This faucet program is designed to work seamlessly with the Range Bet Program:

1. Users obtain test tokens from the faucet
2. These tokens can be used to place bets in prediction markets
3. The same token mint is used across both programs

## Security Considerations

While this is a utility program for testing, several security measures have been implemented:

1. PDA-based authority with proper seed derivation
2. Input validation to prevent misuse
3. Proper logging for audit trails
4. Safe CPI interactions with the Token Program

## Related Documentation

- [Collateral Token Faucet User Guide](../../docs/collateral-token-faucet.md)
- [API Reference](../../docs/api-reference.md#collateral-token-faucet-api)
