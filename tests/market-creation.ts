import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Market Creation", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  it("올바른 파라미터로 마켓이 생성되어야 합니다", async () => {
    // 현재 시간 + 2주 후 마감 시간 설정
    const closeTime = Math.floor(Date.now() / 1000) + 14 * 24 * 60 * 60;

    // 새로운 마켓 ID를 얻어옴 (현재 마켓 카운트)
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const newMarketId = programState.marketCount.toNumber();

    // 새 마켓 계정 주소 계산
    const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      env.program.programId
    );

    // vault authority PDA 계산
    const [vaultAuthority] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      env.program.programId
    );

    // 마켓 생성 파라미터
    const tickSpacing = 120;
    const minTick = -720;
    const maxTick = 720;

    // 새 마켓 생성
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

    // 생성된 마켓 정보 가져오기
    const marketInfo = await env.program.account.market.fetch(newMarket);

    // 파라미터 확인
    expect(marketInfo.active).to.be.true;
    expect(marketInfo.closed).to.be.false;
    expect(marketInfo.tickSpacing.toString()).to.equal(tickSpacing.toString());
    expect(marketInfo.minTick.toString()).to.equal(minTick.toString());
    expect(marketInfo.maxTick.toString()).to.equal(maxTick.toString());
    expect(marketInfo.tTotal.toString()).to.equal("0"); // 초기 공급량
    expect(marketInfo.collateralBalance.toString()).to.equal("0"); // 초기 담보
    expect(marketInfo.winningBin).to.be.null; // 아직 결정되지 않음
    expect(marketInfo.openTs.toString()).to.not.equal("0"); // 오픈 시간은 0이 아니어야 함
    expect(marketInfo.closeTs.toString()).to.equal(closeTime.toString()); // 마감 시간
  });

  it("유효하지 않은 틱 파라미터로 마켓 생성이 실패해야 합니다", async () => {
    const closeTime = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60;

    // tickSpacing이 0인 경우 (양수여야 함)
    try {
      await env.program.methods
        .createMarket(
          0, // 0은 허용되지 않음
          new BN(-360),
          new BN(360),
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("tickSpacing이 0인 경우 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Tick spacing must be positive");
    }

    // minTick이 tickSpacing의 배수가 아닌 경우
    try {
      await env.program.methods
        .createMarket(
          60,
          new BN(-361), // 60의 배수가 아님
          new BN(360),
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("minTick이 tickSpacing의 배수가 아닌 경우 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include(
        "Min tick must be a multiple of tick spacing"
      );
    }

    // maxTick이 tickSpacing의 배수가 아닌 경우
    try {
      await env.program.methods
        .createMarket(
          60,
          new BN(-360),
          new BN(361), // 60의 배수가 아님
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("maxTick이 tickSpacing의 배수가 아닌 경우 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include(
        "Max tick must be a multiple of tick spacing"
      );
    }

    // minTick >= maxTick인 경우
    try {
      await env.program.methods
        .createMarket(
          60,
          new BN(360),
          new BN(360), // minTick과 같음
          new BN(closeTime)
        )
        .accounts({
          owner: env.admin.publicKey,
          collateralMint: env.collateralMint,
        })
        .rpc();

      expect.fail("minTick >= maxTick인 경우 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Min tick must be less than max tick");
    }
  });

  it("관리자만 마켓을 생성할 수 있어야 합니다", async () => {
    const closeTime = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60;
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const newMarketId = programState.marketCount.toNumber();

    // 일반 사용자가 마켓 생성 시도
    try {
      await env.program.methods
        .createMarket(60, new BN(-360), new BN(360), new BN(closeTime))
        .accounts({
          owner: env.user1.publicKey, // 관리자가 아닌 일반 사용자
          collateralMint: env.collateralMint,
        })
        .signers([env.user1])
        .rpc();

      expect.fail("관리자가 아닌 사용자가 마켓을 생성하면 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Owner only function");
    }
  });

  it("여러 마켓을 생성하면 자동으로 증가하는 ID가 부여되어야 합니다", async () => {
    // 시작 마켓 카운트 가져오기
    const initialState = await env.program.account.programState.fetch(
      env.programState
    );
    const startCount = initialState.marketCount.toNumber();

    // 세 개의 마켓 생성 파라미터
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

    // 마켓 생성
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

      // 마켓 정보 확인
      const marketInfo = await env.program.account.market.fetch(newMarket);

      // 파라미터 확인
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

    // 마켓 카운트 확인
    const finalState = await env.program.account.programState.fetch(
      env.programState
    );
    expect(finalState.marketCount.toNumber()).to.equal(
      startCount + marketParams.length
    );
  });

  it("openTimestamp와 closeTimestamp가 올바르게 저장되어야 합니다", async () => {
    const closeTime = Math.floor(Date.now() / 1000) + 14 * 24 * 60 * 60; // 2주 후

    // 새로운 마켓 ID
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const newMarketId = programState.marketCount.toNumber();

    // 새 마켓 계정 주소 계산
    const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      env.program.programId
    );

    // 현재 시간 저장
    const now = Math.floor(Date.now() / 1000);

    // 마켓 생성
    await env.program.methods
      .createMarket(60, new BN(-540), new BN(540), new BN(closeTime))
      .accounts({
        owner: env.admin.publicKey,
        collateralMint: env.collateralMint,
      })
      .rpc();

    // 마켓 정보 가져오기
    const marketInfo = await env.program.account.market.fetch(newMarket);

    // 타임스탬프 확인
    expect(marketInfo.openTs.toNumber()).to.be.greaterThan(0); // 오픈 시간이 설정됨
    expect(marketInfo.openTs.toNumber()).to.be.approximately(now, 10); // 현재 시간과 비슷해야 함 (10초 오차 허용)
    expect(marketInfo.closeTs.toNumber()).to.equal(closeTime); // 마감 시간이 올바르게 설정됨
  });
});
