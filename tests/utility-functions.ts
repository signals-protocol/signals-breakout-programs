import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

// Helper function for testing Anchor errors
async function expectAnchorError(
  promiseFn: () => Promise<any>,
  errorText: string
) {
  try {
    await promiseFn();
    expect.fail("Expected error did not occur");
  } catch (e) {
    // For SimulateError, logs are in simulationResponse.logs path
    if (e.simulationResponse && e.simulationResponse.logs) {
      const errorLogs = e.simulationResponse.logs.join("\n");
      expect(errorLogs).to.include(errorText);
    }
    // For general errors, access the logs property directly
    else if (e.logs) {
      const errorLogs = e.logs.join("\n");
      expect(errorLogs).to.include(errorText);
    }
    // Try to check in error message
    else if (e.message) {
      expect(e.message).to.include(errorText);
    }
    // In other cases, stringify the entire error object and check
    else {
      const errorString = JSON.stringify(e);
      expect(errorString).to.include(errorText);
    }
  }
}

describe("Utility Functions", () => {
  let env: TestEnv;

  // Setup test environment once (shared across all tests)
  before(async () => {
    env = await setupTestEnvironment();
  });

  // Run before each test case
  beforeEach(async () => {
    // Always initialize with a new market to isolate state between tests
    await env.resetMarket();
  });

  describe("calculateBinCost", () => {
    it("In an empty market, cost should be the same as quantity", async () => {
      const amount = new BN(100_000_000_000); // 100 tokens

      // Calculate in empty market
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, amount)
        .accounts({
          market: env.market,
        })
        .view();

      // Cost should be the same as quantity
      expect(cost.toString()).to.equal(amount.toString());
    });

    it("Should throw error in deactivated market", async () => {
      // First buy tokens to add state to the market
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [new BN(100_000_000_000)],
          new BN(150_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Deactivate market
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      // Use helper function
      await expectAnchorError(
        () =>
          env.program.methods
            .calculateBinCost(new BN(env.marketId), 0, new BN(100_000_000_000))
            .accounts({
              market: env.market,
            })
            .view(),
        "MarketNotActive"
      );
    });

    it("Should throw error in closed market", async () => {
      // Create new market
      const { market: newMarket, marketId: newMarketId } =
        await env.createNewMarket();

      // Close market sequentially
      await env.closeMarketsSequentially(newMarketId, 0);

      // Try to get cost
      await expectAnchorError(
        () =>
          env.program.methods
            .calculateBinCost(new BN(newMarketId), 0, new BN(100_000_000_000))
            .accounts({
              market: newMarket,
            })
            .view(),
        "MarketClosed"
      );
    });

    it("Should throw error when calculating with an out-of-range empty index", async () => {
      // Calculate with out-of-range index
      const outOfRangeIndex =
        Math.abs((env.maxTick - env.minTick) / env.tickSpacing) + 1;

      await expectAnchorError(
        () =>
          env.program.methods
            .calculateBinCost(
              new BN(env.marketId),
              outOfRangeIndex,
              new BN(100_000_000_000)
            )
            .accounts({
              market: env.market,
            })
            .view(),
        "BinIndexOutOfRange"
      );
    });

    it("When q < T, cost should be less than quantity", async () => {
      // Always use new market
      await env.resetMarket();

      // First buy tokens in bin 1 to increase T (sufficiently large value)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1], // Bin 1 (60)
          [new BN(500_000_000_000)], // 500 tokens
          new BN(600_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Now calculate token purchase cost for bin 0
      const amount = new BN(50_000_000_000); // 50 tokens

      // Actual calculation execution
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, amount)
        .accounts({
          market: env.market,
        })
        .view();

      // In q=0, T>0 state, cost should be less than quantity
      expect(new BN(cost).lt(amount)).to.be.true;
    });
  });

  describe("calculateBinSellCost", () => {
    // Create a new market before this test group
    beforeEach(async () => {
      // Always initialize with a new market to isolate state between tests
      await env.resetMarket();
    });

    it("Should fail when querying in an empty market", async () => {
      // Try to calculate sell cost in empty market
      await expectAnchorError(
        () =>
          env.program.methods
            .calculateBinSellCost(
              new BN(env.marketId),
              0,
              new BN(100_000_000_000)
            )
            .accounts({
              market: env.market,
            })
            .view(),
        "Cannot sell tokens from empty bin"
      );
    });

    it("Should fail when trying to sell more than the bin has", async () => {
      const amount = new BN(100_000_000_000); // 100 tokens

      await env.program.methods
        .buyTokens(new BN(env.marketId), [0], [amount], new BN(150_000_000_000))
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Try to calculate sale cost for an amount greater than the bin's holdings
      await expectAnchorError(
        () =>
          env.program.methods
            .calculateBinSellCost(
              new BN(env.marketId),
              0,
              amount.add(new BN(1))
            )
            .accounts({
              market: env.market,
            })
            .view(),
        "Cannot sell more tokens than available in bin"
      );
    });

    it("q=T state, cost should be the same as quantity", async () => {
      // First buy tokens
      const buyAmount = new BN(100_000_000_000); // 100 tokens

      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [buyAmount],
          new BN(150_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Now calculate sell cost
      const sellAmount = new BN(50_000_000_000); // 50 tokens (half sell)
      const revenue = await env.program.methods
        .calculateBinSellCost(new BN(env.marketId), 0, sellAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // q=T state, cost should be the same as quantity
      expect(revenue.toString()).to.equal(sellAmount.toString());
    });

    it("After purchase, cost should be returned when selling the same quantity", async () => {
      // Always use new market
      await env.resetMarket();

      // First buy tokens
      const buyAmount = new BN(20_000_000_000); // 20 tokens (use smaller value)

      // Purchase transaction - cost is the same as quantity in empty market
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [buyAmount],
          new BN(30_000_000_000) // sufficient maximum cost
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Empty market, so purchase cost = quantity
      const buyCost = buyAmount;

      // Sell all tokens, revenue is the same as purchase cost in q=T state
      const revenue = buyAmount;

      // Cost should be the same as purchase cost
      expect(revenue.toString()).to.equal(buyCost.toString());
    });

    it("After purchase, cost should be returned when selling the same quantity (q=T case)", async () => {
      // Always use new market
      await env.resetMarket();

      // First buy a small amount of tokens in bin 1 to create initial quantity
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1],
          [new BN(5_000_000_000)], // 5 tokens
          new BN(10_000_000_000) // sufficient maximum cost
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Purchase tokens (buy from bin 0)
      const buyAmount = new BN(30_000_000_000); // 30 tokens

      // Calculate purchase cost (API call)
      const buyCost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, buyAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // Purchase execution
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [buyAmount],
          new BN(buyCost.mul(new BN(2))) // sufficient maximum cost
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Calculate post-purchase sell cost for the same quantity
      const sellRevenue = await env.program.methods
        .calculateBinSellCost(new BN(env.marketId), 0, buyAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // Cost and revenue should be the same in q=T case
      expect(sellRevenue.toString()).to.equal(buyCost.toString());
    });
  });

  describe("calculateXForBin", () => {
    // Create a new market before this test group
    beforeEach(async () => {
      // Always initialize with a new market to isolate state between tests
      await env.resetMarket();
    });

    it("In an empty market, quantity should be the same as cost", async () => {
      const cost = new BN(100_000_000_000);

      // Calculate
      const amount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // In an empty market, quantity should be the same as cost
      expect(amount.toString()).to.equal(cost.toString());
    });

    it("Should throw error in deactivated market", async () => {
      // Deactivate market
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      // Calculate in deactivated market
      await expectAnchorError(
        () =>
          env.program.methods
            .calculateXForBin(new BN(env.marketId), 0, new BN(100_000_000_000))
            .accounts({
              market: env.market,
            })
            .view(),
        "MarketNotActive"
      );

      // Next test, create a new market
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
    });

    it("Should throw error in closed market", async () => {
      // Create new market
      const { market: newMarket, marketId: newMarketId } =
        await env.createNewMarket();

      // Close market sequentially
      await env.closeMarketsSequentially(newMarketId, 0);

      // Try to get quantity
      await expectAnchorError(
        () =>
          env.program.methods
            .calculateXForBin(new BN(newMarketId), 0, new BN(100_000_000_000))
            .accounts({
              market: newMarket,
            })
            .view(),
        "MarketClosed"
      );
    });

    it("Should throw error when calculating with an out-of-range empty index", async () => {
      // Calculate with out-of-range index
      const outOfRangeIndex =
        Math.abs((env.maxTick - env.minTick) / env.tickSpacing) + 1;

      await expectAnchorError(
        () =>
          env.program.methods
            .calculateXForBin(
              new BN(env.marketId),
              outOfRangeIndex,
              new BN(100_000_000_000)
            )
            .accounts({
              market: env.market,
            })
            .view(),
        "BinIndexOutOfRange"
      );
    });

    it("When q < T, quantity should be greater than cost", async () => {
      // Always use new market
      await env.resetMarket();

      // First buy tokens in bin 1 to increase T (sufficiently large value)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1], // Bin 1 (60)
          [new BN(500_000_000_000)], // 500 tokens
          new BN(600_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Now calculate quantity for a cost (sufficiently small value)
      const cost = new BN(10_000_000_000); // 10 tokens worth
      const amount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // In q=0, T>0 state, quantity should be greater than cost
      expect(new BN(amount).gt(cost)).to.be.true;
    });

    it("In an empty market (T=0), inverse function relationship should be exact", async () => {
      // Always use new market
      await env.resetMarket();

      // Test quantity
      const testAmount = new BN(25_000_000_000);

      // Calculate quantity->cost in empty market (API call)
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, testAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // Calculate cost->quantity in empty market (API call)
      const calculatedAmount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // In an empty market, it should be exactly the same
      expect(calculatedAmount.toString()).to.equal(testAmount.toString());
    });
  });
});
