import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Token Operations", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  beforeEach(async () => {
    // Initialize market for each test
    await env.resetMarket();
  });

  describe("Token Purchase (buyTokens)", () => {
    it("User should be able to buy tokens from a single bin", async () => {
      // Initial setup
      const binIndex = 0;
      const amount = new BN(100_000_000_000); // 100 tokens
      const maxCollateral = new BN(150_000_000_000);

      // Create user position account
      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // Check market info before token purchase
      const marketInfoBefore = await env.program.account.market.fetch(
        env.market
      );
      const initialTTotal = marketInfoBefore.tTotal;
      const initialCollateral = marketInfoBefore.collateralBalance;

      // Buy tokens
      await env.program.methods
        .buyTokens(new BN(env.marketId), [binIndex], [amount], maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Check market info after purchase
      const marketInfoAfter = await env.program.account.market.fetch(
        env.market
      );
      const finalTTotal = marketInfoAfter.tTotal;
      const finalCollateral = marketInfoAfter.collateralBalance;

      // Verify total supply
      expect(finalTTotal.toString()).to.equal(
        initialTTotal.add(amount).toString()
      );

      // Verify collateral increase
      expect(finalCollateral.gt(initialCollateral)).to.be.true;

      // Check user position
      const userPositionInfo =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(userPositionInfo.bins.length).to.equal(1);
      expect(userPositionInfo.bins[0].index).to.equal(binIndex);
      expect(userPositionInfo.bins[0].amount.toString()).to.equal(
        amount.toString()
      );

      // Check bin quantity
      expect(marketInfoAfter.bins[binIndex].toString()).to.equal(
        amount.toString()
      );
    });

    it("User should be able to buy tokens from multiple bins", async () => {
      // Initial setup
      const binIndices = [0, 1, 2]; // Corresponds to 0, 60, 120
      const amounts = [
        new BN(100_000_000_000),
        new BN(50_000_000_000),
        new BN(75_000_000_000),
      ];
      const maxCollateral = new BN(300_000_000_000);

      // Create user position account
      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // Check market info before token purchase
      const marketInfoBefore = await env.program.account.market.fetch(
        env.market
      );
      const initialTTotal = marketInfoBefore.tTotal;
      const initialCollateral = marketInfoBefore.collateralBalance;

      // Buy tokens
      await env.program.methods
        .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Check market info after purchase
      const marketInfoAfter = await env.program.account.market.fetch(
        env.market
      );
      const finalTTotal = marketInfoAfter.tTotal;
      const finalCollateral = marketInfoAfter.collateralBalance;

      // Verify total supply (sum of all purchase amounts)
      const totalAmount = amounts.reduce((acc, val) => acc.add(val), new BN(0));
      expect(finalTTotal.toString()).to.equal(
        initialTTotal.add(totalAmount).toString()
      );

      // Verify collateral increase
      expect(finalCollateral.gt(initialCollateral)).to.be.true;

      // Check user position
      const userPositionInfo =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(userPositionInfo.bins.length).to.equal(binIndices.length);

      // Check quantity for each bin
      for (let i = 0; i < binIndices.length; i++) {
        const binIndex = binIndices[i];
        const binAmount = amounts[i];

        // Find the corresponding bin in user position
        const userBin = userPositionInfo.bins.find(
          (bin) => bin.index === binIndex
        );
        expect(userBin).to.not.be.undefined;
        expect(userBin.amount.toString()).to.equal(binAmount.toString());

        // Check bin quantity in market
        expect(marketInfoAfter.bins[binIndex].toString()).to.equal(
          binAmount.toString()
        );
      }
    });

    it("Purchase attempt in an inactive market should fail", async () => {
      // Deactivate market
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      // Attempt to buy
      try {
        await env.program.methods
          .buyTokens(
            new BN(env.marketId),
            [0],
            [new BN(100_000_000_000)],
            new BN(150_000_000_000)
          )
          .accounts({
            user: env.user1.publicKey,
            userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("Purchase in inactive market should fail");
      } catch (e) {
        expect(e.toString()).to.include("Market is not active");
      }
    });

    it("Purchase attempt in a closed market should fail", async () => {
      // Close market sequentially
      await env.closeMarketsSequentially(env.marketId, 0);

      // Attempt to buy
      try {
        await env.program.methods
          .buyTokens(
            new BN(env.marketId),
            [0],
            [new BN(100_000_000_000)],
            new BN(150_000_000_000)
          )
          .accounts({
            user: env.user1.publicKey,
            userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("Purchase in closed market should fail");
      } catch (e) {
        expect(e.toString()).to.include("Market is closed");
      }
    });

    it("Bin index out of range should fail", async () => {
      // Out of range index
      const outOfRangeIndex =
        Math.abs((env.maxTick - env.minTick) / env.tickSpacing) + 1;

      try {
        await env.program.methods
          .buyTokens(
            new BN(env.marketId),
            [outOfRangeIndex],
            [new BN(100_000_000_000)],
            new BN(150_000_000_000)
          )
          .accounts({
            user: env.user1.publicKey,
            userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("Purchase with out of range bin index should fail");
      } catch (e) {
        expect(e.toString()).to.include("Bin index out of range");
      }
    });

    it("Zero quantity purchase should ignore the bin", async () => {
      const binIndices = [0, 1];
      const amounts = [new BN(100_000_000_000), new BN(0)]; // Second bin is 0 quantity
      const maxCollateral = new BN(150_000_000_000);

      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // Buy tokens
      await env.program.methods
        .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Check user position
      const userPositionInfo =
        await env.program.account.userMarketPosition.fetch(userPosition);

      // Only bin 0 should be added
      expect(userPositionInfo.bins.length).to.equal(1);
      expect(userPositionInfo.bins[0].index).to.equal(0);
      expect(userPositionInfo.bins[0].amount.toString()).to.equal(
        amounts[0].toString()
      );

      // Check market info
      const marketInfo = await env.program.account.market.fetch(env.market);

      // Only bin 0 should be updated
      expect(marketInfo.bins[0].toString()).to.equal(amounts[0].toString());
      expect(marketInfo.bins[1].toString()).to.equal("0"); // Quantity should be set to 0

      // Total supply should only include actual purchased quantity
      expect(marketInfo.tTotal.toString()).to.equal(amounts[0].toString());
    });

    it("Additional purchase should correctly add to existing position", async () => {
      const binIndex = 0;
      const initialAmount = new BN(100_000_000_000);
      const additionalAmount = new BN(50_000_000_000);
      const maxCollateral = new BN(200_000_000_000);

      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // First purchase
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [binIndex],
          [initialAmount],
          maxCollateral
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // First purchase position check
      const positionAfterFirst =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(positionAfterFirst.bins.length).to.equal(1);
      expect(positionAfterFirst.bins[0].amount.toString()).to.equal(
        initialAmount.toString()
      );

      // Second purchase (adding to the same bin)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [binIndex],
          [additionalAmount],
          maxCollateral
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Second purchase position check
      const positionAfterSecond =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(positionAfterSecond.bins.length).to.equal(1); // Still 1 bin
      expect(positionAfterSecond.bins[0].amount.toString()).to.equal(
        initialAmount.add(additionalAmount).toString()
      );

      // Check market info
      const marketInfo = await env.program.account.market.fetch(env.market);
      expect(marketInfo.bins[binIndex].toString()).to.equal(
        initialAmount.add(additionalAmount).toString()
      );
    });
  });

  describe("Position Transfer (transferPosition)", () => {
    it("User should be able to transfer position to another wallet", async () => {
      // First, create user1 position
      const binIndices = [0, 1];
      const amounts = [new BN(100_000_000_000), new BN(200_000_000_000)];
      const maxCollateral = new BN(350_000_000_000);

      const user1Position = await env.getUserPosition(env.user1, env.marketId);

      // user1 buys tokens
      await env.program.methods
        .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Create user2 position account
      const user2Position = await env.getUserPosition(env.user2, env.marketId);

      // Transfer position (user1 -> user2)
      // Transfer only part of the position (50% of bin 0)
      const transferAmount = amounts[0].div(new BN(2));

      await env.program.methods
        .transferPosition(
          new BN(env.marketId),
          [binIndices[0]],
          [transferAmount]
        )
        .accounts({
          fromUser: env.user1.publicKey,
          toUser: env.user2.publicKey,
        })
        .signers([env.user1])
        .rpc();

      // Check position after transfer
      const user1PositionAfter =
        await env.program.account.userMarketPosition.fetch(user1Position);
      const user2PositionAfter =
        await env.program.account.userMarketPosition.fetch(user2Position);

      // user1 position - bin 0 is reduced by 50%, bin 1 remains unchanged
      const user1Bin0 = user1PositionAfter.bins.find(
        (bin) => bin.index === binIndices[0]
      );
      const user1Bin1 = user1PositionAfter.bins.find(
        (bin) => bin.index === binIndices[1]
      );

      expect(user1Bin0.amount.toString()).to.equal(
        amounts[0].sub(transferAmount).toString()
      );
      expect(user1Bin1.amount.toString()).to.equal(amounts[1].toString());

      // user2 position - only transferred quantity should be present
      expect(user2PositionAfter.bins.length).to.equal(1);
      expect(user2PositionAfter.bins[0].index).to.equal(binIndices[0]);
      expect(user2PositionAfter.bins[0].amount.toString()).to.equal(
        transferAmount.toString()
      );

      // Market total quantity should remain unchanged
      const marketInfo = await env.program.account.market.fetch(env.market);
      expect(marketInfo.bins[binIndices[0]].toString()).to.equal(
        amounts[0].toString()
      );
      expect(marketInfo.bins[binIndices[1]].toString()).to.equal(
        amounts[1].toString()
      );
    });

    it("Self transfer should fail", async () => {
      // First, create position
      const user1Position = await env.getUserPosition(env.user1, env.marketId);

      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [new BN(100_000_000_000)],
          new BN(150_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Self transfer attempt
      try {
        await env.program.methods
          .transferPosition(new BN(env.marketId), [0], [new BN(50_000_000_000)])
          .accounts({
            fromUser: env.user1.publicKey,
            toUser: env.user1.publicKey, // Transfer to self
          })
          .signers([env.user1])
          .rpc();

        expect.fail("Self transfer should fail");
      } catch (e) {
        expect(e.toString()).to.include("Cannot transfer to self");
      }
    });

    it("Transfer more than owned quantity should fail", async () => {
      // First, create position
      const user1Position = await env.getUserPosition(env.user1, env.marketId);
      const user2Position = await env.getUserPosition(env.user2, env.marketId);

      const amount = new BN(100_000_000_000);

      await env.program.methods
        .buyTokens(new BN(env.marketId), [0], [amount], new BN(150_000_000_000))
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // Use actual user token account
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // Attempt to transfer more than owned quantity
      try {
        await env.program.methods
          .transferPosition(
            new BN(env.marketId),
            [0],
            [amount.add(new BN(10_000_000))] // More than owned quantity
          )
          .accounts({
            fromUser: env.user1.publicKey,
            toUser: env.user2.publicKey,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("Transfer more than owned quantity should fail");
      } catch (e) {
        expect(e.toString()).to.include("Insufficient balance");
      }
    });
  });
});
