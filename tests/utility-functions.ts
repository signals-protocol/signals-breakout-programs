import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";
import { RangeBetProgram } from "../target/types/range_bet_program";

// Anchor 에러를 테스트하기 위한 헬퍼 함수
async function expectAnchorError(
  promiseFn: () => Promise<any>,
  errorText: string
) {
  try {
    await promiseFn();
    expect.fail("예상된 에러가 발생하지 않았습니다");
  } catch (e) {
    // SimulateError의 경우 simulationResponse.logs 경로에 로그가 있음
    if (e.simulationResponse && e.simulationResponse.logs) {
      const errorLogs = e.simulationResponse.logs.join("\n");
      expect(errorLogs).to.include(errorText);
    }
    // 일반 에러의 경우 직접 logs 속성에 접근
    else if (e.logs) {
      const errorLogs = e.logs.join("\n");
      expect(errorLogs).to.include(errorText);
    }
    // 에러 메시지에서 확인 시도
    else if (e.message) {
      expect(e.message).to.include(errorText);
    }
    // 그 외 경우에는 전체 에러 객체를 문자열화하여 확인
    else {
      const errorString = JSON.stringify(e);
      expect(errorString).to.include(errorText);
    }
  }
}

describe("Utility Functions", () => {
  let env: TestEnv;

  // 테스트 환경을 한 번만 설정 (모든 테스트에서 공유)
  before(async () => {
    console.log("🏗️ 유틸리티 함수 테스트 환경 구성 중...");
    env = await setupTestEnvironment();
    console.log("✅ 유틸리티 함수 테스트 환경 구성 완료");
  });

  // 각 테스트 케이스 전에 실행
  beforeEach(async () => {
    // 마켓이 닫혔거나 비활성화된 경우 새로운 마켓을 생성
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

  describe("calculateBinCost", () => {
    it("빈 마켓에서는 비용이 수량과 동일해야 합니다", async () => {
      const amount = new BN(100_000_000_000); // 100 tokens

      // 빈 마켓에서 계산
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, amount)
        .accounts({
          market: env.market,
        })
        .view();

      // 비용은 수량과 동일해야 함
      expect(cost.toString()).to.equal(amount.toString());
    });

    it("비활성화된 마켓에서는 에러가 발생해야 합니다", async () => {
      // 먼저 토큰 구매하여 마켓에 상태 추가
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

      // 마켓 비활성화
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      // 헬퍼 함수 사용
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

    it("마감된 마켓에서는 에러가 발생해야 합니다", async () => {
      // 새 마켓 생성
      const { market: newMarket, marketId: newMarketId } =
        await env.createNewMarket();

      // 순차적으로 마켓 닫기
      await env.closeMarketsSequentially(newMarketId, 0);

      // 비용 조회 시도
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

    it("범위를 벗어난 빈 인덱스로 계산 시 에러가 발생해야 합니다", async () => {
      // 범위를 벗어난 인덱스로 계산
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

    it("q < T 일 때 비용은 수량보다 작아야 합니다", async () => {
      // 우선 빈 1에 토큰 구매하여 T 증가
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1], // 빈 1 (60)
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

      // 이제 빈 0의 토큰 구매 비용 계산
      const amount = new BN(50_000_000_000); // 50 tokens
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, amount)
        .accounts({
          market: env.market,
        })
        .view();

      // q=0, T>0 상태에서 비용은 수량보다 작아야 함
      expect(new BN(cost).lt(amount)).to.be.true;
    });
  });

  describe("calculateBinSellCost", () => {
    // 이 테스트 그룹 전에 새로운 마켓 생성
    beforeEach(async () => {
      // 마켓 상태 확인
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

    it("빈 마켓에서는 조회 시 실패해야 합니다", async () => {
      // 빈 마켓에서 판매 비용 계산 시도
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

    it("빈이 보유한 수량보다 많이 판매하려 할 때 실패해야 합니다", async () => {
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

      // 빈의 보유량보다 많은 양의 판매 비용 계산 시도
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

    it("q=T 상태에서 판매 비용은 판매 수량과 동일해야 합니다", async () => {
      // 먼저 토큰 구매
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

      // 이제 판매 비용 계산
      const sellAmount = new BN(50_000_000_000); // 50 tokens (절반 판매)
      const revenue = await env.program.methods
        .calculateBinSellCost(new BN(env.marketId), 0, sellAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // q=T 상태에서 판매 비용은 판매 수량과 동일해야 함
      expect(revenue.toString()).to.equal(sellAmount.toString());
    });

    it("구매 후 전체 판매 시 원래 비용을 돌려받아야 합니다", async () => {
      // 먼저 토큰 구매
      const buyAmount = new BN(100_000_000_000); // 100 tokens

      // 구매 트랜잭션
      const buyTx = await env.program.methods
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

      // 마켓 정보 확인하여 실제 비용 가져오기
      const marketInfo = await env.program.account.market.fetch(env.market);
      const buyCost = marketInfo.collateralBalance;

      // 모든 토큰 판매 시 판매 비용
      const revenue = await env.program.methods
        .calculateBinSellCost(new BN(env.marketId), 0, buyAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // 판매 비용은 구매 비용과 동일해야 함
      expect(revenue.toString()).to.equal(buyCost.toString());
    });
  });

  describe("calculateXForBin", () => {
    // 이 테스트 그룹 전에 새로운 마켓 생성
    beforeEach(async () => {
      // 마켓 상태 확인
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

    it("빈 마켓에서는 비용으로 살 수 있는 토큰 수량은 비용과 동일해야 합니다", async () => {
      const cost = new BN(100_000_000_000);

      // 계산
      const amount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // 빈 마켓에서는 비용으로 살 수 있는 토큰 수량은 비용과 동일해야 함
      expect(amount.toString()).to.equal(cost.toString());
    });

    it("비활성화된 마켓에서는 에러가 발생해야 합니다", async () => {
      // 마켓 비활성화
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      // 비활성화된 마켓에서 계산
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

      // 다음 테스트를 위해 새 마켓 생성
      const newMarket = await env.createNewMarket();
      env.market = newMarket.market;
      env.marketId = newMarket.marketId;
    });

    it("마감된 마켓에서는 에러가 발생해야 합니다", async () => {
      // 새 마켓 생성
      const { market: newMarket, marketId: newMarketId } =
        await env.createNewMarket();

      // 순차적으로 마켓 닫기
      await env.closeMarketsSequentially(newMarketId, 0);

      // 토큰 수량 조회 시도
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

    it("범위를 벗어난 빈 인덱스로 계산 시 에러가 발생해야 합니다", async () => {
      // 범위를 벗어난 인덱스로 계산
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

    it("q < T 일 때 비용으로 살 수 있는 토큰 수량은 비용보다 커야 합니다", async () => {
      // 우선 빈 1에 토큰 구매하여 T 증가
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1], // 빈 1 (60)
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

      // 이제 빈 0에서 살 수 있는 토큰 수량 계산
      const cost = new BN(50_000_000_000); // 50 tokens worth
      const amount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // q=0, T>0 상태에서 수량은 비용보다 커야 함
      expect(new BN(amount).gt(cost)).to.be.true;
    });

    it("calculateBinCost와 calculateXForBin은 서로 역함수가 되어야 합니다", async () => {
      // 마켓 상태 확인 및 새로운 마켓 생성 필요시
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

      // 먼저 토큰 구매하여 마켓에 상태 추가
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0, 1],
          [new BN(100_000_000_000), new BN(50_000_000_000)],
          new BN(200_000_000_000)
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 테스트할 수량
      const testAmount = new BN(25_000_000_000);

      // 1) 먼저 수량 -> 비용 계산
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, testAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // 2) 그 다음 비용 -> 수량 계산
      const calculatedAmount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // 작은 반올림 오차 허용하여 원래 수량과 동일해야 함
      const diff = calculatedAmount.sub(testAmount).abs();
      expect(diff.lten(10000)).to.be.true; // 매우 작은 오차 허용
    });
  });
});
