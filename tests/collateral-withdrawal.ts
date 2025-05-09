import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Collateral Withdrawal", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  beforeEach(async () => {
    // Initialize market for each test
    await env.resetMarket();
  });

  it("Admin should not be able to withdraw market collateral (market not closed)", async () => {
    // First have user buy tokens to add collateral
    const binIndices = [0, 1, 2]; // Corresponding to 0, 60, 120
    const amounts = [
      new BN(100_000_000_000),
      new BN(200_000_000_000),
      new BN(100_000_000_000),
    ];
    const maxCollateral = new BN(500_000_000_000);

    // Buy tokens
    await env.program.methods
      .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
      .accounts({
        user: env.user1.publicKey,
        userTokenAccount: env.userTokenAccounts.user1,
        vault: env.vault,
      })
      .signers([env.user1])
      .rpc();

    // Verify market info to ensure collateral was stored
    const marketInfoBefore = await env.program.account.market.fetch(env.market);
    expect(marketInfoBefore.collateralBalance.toString()).to.not.equal("0");

    // Admin attempts to withdraw collateral from market that's not closed
    try {
      await env.program.methods
        .withdrawCollateral(new BN(env.marketId))
        .accounts({
          owner: env.admin.publicKey,
          ownerTokenAccount: env.userTokenAccounts.admin,
          vault: env.vault,
        })
        .rpc();

      expect.fail("Collateral withdrawal from unclosed market should fail");
    } catch (e) {
      expect(e.toString()).to.include("Market is not closed");
    }
  });

  it("Non-admin users should not be able to withdraw collateral", async () => {
    // Close market
    await env.closeMarketsSequentially(env.marketId, 0);

    // Regular user attempts to withdraw collateral
    try {
      await env.program.methods
        .withdrawCollateral(new BN(env.marketId))
        .accounts({
          owner: env.user1.publicKey,
          ownerTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      expect.fail("Collateral withdrawal by non-admin user should fail");
    } catch (e) {
      expect(e.toString()).to.include("Owner only function");
    }
  });

  it("Withdrawal attempt should fail when there's no collateral", async () => {
    // Close market
    await env.closeMarketsSequentially(env.marketId, 0);

    // Attempt to withdraw from market with no token purchases
    try {
      await env.program.methods
        .withdrawCollateral(new BN(env.marketId))
        .accounts({
          owner: env.admin.publicKey,
          ownerTokenAccount: env.userTokenAccounts.admin,
          vault: env.vault,
        })
        .rpc();

      expect.fail("Withdrawal from market with no collateral should fail");
    } catch (e) {
      expect(e.toString()).to.include("No collateral to withdraw");
    }
  });

  it("Should be able to withdraw collateral from closed market", async () => {
    // First have user buy tokens to add collateral
    const binIndices = [0, 1];
    const amounts = [new BN(100_000_000_000), new BN(200_000_000_000)];
    const maxCollateral = new BN(350_000_000_000);

    // Buy tokens
    await env.program.methods
      .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
      .accounts({
        user: env.user1.publicKey,
        userTokenAccount: env.userTokenAccounts.user1,
        vault: env.vault,
      })
      .signers([env.user1])
      .rpc();

    // Verify market info to ensure collateral was stored
    const marketInfoBefore = await env.program.account.market.fetch(env.market);
    const collateralAmount = marketInfoBefore.collateralBalance;
    expect(collateralAmount.toString()).to.not.equal("0");

    // Close markets sequentially
    await env.closeMarketsSequentially(env.marketId, 0);

    // Admin withdraws collateral
    await env.program.methods
      .withdrawCollateral(new BN(env.marketId))
      .accounts({
        owner: env.admin.publicKey,
        ownerTokenAccount: env.userTokenAccounts.admin,
        vault: env.vault,
      })
      .rpc();

    // Verify market info
    const marketInfoAfter = await env.program.account.market.fetch(env.market);
    expect(marketInfoAfter.collateralBalance.toString()).to.equal("0"); // Collateral = 0
  });
});
