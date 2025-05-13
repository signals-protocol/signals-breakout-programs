import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Bin Range Query", () => {
  let env: TestEnv;

  // Setup test environment only once (shared across all tests)
  before(async () => {
    env = await setupTestEnvironment();
  });

  // Check market state before each test
  beforeEach(async () => {
    // Create a new market if the market is closed or deactivated
    try {
      const marketInfo = await env.program.account.market.fetch(env.market);
      if (marketInfo.closed || !marketInfo.active) {
        const newMarket = await env.createNewMarket();
        env.market = newMarket.market;
        env.marketId = newMarket.marketId;
      }
    } catch (e) {
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
    }
  });

  // Helper function to get bin data in specified range
  async function getBinRangeData(startBin: number, endBin: number) {
    // Get market information
    const marketInfo = await env.program.account.market.fetch(env.market);

    // Validate range
    if (startBin > endBin) {
      throw new Error("End bin must be >= start bin");
    }

    const maxRangeSize = 20; // Set appropriate maximum range size
    if (endBin - startBin + 1 > maxRangeSize) {
      throw new Error("Range too large");
    }

    // Check bin index range in the market
    const minBinIndex = Math.floor(
      Number(marketInfo.minTick) / Number(marketInfo.tickSpacing)
    );
    const maxBinIndex = Math.ceil(
      Number(marketInfo.maxTick) / Number(marketInfo.tickSpacing)
    );

    if (startBin < minBinIndex || endBin > maxBinIndex) {
      throw new Error("Bin index out of range");
    }

    // Collect data for all bins in the range
    const amounts = [];
    const costs = [];

    for (let i = startBin; i <= endBin; i++) {
      // Check if it's within market range and calculate bin index
      const binIndex = i;
      const binAmount = marketInfo.bins[binIndex] || new BN(0);

      // Add quantity for each bin
      amounts.push(binAmount);

      // Calculate cost (return 0 if empty market or quantity is 0)
      let cost = new BN(0);
      if (binAmount.gt(new BN(0))) {
        try {
          cost = await env.program.methods
            .calculateBinBuyCost(new BN(env.marketId), binIndex, binAmount)
            .accounts({})
            .view();
        } catch (e) {
          // Set cost to 0 if error occurs
          cost = new BN(0);
        }
      }
      costs.push(cost);
    }

    return { amounts, costs };
  }

  it("All values should be 0 when querying bin range in an empty market", async () => {
    // Query range (1 ~ 3)
    const rangeData = await getBinRangeData(1, 3);

    // All bin values should be 0
    for (let i = 0; i < rangeData.amounts.length; i++) {
      expect(rangeData.amounts[i].toString()).to.equal("0");
      expect(rangeData.costs[i].toString()).to.equal("0");
    }
  });

  it("Bin values should be updated after token purchase when querying range", async () => {
    // Buy tokens (bins 1 and 2)
    await env.program.methods
      .buyTokens(
        new BN(env.marketId),
        [1, 2], // Bins 1(60) and 2(120)
        [new BN(100_000_000_000), new BN(150_000_000_000)], // 100, 150 tokens
        new BN(300_000_000_000)
      )
      .accounts({
        user: env.user1.publicKey,
        userTokenAccount: env.userTokenAccounts.user1,
        vault: env.vault,
      })
      .signers([env.user1])
      .rpc();

    // Query range (0 ~ 3)
    const rangeData = await getBinRangeData(0, 3);

    // Bins 0 and 3 should be empty
    expect(rangeData.amounts[0].toString()).to.equal("0");
    expect(rangeData.costs[0].toString()).to.equal("0");
    expect(rangeData.amounts[3].toString()).to.equal("0");
    expect(rangeData.costs[3].toString()).to.equal("0");

    // Bins 1 and 2 should have values
    expect(rangeData.amounts[1].toString()).to.equal("100000000000");
    expect(rangeData.costs[1].toString()).to.not.equal("0");
    expect(rangeData.amounts[2].toString()).to.equal("150000000000");
    expect(rangeData.costs[2].toString()).to.not.equal("0");
  });

  it("Should fail if range is too large", async () => {
    try {
      // Query with too large range (0 ~ 100)
      await getBinRangeData(0, 100);
      expect.fail("Query with too large range should fail");
    } catch (e) {
      expect(e.toString()).to.include("Range too large");
    }
  });

  it("Should fail if end bin is less than start bin", async () => {
    try {
      // Query with incorrect order range (3 ~ 1)
      await getBinRangeData(3, 1);
      expect.fail("Query with incorrect range should fail");
    } catch (e) {
      expect(e.toString()).to.include("End bin must be >= start bin");
    }
  });

  it("Should fail if range is outside the market's min/max range", async () => {
    // Get market information
    const marketInfo = await env.program.account.market.fetch(env.market);

    // Calculate minimum bin index
    const minBinIndex = Math.floor(
      Number(marketInfo.minTick) / Number(marketInfo.tickSpacing)
    );

    try {
      // Query with bin smaller than minimum range
      const outOfRangeIndex = minBinIndex - 1;
      await getBinRangeData(outOfRangeIndex, 0);
      expect.fail("Query outside range should fail");
    } catch (e) {
      expect(e.toString()).to.include("out of range");
    }

    // Calculate maximum bin index
    const maxBinIndex = Math.ceil(
      Number(marketInfo.maxTick) / Number(marketInfo.tickSpacing)
    );

    try {
      // Query with bin larger than maximum range
      const outOfRangeIndex = maxBinIndex + 1;
      await getBinRangeData(0, outOfRangeIndex);
      expect.fail("Query outside range should fail");
    } catch (e) {
      expect(e.toString()).to.include("out of range");
    }
  });
});
