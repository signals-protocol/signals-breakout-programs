import { expect } from "chai";
import { BN } from "bn.js";
import { setupTestEnvironment, TestEnv } from "./setup";

describe("Collateral Withdrawal", () => {
  let env: TestEnv;

  before(async () => {
    env = await setupTestEnvironment();
  });

  beforeEach(async () => {
    // 각 테스트마다 마켓 초기화
    await env.resetMarket();
  });

  it("관리자가 마켓 담보금을 출금할 수 없어야 합니다 (닫히지 않은 마켓)", async () => {
    // 먼저 사용자가 토큰을 구매하여 담보금 추가
    const binIndices = [0, 1, 2]; // 0, 60, 120에 해당
    const amounts = [
      new BN(100_000_000_000),
      new BN(200_000_000_000),
      new BN(100_000_000_000),
    ];
    const maxCollateral = new BN(500_000_000_000);

    // 토큰 구매
    await env.program.methods
      .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
      .accounts({
        user: env.user1.publicKey,
        userTokenAccount: env.userTokenAccounts.user1,
        vault: env.vault,
      })
      .signers([env.user1])
      .rpc();

    // 마켓 정보 확인하여 담보금이 저장되었는지 확인
    const marketInfoBefore = await env.program.account.market.fetch(env.market);
    expect(marketInfoBefore.collateralBalance.toString()).to.not.equal("0");

    // 닫히지 않은 마켓에서 관리자가 담보금 출금 시도
    try {
      await env.program.methods
        .withdrawCollateral(new BN(env.marketId))
        .accounts({
          owner: env.admin.publicKey,
          ownerTokenAccount: env.userTokenAccounts.admin,
          vault: env.vault,
        })
        .rpc();

      expect.fail("닫히지 않은 마켓에서 담보금 출금이 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Market is not closed");
    }
  });

  it("관리자가 아닌 사용자는 담보금을 출금할 수 없어야 합니다", async () => {
    // 마켓 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 일반 사용자가 담보금 출금 시도
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

      expect.fail("관리자가 아닌 사용자가 담보금을 출금하면 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("Owner only function");
    }
  });

  it("담보금이 없을 때 출금 시도는 실패해야 합니다", async () => {
    // 마켓 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 아직 토큰 구매가 없는 마켓에서 출금 시도
    try {
      await env.program.methods
        .withdrawCollateral(new BN(env.marketId))
        .accounts({
          owner: env.admin.publicKey,
          ownerTokenAccount: env.userTokenAccounts.admin,
          vault: env.vault,
        })
        .rpc();

      expect.fail("담보금이 없는 마켓에서 출금이 실패해야 함");
    } catch (e) {
      expect(e.toString()).to.include("No collateral to withdraw");
    }
  });

  it("마감된 마켓에서 담보금을 출금할 수 있어야 합니다", async () => {
    // 먼저 사용자가 토큰을 구매하여 담보금 추가
    const binIndices = [0, 1];
    const amounts = [new BN(100_000_000_000), new BN(200_000_000_000)];
    const maxCollateral = new BN(350_000_000_000);

    // 토큰 구매
    await env.program.methods
      .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
      .accounts({
        user: env.user1.publicKey,
        userTokenAccount: env.userTokenAccounts.user1,
        vault: env.vault,
      })
      .signers([env.user1])
      .rpc();

    // 마켓 정보 확인하여 담보금이 저장되었는지 확인
    const marketInfoBefore = await env.program.account.market.fetch(env.market);
    const collateralAmount = marketInfoBefore.collateralBalance;
    expect(collateralAmount.toString()).to.not.equal("0");

    // 순차적으로 마켓 닫기
    await env.closeMarketsSequentially(env.marketId, 0);

    // 관리자가 담보금 출금
    await env.program.methods
      .withdrawCollateral(new BN(env.marketId))
      .accounts({
        owner: env.admin.publicKey,
        ownerTokenAccount: env.userTokenAccounts.admin,
        vault: env.vault,
      })
      .rpc();

    // 마켓 정보 확인
    const marketInfoAfter = await env.program.account.market.fetch(env.market);
    expect(marketInfoAfter.collateralBalance.toString()).to.equal("0"); // 담보금 = 0
  });
});
