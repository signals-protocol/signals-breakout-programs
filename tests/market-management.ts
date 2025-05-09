import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Market Management", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  beforeEach(async () => {
    // Initialize market for each test
    await env.resetMarket();
  });

  it("Admin should be able to deactivate and reactivate the market", async () => {
    // Deactivate market
    await env.program.methods
      .activateMarket(new BN(env.marketId), false) // false = deactivate
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // Check market info
    let marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.false; // active = false

    // Try to buy tokens in deactivated market
    const userPosition = await env.getUserPosition(env.user1, env.marketId);

    try {
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0], // binIndex
          [new BN(100_000_000_000)], // 100 tokens
          new BN(150_000_000_000) // max collateral
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      expect.fail("Token purchase in deactivated market should fail");
    } catch (e) {
      expect(e.toString()).to.include("Market is not active");
    }

    // Reactivate market
    await env.program.methods
      .activateMarket(new BN(env.marketId), true) // true = activate
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // Check market info
    marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.true; // active = true

    // Now token purchase should be possible
    await env.program.methods
      .buyTokens(
        new BN(env.marketId),
        [0], // binIndex
        [new BN(100_000_000_000)], // 100 tokens
        new BN(150_000_000_000) // max collateral
      )
      .accounts({
        user: env.user1.publicKey,
        userTokenAccount: env.userTokenAccounts.user1,
        vault: env.vault,
      })
      .signers([env.user1])
      .rpc();

    // Verify purchase
    const userPositionInfo = await env.program.account.userMarketPosition.fetch(
      userPosition
    );
    expect(userPositionInfo.bins.length).to.be.greaterThan(0);
    expect(userPositionInfo.bins[0].amount.toString()).to.equal("100000000000");
  });

  it("Deactivating already deactivated market should be allowed (idempotent)", async () => {
    // First deactivation
    await env.program.methods
      .activateMarket(new BN(env.marketId), false)
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // Check market info
    let marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.false;

    // Second deactivation (repeat same command)
    await env.program.methods
      .activateMarket(new BN(env.marketId), false)
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // Check market info again
    marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.false; // still deactivated
  });

  it("Activating already activated market should be allowed (idempotent)", async () => {
    // Market is activated by default
    let marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.true;

    // Execute activation command
    await env.program.methods
      .activateMarket(new BN(env.marketId), true)
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // Check market info again
    marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.true; // still activated
  });

  it("Closed market cannot be activated/deactivated", async () => {
    // Close markets sequentially
    await env.closeMarketsSequentially(env.marketId, 0);

    // Verify closing
    const marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.closed).to.be.true;

    // Try to deactivate
    try {
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("Deactivating closed market should fail");
    } catch (e) {
      expect(e.toString()).to.include("Market is closed");
    }

    // Try to activate
    try {
      await env.program.methods
        .activateMarket(new BN(env.marketId), true)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("Activating closed market should fail");
    } catch (e) {
      expect(e.toString()).to.include("Market is closed");
    }
  });

  it("Non-admin users cannot change market state", async () => {
    // Regular user tries to deactivate
    try {
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.user1.publicKey,
        })
        .signers([env.user1])
        .rpc();

      expect.fail("Non-admin user changing market state should fail");
    } catch (e) {
      expect(e.toString()).to.include("Owner only function");
    }
  });
});
