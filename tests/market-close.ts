import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";
import { RangeBetProgram } from "../target/types/range_bet_program";

describe("Market Close", () => {
  let env: TestEnv;

  // 테스트 환경을 한 번만 설정 (모든 테스트에서 공유)
  before(async () => {
    console.log("🏗️ 마켓 종료 테스트 환경 구성 중...");
    env = await setupTestEnvironment();

    // 프로그램 상태에서 마지막으로 닫힌 마켓 ID 확인
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    const lastClosedMarketId = programState.lastClosedMarket
      ? programState.lastClosedMarket.toNumber()
      : -1;

    console.log(`📊 마지막으로 닫힌 마켓 ID: ${lastClosedMarketId}`);

    // 현재 마켓 상태 확인
    let needNewMarket = false;

    try {
      const marketInfo = await env.program.account.market.fetch(env.market);
      console.log(
        `📊 현재 마켓 ID: ${env.marketId}, 닫힘 상태: ${marketInfo.closed}`
      );

      // 현재 마켓이 닫혔거나 비활성화된 경우, 새 마켓 생성 필요
      if (marketInfo.closed || !marketInfo.active) {
        console.log("🔄 현재 마켓이 닫혔거나 비활성화됨, 새 마켓 생성 필요");
        needNewMarket = true;
      }
    } catch (e) {
      console.log("⚠️ 현재 마켓 정보를 불러올 수 없음, 새 마켓 생성 필요");
      needNewMarket = true;
    }

    // 새 마켓 생성 (필요한 경우)
    if (needNewMarket) {
      console.log("🏦 새 마켓 생성 중...");
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
      console.log(`✅ 새 마켓 생성 완료: ID = ${env.marketId}`);
    }

    // 테스트용 베팅 설정
    console.log("💰 테스트용 베팅 생성 중...");
    await setupTestBets();
    console.log("✅ 마켓 종료 테스트 환경 구성 완료");
  });

  // 각 테스트 전에 마켓 상태가 올바른지 확인
  beforeEach(async () => {
    // 마켓이 활성 상태이고 닫히지 않았는지 확인
    try {
      const marketInfo = await env.program.account.market.fetch(env.market);

      // 마켓이 이미 닫혔거나 비활성화된 경우, 새 마켓 생성
      if (marketInfo.closed || !marketInfo.active) {
        console.log(
          `🔄 테스트를 위해 새 마켓 생성 중 (현재 마켓 ID ${env.marketId}가 닫혔거나 비활성화됨)`
        );
        const newMarket = await env.createNewMarket();
        env.market = newMarket.market;
        env.marketId = newMarket.marketId;

        // 새 마켓에 기본 베팅 설정
        await setupTestBets();
        console.log(`✅ 새 마켓 ID ${env.marketId} 생성 및 베팅 설정 완료`);
      }
    } catch (e) {
      // 마켓이 없는 경우도 새로 생성
      console.log("⚠️ 마켓 정보를 불러올 수 없음, 새 마켓 생성");
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;

      // 새 마켓에 기본 베팅 설정
      await setupTestBets();
      console.log(`✅ 새 마켓 ID ${env.marketId} 생성 및 베팅 설정 완료`);
    }
  });

  // 테스트에 사용할 기본 베팅 설정 함수
  async function setupTestBets() {
    try {
      // 마켓이 열려있는지 확인
      const marketInfo = await env.program.account.market.fetch(env.market);
      if (marketInfo.closed) {
        console.log(
          `⚠️ 마켓 ID ${env.marketId}가 이미 닫혀있어 베팅을 설정할 수 없음`
        );
        return; // 닫힌 마켓에는 베팅하지 않음
      }

      console.log(`💰 마켓 ID ${env.marketId}에 테스트 베팅 설정 중...`);

      // user1이 0 빈에 베팅
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

      // user2가 0과 1(60) 빈에 베팅
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

      // user3이 -1 빈(-60)에 베팅
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [Math.ceil(Math.abs(-60 / env.tickSpacing))], // tickIndex 계산 (절대값으로 변환)
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

      console.log(`✅ 마켓 ID ${env.marketId}에 테스트 베팅 설정 완료`);
    } catch (error) {
      console.error(`❌ 테스트 베팅 설정 중 오류 발생: ${error}`);
      throw error; // 에러를 다시 발생시켜 테스트가 실패하도록 함
    }
  }

  it("관리자가 마켓을 종료하고 승리 빈을 설정할 수 있어야 합니다", async () => {
    // 마켓이 이미 닫혔는지 확인
    const marketInfo = await env.program.account.market.fetch(env.market);
    if (marketInfo.closed) {
      console.log(`마켓 ID ${env.marketId}가 이미 닫힘, 새 마켓 생성 중...`);
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
      await setupTestBets();
    }

    // 순차적으로 마켓 닫기 (현재 마켓 ID까지)
    await env.closeMarketsSequentially(env.marketId, 0);

    // 마켓 정보 확인
    const updatedMarketInfo = await env.program.account.market.fetch(
      env.market
    );
    expect(updatedMarketInfo.closed).to.be.true; // closed = true
    expect(updatedMarketInfo.winningBin).to.not.be.null;
    expect(updatedMarketInfo.winningBin.toString()).to.equal("0"); // winningBin = 0

    // 마감된 마켓에서 토큰 구매 시도
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

      expect.fail("마감된 마켓에서 토큰 구매가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Market is closed");
    }
  });

  it("유효하지 않은 승리 빈으로 마켓 종료가 실패해야 합니다", async () => {
    // 새 마켓 생성
    const newMarket = await env.createNewMarket();
    env.market = newMarket.market;
    env.marketId = newMarket.marketId;
    await setupTestBets();

    // 범위를 벗어난 승리 빈으로 종료 시도
    const outOfRangeIndex = Math.floor(env.maxTick / env.tickSpacing) + 10; // 확실하게 범위 벗어나게

    // 이전 마켓까지 모두 순차적으로 닫기
    await env.closeMarketsSequentially(env.marketId - 1, 0);

    try {
      // 현재 마켓만 직접 닫기 시도 (이상한 값으로)
      await env.program.methods
        .closeMarket(new BN(env.marketId), outOfRangeIndex)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("범위를 벗어난 승리 빈으로 마켓 종료가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("BinIndexOutOfRange");
    }
  });

  it("이미 종료된 마켓을 다시 종료할 수 없어야 합니다", async () => {
    // 새 마켓 생성
    const newMarket = await env.createNewMarket();
    env.market = newMarket.market;
    env.marketId = newMarket.marketId;
    await setupTestBets();

    // 순차적으로 마켓 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 다시 종료 시도
    try {
      await env.program.methods
        .closeMarket(new BN(env.marketId), 1) // 다른 승리 빈으로 시도
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      expect.fail("이미 종료된 마켓을 다시 종료할 수 없어야 함");
    } catch (e) {
      expect(e.toString()).to.include("Market is closed");
    }
  });

  it("마지막으로 종료된 마켓 ID를 올바르게 추적해야 합니다", async () => {
    // 새 마켓 생성
    const newMarket = await env.createNewMarket();
    env.market = newMarket.market;
    env.marketId = newMarket.marketId;
    await setupTestBets();

    // 초기값 확인
    const initialState = await env.program.account.programState.fetch(
      env.programState
    );
    const initialLastClosed = initialState.lastClosedMarket
      ? initialState.lastClosedMarket.toNumber()
      : -1;

    // 순차적으로 마켓 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 업데이트된 값 확인
    const updatedState = await env.program.account.programState.fetch(
      env.programState
    );
    expect(updatedState.lastClosedMarket).to.not.be.null;
    expect(updatedState.lastClosedMarket.toString()).to.equal(
      env.marketId.toString()
    );

    // 이전 값보다 큰지 확인
    if (initialLastClosed >= 0) {
      expect(updatedState.lastClosedMarket.toNumber()).to.be.greaterThan(
        initialLastClosed
      );
    }
  });

  it("여러 마켓을 순차적으로 종료할 수 있어야 합니다", async () => {
    // 기존 마켓 생성 또는 새로운 마켓 생성
    const marketInfo = await env.program.account.market.fetch(env.market);
    if (marketInfo.closed) {
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
      await setupTestBets();
    }

    // 추가 마켓 생성
    const { market: newMarket, marketId: newMarketId } =
      await env.createNewMarket();

    // 첫 번째 마켓까지 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 첫 번째 마켓 닫힘 상태 확인
    const firstMarketInfo = await env.program.account.market.fetch(env.market);
    expect(firstMarketInfo.closed).to.be.true;

    // 두 번째 마켓까지 닫기
    await env.closeMarketsSequentially(newMarketId, 1);

    // 두 번째 마켓 닫힘 상태 확인
    const secondMarketInfo = await env.program.account.market.fetch(newMarket);
    expect(secondMarketInfo.closed).to.be.true;
    expect(secondMarketInfo.winningBin.toString()).to.equal("1");

    // 프로그램 상태의 last_closed_market 확인
    const programState = await env.program.account.programState.fetch(
      env.programState
    );
    expect(programState.lastClosedMarket.toString()).to.equal(
      newMarketId.toString()
    );
  });
});
