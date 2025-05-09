import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Market Close", () => {
  let env: TestEnv;

  // Setup test environment only once (shared across all tests)
  before(async () => {
    env = await setupTestEnvironment();

    // Check last closed market ID from program state
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const lastClosedMarketId = programState.lastClosedMarket
      ? programState.lastClosedMarket.toNumber()
      : -1;

    // Check current market status
    let needNewMarket = false;

    try {
      const marketInfo = await env.program.account.market.fetch(env.market);

      // If current market is closed or inactive, need new market
      if (marketInfo.closed || !marketInfo.active) {
        needNewMarket = true;
      }
    } catch (e) {
      // Cannot load current market info, need new market
      needNewMarket = true;
    }

    // Create new market (if needed)
    if (needNewMarket) {
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
    }

    // Setup test bets
    await setupTestBets();
  });

  // Check if market status is correct before each test
  beforeEach(async () => {
    // Check if market is active and not closed
    try {
      const marketInfo = await env.program.account.market.fetch(env.market);

      // If market is already closed or inactive, create new market
      if (marketInfo.closed || !marketInfo.active) {
        const newMarket = await env.createNewMarket();
        env.market = newMarket.market;
        env.marketId = newMarket.marketId;

        // Setup basic bets for new market
        await setupTestBets();
      }
    } catch (e) {
      // If market doesn't exist, create new one
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;

      // Setup basic bets for new market
      await setupTestBets();
    }
  });

  // Function to setup basic bets for testing
  async function setupTestBets() {
    try {
      // Check if market is open
      const marketInfo = await env.program.account.market.fetch(env.market);
      if (marketInfo.closed) {
        return; // Don't bet on closed markets
      }

      // user1 bets on bin 0
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [new BN(100_000_000_000)], // 100 tokens
          new BN(150_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // user2 bets on bins 0 and 1(60)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0, 1],
          [new BN(50_000_000_000), new BN(100_000_000_000)], // 50, 100 tokens
          new BN(200_000_000_000)
        )
        .accounts({
          user: env.user2.publicKey,
          userTokenAccount: env.userTokenAccounts.user2,
          vault: env.vault,
        })
        .signers([env.user2])
        .rpc();

      // user3 bets on bin -1 (-60)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [Math.ceil(Math.abs(-60 / env.tickSpacing))], // Calculate tickIndex (convert to absolute value)
          [new BN(150_000_000_000)], // 150 tokens
          new BN(200_000_000_000)
        )
        .accounts({
          user: env.user3.publicKey,
          userTokenAccount: env.userTokenAccounts.user3,
          vault: env.vault,
        })
        .signers([env.user3])
        .rpc();
    } catch (error) {
      throw error; // Rethrow error to fail the test
    }
  }

  it("Admin should be able to close the market and set winning bin", async () => {
    // Check if market is already closed
    const marketInfo = await env.program.account.market.fetch(env.market);
    if (marketInfo.closed) {
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
      await setupTestBets();
    }

    // Close markets sequentially (up to current market ID)
    await env.closeMarketsSequentially(env.marketId, 0);

    // Check market information
    const updatedMarketInfo = await env.program.account.market.fetch(
      env.market
    );
    expect(updatedMarketInfo.closed).to.be.true; // closed = true
    expect(updatedMarketInfo.winningBin).to.not.be.null;
    expect(updatedMarketInfo.winningBin.toString()).to.equal("0"); // winningBin = 0

    // Try to buy tokens in closed market
    try {
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [new BN(100_000_000_000)],
          new BN(150_000_000_000)
        )
        .accounts({
          user: env.user4.publicKey,
          userTokenAccount: env.userTokenAccounts.user4,
          vault: env.vault,
        })
        .signers([env.user4])
        .rpc();

      expect.fail("Token purchase in closed market should fail");
    } catch (e) {
      expect(e.toString()).to.include("Market is closed");
    }
  });

  it("Invalid winning bin should fail market close", async () => {
    // Create new market
    const newMarket = await env.createNewMarket();
    env.market = newMarket.market;
    env.marketId = newMarket.marketId;
    await setupTestBets();

    // Try to close market with out of range winning bin
    const outOfRangeIndex = Math.floor(env.maxTick / env.tickSpacing) + 10; // Ensure out of range

    // Close all markets up to the previous one
    await env.closeMarketsSequentially(env.marketId - 1, 0);

    try {
      // Try to close current market directly (with invalid value)
      await env.program.methods
        .closeMarket(new BN(env.marketId), outOfRangeIndex)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("Market close with out of range winning bin should fail");
    } catch (e) {
      expect(e.toString()).to.include("BinIndexOutOfRange");
    }
  });

  it("Cannot close already closed market", async () => {
    // Create new market
    const newMarket = await env.createNewMarket();
    env.market = newMarket.market;
    env.marketId = newMarket.marketId;
    await setupTestBets();

    // Close markets sequentially
    await env.closeMarketsSequentially(env.marketId, 0);

    // Try to close again
    try {
      await env.program.methods
        .closeMarket(new BN(env.marketId), 1) // Try to close with different winning bin
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("Already closed market cannot be closed again");
    } catch (e) {
      expect(e.toString()).to.include("Market is closed");
    }
  });

  it("Last closed market ID should be tracked correctly", async () => {
    // Create new market
    const newMarket = await env.createNewMarket();
    env.market = newMarket.market;
    env.marketId = newMarket.marketId;
    await setupTestBets();

    // Initial value check
    const initialState = await env.program.account.programState.fetch(
      env.programState
    );
    const initialLastClosed = initialState.lastClosedMarket
      ? initialState.lastClosedMarket.toNumber()
      : -1;

    // Close markets sequentially
    await env.closeMarketsSequentially(env.marketId, 0);

    // Updated value check
    const updatedState = await env.program.account.programState.fetch(
      env.programState
    );
    expect(updatedState.lastClosedMarket).to.not.be.null;
    expect(updatedState.lastClosedMarket.toString()).to.equal(
      env.marketId.toString()
    );

    // Check if updated value is greater than initial value
    if (initialLastClosed >= 0) {
      expect(updatedState.lastClosedMarket.toNumber()).to.be.greaterThan(
        initialLastClosed
      );
    }
  });

  it("Multiple markets should be able to close sequentially", async () => {
    // Existing market creation or new market creation
    const marketInfo = await env.program.account.market.fetch(env.market);
    if (marketInfo.closed) {
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
      await setupTestBets();
    }

    // Add new market
    const { market: newMarket, marketId: newMarketId } =
      await env.createNewMarket();

    // Close first market
    await env.closeMarketsSequentially(env.marketId, 0);

    // Check first market closed status
    const firstMarketInfo = await env.program.account.market.fetch(env.market);
    expect(firstMarketInfo.closed).to.be.true;

    // Close second market
    await env.closeMarketsSequentially(newMarketId, 1);

    // Check second market closed status
    const secondMarketInfo = await env.program.account.market.fetch(newMarket);
    expect(secondMarketInfo.closed).to.be.true;
    expect(secondMarketInfo.winningBin.toString()).to.equal("1");

    // Check program state last_closed_market
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    expect(programState.lastClosedMarket.toString()).to.equal(
      newMarketId.toString()
    );
  });
});
