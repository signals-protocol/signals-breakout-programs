import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Keypair, Connection } from "@solana/web3.js";
import { RangeBetProgram } from "../target/types/range_bet_program";
import { BN } from "bn.js";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  createAccount,
} from "@solana/spl-token";

export interface TestEnv {
  provider: anchor.AnchorProvider;
  program: Program<RangeBetProgram>;
  admin: any;
  user1: Keypair;
  user2: Keypair;
  user3: Keypair;
  user4: Keypair;
  user5: Keypair;
  programState: anchor.web3.PublicKey;
  market: anchor.web3.PublicKey;
  collateralMint: anchor.web3.PublicKey;
  vault: anchor.web3.PublicKey;
  vaultAuthority: anchor.web3.PublicKey;
  vaultAuthorityBump: number;
  marketId: number;
  tickSpacing: number;
  minTick: number;
  maxTick: number;
  closeTime: typeof BN.prototype;
  getUserPosition: (
    user: Keypair,
    marketId: number
  ) => Promise<anchor.web3.PublicKey>;
  userTokenAccounts: {
    admin: anchor.web3.PublicKey;
    user1: anchor.web3.PublicKey;
    user2: anchor.web3.PublicKey;
    user3: anchor.web3.PublicKey;
    user4: anchor.web3.PublicKey;
    user5: anchor.web3.PublicKey;
  };
  // New feature: Market reset and efficient test environment management
  resetMarket: () => Promise<void>;
  createNewMarket: (params?: {
    tickSpacing?: number;
    minTick?: number;
    maxTick?: number;
    closeTime?: number;
  }) => Promise<{
    market: anchor.web3.PublicKey;
    marketId: number;
  }>;
  replenishTokens: (user: Keypair, amount?: number) => Promise<void>;
  closeMarketsSequentially: (
    targetMarketId: number,
    winningBin: number
  ) => Promise<void>;
}

/**
 * Complete test environment setup - efficient to call only once
 */
export async function setupTestEnvironment(): Promise<TestEnv> {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RangeBetProgram as Program<RangeBetProgram>;

  // Admin (program owner) keypair
  const admin = provider.wallet;

  // Create test users
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();
  const user4 = Keypair.generate();
  const user5 = Keypair.generate();

  // Airdrop SOL to test users (generous 10 SOL)
  for (const user of [user1, user2, user3, user4, user5]) {
    const airdropSig = await provider.connection.requestAirdrop(
      user.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
  }

  // Program state account
  const [programState, programStateBump] =
    await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("range-bet-state")],
      program.programId
    );

  // Initialize program - if not already initialized
  try {
    await program.account.programState.fetch(programState);
  } catch (e) {
    // Initialize program state if it doesn't exist
    await program.methods
      .initializeProgram()
      .accounts({
        initializer: admin.publicKey,
      })
      .rpc();
  }

  // Create collateral token Mint
  const collateralMint = await createMint(
    provider.connection,
    admin.payer,
    admin.publicKey,
    null,
    9 // 9 decimals
  );

  // Create token accounts (ATA) for each user
  const adminTokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    admin.publicKey
  );

  const user1TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user1.publicKey
  );

  const user2TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user2.publicKey
  );

  const user3TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user3.publicKey
  );

  const user4TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user4.publicKey
  );

  const user5TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user5.publicKey
  );

  // Mint tokens to each user
  const mintAmount = 10000_000_000_000; // 10,000 tokens (generous amount)

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    adminTokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user1TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user2TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user3TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user4TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user5TokenAccount,
    admin.publicKey,
    mintAmount
  );

  // Market creation required basic values
  const tickSpacing = 60;
  const minTick = -360;
  const maxTick = 360;
  const closeTime = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60; // 1 week later

  // Calculate vault authority PDA for the first market ID (0)
  let marketId = 0;
  const [vaultAuthority, vaultAuthorityBump] =
    await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), new BN(marketId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

  // Create token account owned by the PDA (admin pays)
  // Here, only the account is created, and funds are filled by the user through buyTokens
  const vault = await createAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    vaultAuthority, // PDA is owner
    Keypair.generate() // Create new account keypair
  );

  // Market account address (PDA) calculation
  const [market, marketBump] =
    await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(marketId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

  // Market creation (first time only)
  async function createMarketIfNeeded() {
    try {
      // Check if market already exists
      await program.account.market.fetch(market);
      console.log("✅ Market ID", marketId, "already exists.");
      return false;
    } catch (e) {
      // Create market if it doesn't exist
      console.log("🔨 Creating Market ID", marketId, "...");
      await program.methods
        .createMarket(
          tickSpacing,
          new BN(minTick),
          new BN(maxTick),
          new BN(closeTime)
        )
        .accounts({
          owner: admin.publicKey,
          collateralMint: collateralMint,
        })
        .rpc();
      console.log("✅ Market ID", marketId, "created!");
      return true;
    }
  }

  await createMarketIfNeeded();

  // User position account address (PDA) calculation
  async function getUserPosition(user: Keypair, marketId: number) {
    const [userPosition, userPositionBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("pos"),
          user.publicKey.toBuffer(),
          new BN(marketId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    return userPosition;
  }

  // Market reset function (create new market to provide clean test environment)
  async function resetMarketInternal() {
    try {
      // Create new market without closing the existing one
      // This approach bypasses the program's market closing order constraint
      console.log("🔄 Creating new test market...");
      const {
        market: newMarket,
        marketId: newMarketId,
        vault: newVault,
        vaultAuthority: newVaultAuthority,
        vaultAuthorityBump: newVaultAuthorityBump,
      } = await createNewMarket();

      // Store new values for the object to be returned
      updatedMarket = newMarket;
      updatedMarketId = newMarketId;
      updatedVault = newVault;
      updatedVaultAuthority = newVaultAuthority;
      updatedVaultAuthorityBump = newVaultAuthorityBump;
      console.log("✅ New market ID", newMarketId, "created (for testing)");
    } catch (e) {
      console.log("⚠️ Error occurred during new market creation:", e.message);
    }
  }

  // Temporary variables for market update
  let updatedMarket = market;
  let updatedMarketId = marketId;
  let updatedVault = vault;
  let updatedVaultAuthority = vaultAuthority;
  let updatedVaultAuthorityBump = vaultAuthorityBump;

  // New market creation function
  async function createNewMarket(params?: {
    tickSpacing?: number;
    minTick?: number;
    maxTick?: number;
    closeTime?: number;
  }) {
    // Get current market count from program state
    const state = await program.account.programState.fetch(programState);
    const newMarketId = state.marketCount.toNumber();

    // Calculate new market account address
    const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // Calculate vault authority for new market
    const [newVaultAuthority, newVaultAuthorityBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("vault"),
          new BN(newMarketId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    // Create token vault for new market
    const newVault = await createAccount(
      provider.connection,
      admin.payer,
      collateralMint,
      newVaultAuthority,
      Keypair.generate()
    );

    console.log(
      `🏦 Setting Vault account for new market ID ${newMarketId}:`,
      newVault.toString()
    );

    // Create new market
    console.log("🔨 Creating new market ID", newMarketId, "...");
    await program.methods
      .createMarket(
        params?.tickSpacing ?? tickSpacing,
        new BN(params?.minTick ?? minTick),
        new BN(params?.maxTick ?? maxTick),
        new BN(params?.closeTime ?? closeTime)
      )
      .accounts({
        owner: admin.publicKey,
        collateralMint: collateralMint,
      })
      .rpc();
    console.log("✅ New market ID", newMarketId, "created!");

    // Store updated values
    updatedVault = newVault;
    updatedVaultAuthority = newVaultAuthority;
    updatedVaultAuthorityBump = newVaultAuthorityBump;

    return {
      market: newMarket,
      marketId: newMarketId,
      vault: newVault,
      vaultAuthority: newVaultAuthority,
      vaultAuthorityBump: newVaultAuthorityBump,
    };
  }

  // Token replenishment function
  async function replenishTokens(user: Keypair, amount = mintAmount) {
    // Find token account
    let tokenAccount;
    if (user.publicKey.equals(admin.publicKey)) {
      tokenAccount = adminTokenAccount;
    } else if (user.publicKey.equals(user1.publicKey)) {
      tokenAccount = user1TokenAccount;
    } else if (user.publicKey.equals(user2.publicKey)) {
      tokenAccount = user2TokenAccount;
    } else if (user.publicKey.equals(user3.publicKey)) {
      tokenAccount = user3TokenAccount;
    } else if (user.publicKey.equals(user4.publicKey)) {
      tokenAccount = user4TokenAccount;
    } else if (user.publicKey.equals(user5.publicKey)) {
      tokenAccount = user5TokenAccount;
    } else {
      throw new Error("Unknown user");
    }

    // Mint tokens
    await mintTo(
      provider.connection,
      admin.payer,
      collateralMint,
      tokenAccount,
      admin.publicKey,
      amount
    );
  }

  console.log("🎉 Complete test environment setup!");

  // Test environment object configuration
  const testEnv: TestEnv = {
    provider,
    program,
    admin,
    user1,
    user2,
    user3,
    user4,
    user5,
    programState,
    market: updatedMarket,
    collateralMint,
    vault: updatedVault,
    vaultAuthority: updatedVaultAuthority,
    vaultAuthorityBump: updatedVaultAuthorityBump,
    marketId: updatedMarketId,
    tickSpacing,
    minTick,
    maxTick,
    closeTime: new BN(closeTime),
    getUserPosition,
    userTokenAccounts: {
      admin: adminTokenAccount,
      user1: user1TokenAccount,
      user2: user2TokenAccount,
      user3: user3TokenAccount,
      user4: user4TokenAccount,
      user5: user5TokenAccount,
    },
    resetMarket: async () => {
      await resetMarketInternal();
      // Update object properties in resetMarket
      testEnv.market = updatedMarket;
      testEnv.marketId = updatedMarketId;
      testEnv.vault = updatedVault;
      testEnv.vaultAuthority = updatedVaultAuthority;
      testEnv.vaultAuthorityBump = updatedVaultAuthorityBump;
    },
    createNewMarket,
    replenishTokens,
    closeMarketsSequentially: async (
      targetMarketId: number,
      winningBin: number = 0
    ) => {
      // Check program state for last_closed_market value
      const state = await program.account.programState.fetch(programState);
      let lastClosed = state.lastClosedMarket
        ? state.lastClosedMarket.toNumber()
        : -1;

      // Markets must close sequentially, so close from last_closed_market+1 to target market
      for (let id = lastClosed + 1; id <= targetMarketId; id++) {
        try {
          // Calculate market account address
          const [marketToClose] =
            await anchor.web3.PublicKey.findProgramAddressSync(
              [Buffer.from("market"), new BN(id).toArrayLike(Buffer, "le", 8)],
              program.programId
            );

          console.log(`Closing market ID ${id}...`);

          // Check market information
          try {
            const marketInfo = await program.account.market.fetch(
              marketToClose
            );

            // Skip already closed markets
            if (marketInfo.closed) {
              console.log(`Market ID ${id} is already closed.`);
              continue;
            }

            // Close market
            const closeBin = id === targetMarketId ? winningBin : 0; // Only target market is specified with winning bin
            await program.methods
              .closeMarket(new BN(id), closeBin)
              .accounts({
                owner: admin.publicKey,
              })
              .rpc();

            console.log(`Market ID ${id} closed successfully.`);
          } catch (e) {
            // If market doesn't exist, show warning and continue
            console.log(
              `Market ID ${id} doesn't exist or error occurred during processing: ${e.message}`
            );
          }
        } catch (e) {
          console.error(`Failed to close market ID ${id}: ${e.message}`);
          throw e;
        }
      }
    },
  };

  return testEnv;
}
