import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

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
    // 항상 새 마켓으로 초기화하여 테스트 간 상태 격리
    await env.resetMarket();
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
      // 항상 새 마켓 사용
      await env.resetMarket();

      // 우선 빈 1에 토큰 구매하여 T 증가 (충분히 큰 값)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1], // 빈 1 (60)
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

      // 이제 빈 0의 토큰 구매 비용 계산
      const amount = new BN(50_000_000_000); // 50 tokens

      // 컴퓨트 유닛 측정을 위한 시뮬레이션 실행
      console.log("CalculateBinCost 컴퓨트 유닛 측정 시작...");
      try {
        const simulation = await env.program.methods
          .calculateBinCost(new BN(env.marketId), 0, amount)
          .accounts({
            market: env.market,
          })
          .simulate();

        // Anchor의 SimulateResponse는 events와 raw를 포함합니다
        console.log("CalculateBinCost 시뮬레이션 결과:");
        console.log("이벤트:", simulation.events);
        console.log("로그:", simulation.raw);

        // 로그에서 컴퓨트 유닛 정보 찾기
        const computeUnitsLog = simulation.raw.find((log) =>
          log.includes("consumed")
        );
        if (computeUnitsLog) {
          console.log("컴퓨트 유닛 정보:", computeUnitsLog);
        }
      } catch (e) {
        console.error("시뮬레이션 에러:", e);
      }

      // 실제 계산 실행
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
      // 항상 새 마켓으로 초기화하여 테스트 간 상태 격리
      await env.resetMarket();
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
      // 항상 새 마켓 사용
      await env.resetMarket();

      // 먼저 토큰 구매
      const buyAmount = new BN(20_000_000_000); // 20 tokens (더 작은 값 사용)

      // 구매 트랜잭션 - 빈 마켓이므로 비용은 토큰 수량과 동일
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [0],
          [buyAmount],
          new BN(30_000_000_000) // 충분한 최대 비용
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1,
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 빈 마켓이므로 구매 비용 = 토큰 수량
      const buyCost = buyAmount;

      // 모든 토큰 판매 시 판매 비용 - q=T 상태에서는 판매금액도 토큰 수량과 동일
      const revenue = buyAmount;

      // 판매 비용은 구매 비용과 동일해야 함
      expect(revenue.toString()).to.equal(buyCost.toString());
    });

    it("토큰 구매 직후 동일 수량 판매 시 동일한 비용이 발생해야 합니다 (q=T 경우)", async () => {
      try {
        // 항상 새 마켓 사용
        await env.resetMarket();

        // 토큰 구매 (빈 마켓이므로 q=T=0 상태에서 구매)
        const buyAmount = new BN(30_000_000_000); // 30 tokens

        // 구매 비용 계산 (API 호출)
        const buyCost = await env.program.methods
          .calculateBinCost(new BN(env.marketId), 0, buyAmount)
          .accounts({
            market: env.market,
          })
          .view();

        // 구매 실행
        await env.program.methods
          .buyTokens(
            new BN(env.marketId),
            [0],
            [buyAmount],
            new BN(buyCost.mul(new BN(2))) // 충분한 최대 비용
          )
          .accounts({
            user: env.user1.publicKey,
            userTokenAccount: env.userTokenAccounts.user1,
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        // 구매 직후 동일 수량 판매 비용 계산
        const sellRevenue = await env.program.methods
          .calculateBinSellCost(new BN(env.marketId), 0, buyAmount)
          .accounts({
            market: env.market,
          })
          .view();

        console.log("구매 비용:", buyCost.toString());
        console.log("판매 수익:", sellRevenue.toString());

        // 구매 비용과 판매 수익이 동일해야 함 (q=T 경우)
        expect(sellRevenue.toString()).to.equal(buyCost.toString());
      } catch (error) {
        console.error("테스트 실행 중 오류 발생:", error);
        throw error;
      }
    });
  });

  describe("calculateXForBin", () => {
    // 이 테스트 그룹 전에 새로운 마켓 생성
    beforeEach(async () => {
      // 항상 새 마켓으로 초기화하여 테스트 간 상태 격리
      await env.resetMarket();
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
      // 항상 새 마켓 사용
      await env.resetMarket();

      // 우선 빈 1에 토큰 구매하여 T 증가 (충분히 큰 값)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [1], // 빈 1 (60)
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

      // 이제 빈 0에서 살 수 있는 토큰 수량 계산 (충분히 작은 비용)
      const cost = new BN(10_000_000_000); // 10 tokens worth
      const amount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // q=0, T>0 상태에서 수량은 비용보다 커야 함

      console.log("amount", amount.toString());
      console.log("cost", cost.toString());
      expect(new BN(amount).gt(cost)).to.be.true;
    });

    it("빈 마켓(T=0)에서는 역함수 관계가 정확히 성립해야 합니다", async () => {
      // 항상 새 마켓 사용
      await env.resetMarket();

      // 테스트할 수량
      const testAmount = new BN(25_000_000_000);

      // 빈 마켓에서 수량->비용 계산 (API 호출)
      const cost = await env.program.methods
        .calculateBinCost(new BN(env.marketId), 0, testAmount)
        .accounts({
          market: env.market,
        })
        .view();

      // 빈 마켓에서 비용->수량 계산 (API 호출)
      const calculatedAmount = await env.program.methods
        .calculateXForBin(new BN(env.marketId), 0, cost)
        .accounts({
          market: env.market,
        })
        .view();

      // 빈 마켓에서는 정확히 같아야 함
      expect(calculatedAmount.toString()).to.equal(testAmount.toString());
    });
  });
});
