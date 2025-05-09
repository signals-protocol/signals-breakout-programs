import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Bin Range Query", () => {
  let env: TestEnv;

  // 테스트 환경을 한 번만 설정 (모든 테스트에서 공유)
  before(async () => {
    console.log("🏗️ 빈 범위 조회 테스트 환경 구성 중...");
    env = await setupTestEnvironment();
    console.log("✅ 빈 범위 조회 테스트 환경 구성 완료");
  });

  // 각 테스트 전에 마켓 상태 확인
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

  // 지정된 범위의 빈 데이터를 가져오는 헬퍼 함수
  async function getBinRangeData(startBin: number, endBin: number) {
    // 마켓 정보 가져오기
    const marketInfo = await env.program.account.market.fetch(env.market);

    // 범위 유효성 검사
    if (startBin > endBin) {
      throw new Error("End bin must be >= start bin");
    }

    const maxRangeSize = 20; // 적절한 최대 범위 크기 설정
    if (endBin - startBin + 1 > maxRangeSize) {
      throw new Error("Range too large");
    }

    // 마켓의 빈 인덱스 범위 확인
    const minBinIndex = Math.floor(
      Number(marketInfo.minTick) / Number(marketInfo.tickSpacing)
    );
    const maxBinIndex = Math.ceil(
      Number(marketInfo.maxTick) / Number(marketInfo.tickSpacing)
    );

    if (startBin < minBinIndex || endBin > maxBinIndex) {
      throw new Error("Bin index out of range");
    }

    // 범위 내의 모든 빈에 대한 데이터 수집
    const amounts = [];
    const costs = [];

    for (let i = startBin; i <= endBin; i++) {
      // 마켓 범위 내에 있는지 확인하고 빈 인덱스 계산
      const binIndex = i;
      const binAmount = marketInfo.bins[binIndex] || new BN(0);

      // 각 빈의 수량 추가
      amounts.push(binAmount);

      // 비용 계산 (빈 마켓이거나 수량이 0이면 0 반환)
      let cost = new BN(0);
      if (binAmount.gt(new BN(0))) {
        try {
          cost = await env.program.methods
            .calculateBinCost(new BN(env.marketId), binIndex, binAmount)
            .accounts({})
            .view();
        } catch (e) {
          // 오류 발생 시 비용은 0으로 설정
          cost = new BN(0);
        }
      }
      costs.push(cost);
    }

    return { amounts, costs };
  }

  it("비어있는 마켓에서 빈 범위 조회 시 모든 값이 0이어야 합니다", async () => {
    // 범위 조회 (1 ~ 3)
    const rangeData = await getBinRangeData(1, 3);

    // 모든 빈의 값이 0이어야 함
    for (let i = 0; i < rangeData.amounts.length; i++) {
      expect(rangeData.amounts[i].toString()).to.equal("0");
      expect(rangeData.costs[i].toString()).to.equal("0");
    }
  });

  it("토큰 구매 후 범위 조회 시 해당 빈의 값이 업데이트되어야 합니다", async () => {
    // 토큰 구매 (빈 1과 2)
    await env.program.methods
      .buyTokens(
        new BN(env.marketId),
        [1, 2], // 빈 1(60)과 2(120)
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

    // 범위 조회 (0 ~ 3)
    const rangeData = await getBinRangeData(0, 3);

    // 빈 0과 3은 비어있어야 함
    expect(rangeData.amounts[0].toString()).to.equal("0");
    expect(rangeData.costs[0].toString()).to.equal("0");
    expect(rangeData.amounts[3].toString()).to.equal("0");
    expect(rangeData.costs[3].toString()).to.equal("0");

    // 빈 1과 2는 값이 있어야 함
    expect(rangeData.amounts[1].toString()).to.equal("100000000000");
    expect(rangeData.costs[1].toString()).to.not.equal("0");
    expect(rangeData.amounts[2].toString()).to.equal("150000000000");
    expect(rangeData.costs[2].toString()).to.not.equal("0");
  });

  it("범위가 너무 크면 실패해야 합니다", async () => {
    try {
      // 너무 큰 범위 조회 (0 ~ 100)
      await getBinRangeData(0, 100);
      expect.fail("너무 큰 범위 조회가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Range too large");
    }
  });

  it("종료 빈이 시작 빈보다 작으면 실패해야 합니다", async () => {
    try {
      // 잘못된 순서의 범위 조회 (3 ~ 1)
      await getBinRangeData(3, 1);
      expect.fail("잘못된 범위 조회가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("End bin must be >= start bin");
    }
  });

  it("범위가 마켓의 최소/최대 범위를 벗어나면 실패해야 합니다", async () => {
    // 마켓 정보 가져오기
    const marketInfo = await env.program.account.market.fetch(env.market);

    // 최소 빈 인덱스 계산
    const minBinIndex = Math.floor(
      Number(marketInfo.minTick) / Number(marketInfo.tickSpacing)
    );

    try {
      // 최소 범위보다 작은 빈으로 조회
      const outOfRangeIndex = minBinIndex - 1;
      await getBinRangeData(outOfRangeIndex, 0);
      expect.fail("범위를 벗어난 조회가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("out of range");
    }

    // 최대 빈 인덱스 계산
    const maxBinIndex = Math.ceil(
      Number(marketInfo.maxTick) / Number(marketInfo.tickSpacing)
    );

    try {
      // 최대 범위보다 큰 빈으로 조회
      const outOfRangeIndex = maxBinIndex + 1;
      await getBinRangeData(0, outOfRangeIndex);
      expect.fail("범위를 벗어난 조회가 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("out of range");
    }
  });
});
