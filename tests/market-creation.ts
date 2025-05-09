import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Market Creation", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  it("Market should be created with correct parameters", async () => {
    // Set close time 2 weeks from now
    const closeTime = Math.floor(Date.now() / 1000) + 14 * 24 * 60 * 60;

    // Get new market ID (current market count)
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const newMarketId = programState.marketCount.toNumber();

    // Calculate new market account address
    const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      env.program.programId
    );

    // Calculate vault authority PDA
    const [vaultAuthority] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      env.program.programId
    );

    // Market creation parameters
    const tickSpacing = 120;
    const minTick = -720;
    const maxTick = 720;

    // Create new market
    await env.program.methods
      .createMarket(
        tickSpacing,
        new BN(minTick),
        new BN(maxTick),
        new BN(closeTime)
      )
      .accounts({
        owner: env.admin.publicKey,
        collateralMint: env.collateralMint,
      })
      .rpc();

    // Get created market info
    const marketInfo = await env.program.account.market.fetch(newMarket);

    // Verify parameters
    expect(marketInfo.active).to.be.true;
    expect(marketInfo.closed).to.be.false;
    expect(marketInfo.tickSpacing.toString()).to.equal(tickSpacing.toString());
    expect(marketInfo.minTick.toString()).to.equal(minTick.toString());
    expect(marketInfo.maxTick.toString()).to.equal(maxTick.toString());
    expect(marketInfo.tTotal.toString()).to.equal("0"); // Initial supply
    expect(marketInfo.collateralBalance.toString()).to.equal("0"); // Initial collateral
    expect(marketInfo.winningBin).to.be.null; // Not determined yet
    expect(marketInfo.openTs.toString()).to.not.equal("0"); // Open time should not be 0
    expect(marketInfo.closeTs.toString()).to.equal(closeTime.toString()); // Close time
  });

  it("Market creation should fail with invalid tick parameters", async () => {
    const closeTime = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60;

    // When tickSpacing is 0 (must be positive)
    try {
      await env.program.methods
        .createMarket(
          0, // 0 is not allowed
          new BN(-360),
          new BN(360),
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("Should fail when tickSpacing is 0");
    } catch (e) {
      expect(e.toString()).to.include("Tick spacing must be positive");
    }

    // When minTick is not a multiple of tickSpacing
    try {
      await env.program.methods
        .createMarket(
          60,
          new BN(-361), // Not a multiple of 60
          new BN(360),
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("Should fail when minTick is not a multiple of tickSpacing");
    } catch (e) {
      expect(e.toString()).to.include(
        "Min tick must be a multiple of tick spacing"
      );
    }

    // When maxTick is not a multiple of tickSpacing
    try {
      await env.program.methods
        .createMarket(
          60,
          new BN(-360),
          new BN(361), // Not a multiple of 60
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("Should fail when maxTick is not a multiple of tickSpacing");
    } catch (e) {
      expect(e.toString()).to.include(
        "Max tick must be a multiple of tick spacing"
      );
    }

    // When minTick >= maxTick
    try {
      await env.program.methods
        .createMarket(
          60,
          new BN(360),
          new BN(360), // Same as minTick
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("Should fail when minTick >= maxTick");
    } catch (e) {
      expect(e.toString()).to.include("Min tick must be less than max tick");
    }
  });

  it("Only admin should be able to create markets", async () => {
    const closeTime = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60;
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const newMarketId = programState.marketCount.toNumber();

    // Regular user attempts to create market
    try {
      await env.program.methods
        .createMarket(60, new BN(-360), new BN(360), new BN(closeTime))
        .accounts({
          owner: env.user1.publicKey, // Regular user, not admin
          collateralMint: env.collateralMint,
        })
        .signers([env.user1])
        .rpc();

      expect.fail("Non-admin user creating market should fail");
    } catch (e) {
      expect(e.toString()).to.include("Owner only function");
    }
  });

  it("Multiple markets should be assigned auto-incrementing IDs", async () => {
    // Get initial market count
    const initialState = await env.program.account.programState.fetch(
      env.programState
    );
    const startCount = initialState.marketCount.toNumber();

    // Three market creation parameters
    const marketParams = [
      {
        tickSpacing: 60,
        minTick: -360,
        maxTick: 360,
        closeTime: Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60,
      },
      {
        tickSpacing: 120,
        minTick: -720,
        maxTick: 720,
        closeTime: Math.floor(Date.now() / 1000) + 14 * 24 * 60 * 60,
      },
      {
        tickSpacing: 180,
        minTick: -1080,
        maxTick: 1080,
        closeTime: Math.floor(Date.now() / 1000) + 21 * 24 * 60 * 60,
      },
    ];

    // Create markets
    for (let i = 0; i < marketParams.length; i++) {
      const params = marketParams[i];
      const currentMarketId = startCount + i;

      const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("market"),
          new BN(currentMarketId).toArrayLike(Buffer, "le", 8),
        ],
        env.program.programId
      );

      await env.program.methods
        .createMarket(
          params.tickSpacing,
          new BN(params.minTick),
          new BN(params.maxTick),
          new BN(params.closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      // Verify market info
      const marketInfo = await env.program.account.market.fetch(newMarket);

      // Verify parameters
      expect(marketInfo.active).to.be.true;
      expect(marketInfo.closed).to.be.false;
      expect(marketInfo.tickSpacing.toString()).to.equal(
        params.tickSpacing.toString()
      );
      expect(marketInfo.minTick.toString()).to.equal(params.minTick.toString());
      expect(marketInfo.maxTick.toString()).to.equal(params.maxTick.toString());
      expect(marketInfo.closeTs.toString()).to.equal(
        params.closeTime.toString()
      );
    }

    // Verify market count
    const finalState = await env.program.account.programState.fetch(
      env.programState
    );
    expect(finalState.marketCount.toNumber()).to.equal(
      startCount + marketParams.length
    );
  });

  it("openTimestamp and closeTimestamp should be correctly stored", async () => {
    const closeTime = Math.floor(Date.now() / 1000) + 14 * 24 * 60 * 60; // 2 weeks from now

    // Get new market ID
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const newMarketId = programState.marketCount.toNumber();

    // Calculate new market account address
    const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      env.program.programId
    );

    // Get current time
    const now = Math.floor(Date.now() / 1000);

    // Create market
    await env.program.methods
      .createMarket(60, new BN(-540), new BN(540), new BN(closeTime))
      .accounts({
        owner: env.admin.publicKey,
        collateralMint: env.collateralMint,
      })
      .rpc();

    // Get market info
    const marketInfo = await env.program.account.market.fetch(newMarket);

    // Verify timestamps
    expect(marketInfo.openTs.toNumber()).to.be.greaterThan(0); // Open time should be set
    expect(marketInfo.openTs.toNumber()).to.be.approximately(now, 10); // Open time should be close to current time (10 seconds tolerance)
    expect(marketInfo.closeTs.toNumber()).to.equal(closeTime); // Close time should be correctly set
  });
});
