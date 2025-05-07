import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";
import { RangeBetProgram } from "../target/types/range_bet_program";

describe("Market Management", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  beforeEach(async () => {
    // 각 테스트마다 마켓 초기화
    await env.resetMarket();
  });

  it("관리자가 마켓을 비활성화하고 다시 활성화할 수 있어야 합니다", async () => {
    // 마켓 비활성화
    await env.program.methods
      .activateMarket(new BN(env.marketId), false) // false = 비활성화
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // 마켓 정보 확인
    let marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.false; // active = false

    // 비활성화된 마켓에서 토큰 구매 시도
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
        })
        .signers([env.user1])
        .rpc();

      expect.fail("비활성화된 마켓에서 토큰 구매가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Market is not active");
    }

    // 마켓 다시 활성화
    await env.program.methods
      .activateMarket(new BN(env.marketId), true) // true = 활성화
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // 마켓 정보 확인
    marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.true; // active = true

    // 이제 토큰 구매가 가능해야 함
    await env.program.methods
      .buyTokens(
        new BN(env.marketId),
        [0], // binIndex
        [new BN(100_000_000_000)], // 100 tokens
        new BN(150_000_000_000) // max collateral
      )
      .accounts({
        user: env.user1.publicKey,
      })
      .signers([env.user1])
      .rpc();

    // 구매 확인
    const userPositionInfo = await env.program.account.userMarketPosition.fetch(
      userPosition
    );
    expect(userPositionInfo.bins.length).to.be.greaterThan(0);
    expect(userPositionInfo.bins[0].amount.toString()).to.equal("100000000000");
  });

  it("이미 비활성화된 마켓을 다시 비활성화하는 것은 허용되어야 합니다(멱등성)", async () => {
    // 첫 번째 비활성화
    await env.program.methods
      .activateMarket(new BN(env.marketId), false)
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // 마켓 정보 확인
    let marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.false;

    // 두 번째 비활성화 (같은 명령 반복)
    await env.program.methods
      .activateMarket(new BN(env.marketId), false)
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // 마켓 정보 다시 확인
    marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.false; // 여전히 비활성화 상태
  });

  it("이미 활성화된 마켓을 다시 활성화하는 것은 허용되어야 합니다(멱등성)", async () => {
    // 마켓은 기본적으로 활성화 상태
    let marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.true;

    // 활성화 명령 실행
    await env.program.methods
      .activateMarket(new BN(env.marketId), true)
      .accounts({
        owner: env.admin.publicKey,
      })
      .rpc();

    // 마켓 정보 다시 확인
    marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.active).to.be.true; // 여전히 활성화 상태
  });

  it("마감된 마켓은 활성화/비활성화할 수 없어야 합니다", async () => {
    // 순차적으로 마켓 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 마감 확인
    const marketInfo = await env.program.account.market.fetch(env.market);
    expect(marketInfo.closed).to.be.true;

    // 비활성화 시도
    try {
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("마감된 마켓을 비활성화하면 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Market is already closed");
    }

    // 활성화 시도
    try {
      await env.program.methods
        .activateMarket(new BN(env.marketId), true)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("마감된 마켓을 활성화하면 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Market is already closed");
    }
  });

  it("관리자가 아닌 사용자는 마켓 상태를 변경할 수 없어야 합니다", async () => {
    // 일반 사용자가 비활성화 시도
    try {
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.user1.publicKey,
        })
        .signers([env.user1])
        .rpc();

      expect.fail("관리자가 아닌 사용자가 마켓 상태를 변경하면 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Owner only function");
    }
  });
});
