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
    // 각 테스트마다 마켓 초기화
    await env.resetMarket();
  });

  describe("토큰 구매 (buyTokens)", () => {
    it("사용자가 단일 빈에서 토큰을 구매할 수 있어야 합니다", async () => {
      // 초기 설정
      const binIndex = 0;
      const amount = new BN(100_000_000_000); // 100 tokens
      const maxCollateral = new BN(150_000_000_000);

      // 사용자 포지션 계정 생성
      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // 토큰 구매 전 마켓 정보 확인
      const marketInfoBefore = await env.program.account.market.fetch(
        env.market
      );
      const initialTTotal = marketInfoBefore.tTotal;
      const initialCollateral = marketInfoBefore.collateralBalance;

      // 토큰 구매
      await env.program.methods
        .buyTokens(new BN(env.marketId), [binIndex], [amount], maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 구매 후 마켓 정보 확인
      const marketInfoAfter = await env.program.account.market.fetch(
        env.market
      );
      const finalTTotal = marketInfoAfter.tTotal;
      const finalCollateral = marketInfoAfter.collateralBalance;

      // 총 공급량 검증
      expect(finalTTotal.toString()).to.equal(
        initialTTotal.add(amount).toString()
      );

      // 담보금 증가 검증
      expect(finalCollateral.gt(initialCollateral)).to.be.true;

      // 사용자 포지션 확인
      const userPositionInfo =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(userPositionInfo.bins.length).to.equal(1);
      expect(userPositionInfo.bins[0].index).to.equal(binIndex);
      expect(userPositionInfo.bins[0].amount.toString()).to.equal(
        amount.toString()
      );

      // 빈 수량 확인
      expect(marketInfoAfter.bins[binIndex].toString()).to.equal(
        amount.toString()
      );
    });

    it("사용자가 여러 빈에서 토큰을 구매할 수 있어야 합니다", async () => {
      // 초기 설정
      const binIndices = [0, 1, 2]; // 0, 60, 120에 해당
      const amounts = [
        new BN(100_000_000_000),
        new BN(50_000_000_000),
        new BN(75_000_000_000),
      ];
      const maxCollateral = new BN(300_000_000_000);

      // 사용자 포지션 계정 생성
      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // 토큰 구매 전 마켓 정보 확인
      const marketInfoBefore = await env.program.account.market.fetch(
        env.market
      );
      const initialTTotal = marketInfoBefore.tTotal;
      const initialCollateral = marketInfoBefore.collateralBalance;

      // 토큰 구매
      await env.program.methods
        .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 구매 후 마켓 정보 확인
      const marketInfoAfter = await env.program.account.market.fetch(
        env.market
      );
      const finalTTotal = marketInfoAfter.tTotal;
      const finalCollateral = marketInfoAfter.collateralBalance;

      // 총 공급량 검증 (모든 구매 수량의 합)
      const totalAmount = amounts.reduce((acc, val) => acc.add(val), new BN(0));
      expect(finalTTotal.toString()).to.equal(
        initialTTotal.add(totalAmount).toString()
      );

      // 담보금 증가 검증
      expect(finalCollateral.gt(initialCollateral)).to.be.true;

      // 사용자 포지션 확인
      const userPositionInfo =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(userPositionInfo.bins.length).to.equal(binIndices.length);

      // 각 빈별 수량 확인
      for (let i = 0; i < binIndices.length; i++) {
        const binIndex = binIndices[i];
        const binAmount = amounts[i];

        // 사용자 포지션의 해당 빈 찾기
        const userBin = userPositionInfo.bins.find(
          (bin) => bin.index === binIndex
        );
        expect(userBin).to.not.be.undefined;
        expect(userBin.amount.toString()).to.equal(binAmount.toString());

        // 마켓의 해당 빈 수량 확인
        expect(marketInfoAfter.bins[binIndex].toString()).to.equal(
          binAmount.toString()
        );
      }
    });

    it("활성 상태가 아닌 마켓에서 구매 시도가 실패해야 합니다", async () => {
      // 마켓 비활성화
      await env.program.methods
        .activateMarket(new BN(env.marketId), false)
        .accounts({
          owner: env.admin.publicKey,
        })
        .rpc();

      // 구매 시도
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
            userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("비활성화된 마켓에서 구매가 실패해야 함");
      } catch (e) {
        expect(e.toString()).to.include("Market is not active");
      }
    });

    it("마감된 마켓에서 구매 시도가 실패해야 합니다", async () => {
      // 순차적으로 마켓 닫기
      await env.closeMarketsSequentially(env.marketId, 0);

      // 구매 시도
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
            userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("마감된 마켓에서 구매가 실패해야 함");
      } catch (e) {
        expect(e.toString()).to.include("Market is closed");
      }
    });

    it("빈 인덱스가 범위를 벗어날 경우 구매 시도가 실패해야 합니다", async () => {
      // 범위를 벗어난 인덱스
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
            userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
            vault: env.vault,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("범위를 벗어난 빈 인덱스로 구매가 실패해야 함");
      } catch (e) {
        expect(e.toString()).to.include("Bin index out of range");
      }
    });

    it("0 수량으로 구매 요청 시 해당 빈은 무시되어야 합니다", async () => {
      const binIndices = [0, 1];
      const amounts = [new BN(100_000_000_000), new BN(0)]; // 두 번째 빈은 0 수량
      const maxCollateral = new BN(150_000_000_000);

      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // 토큰 구매
      await env.program.methods
        .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 사용자 포지션 확인
      const userPositionInfo =
        await env.program.account.userMarketPosition.fetch(userPosition);

      // 빈 인덱스 0만 추가되어야 함
      expect(userPositionInfo.bins.length).to.equal(1);
      expect(userPositionInfo.bins[0].index).to.equal(0);
      expect(userPositionInfo.bins[0].amount.toString()).to.equal(
        amounts[0].toString()
      );

      // 마켓 정보 확인
      const marketInfo = await env.program.account.market.fetch(env.market);

      // 빈 0만 업데이트되어야 함
      expect(marketInfo.bins[0].toString()).to.equal(amounts[0].toString());
      expect(marketInfo.bins[1].toString()).to.equal("0"); // 수량이 0으로 설정되어 있어야 함

      // 총 공급량은 실제 구매한 수량만 포함해야 함
      expect(marketInfo.tTotal.toString()).to.equal(amounts[0].toString());
    });

    it("추가 구매 시 기존 포지션에 올바르게 추가되어야 합니다", async () => {
      const binIndex = 0;
      const initialAmount = new BN(100_000_000_000);
      const additionalAmount = new BN(50_000_000_000);
      const maxCollateral = new BN(200_000_000_000);

      const userPosition = await env.getUserPosition(env.user1, env.marketId);

      // 첫 번째 구매
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [binIndex],
          [initialAmount],
          maxCollateral
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 첫 번째 구매 후 포지션 확인
      const positionAfterFirst =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(positionAfterFirst.bins.length).to.equal(1);
      expect(positionAfterFirst.bins[0].amount.toString()).to.equal(
        initialAmount.toString()
      );

      // 두 번째 구매 (같은 빈에 추가)
      await env.program.methods
        .buyTokens(
          new BN(env.marketId),
          [binIndex],
          [additionalAmount],
          maxCollateral
        )
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 두 번째 구매 후 포지션 확인
      const positionAfterSecond =
        await env.program.account.userMarketPosition.fetch(userPosition);
      expect(positionAfterSecond.bins.length).to.equal(1); // 여전히 1개의 빈
      expect(positionAfterSecond.bins[0].amount.toString()).to.equal(
        initialAmount.add(additionalAmount).toString()
      );

      // 마켓 정보 확인
      const marketInfo = await env.program.account.market.fetch(env.market);
      expect(marketInfo.bins[binIndex].toString()).to.equal(
        initialAmount.add(additionalAmount).toString()
      );
    });
  });

  describe("포지션 이전 (transferPosition)", () => {
    it("사용자가 포지션을 다른 지갑으로 이전할 수 있어야 합니다", async () => {
      // 먼저 user1이 포지션 생성
      const binIndices = [0, 1];
      const amounts = [new BN(100_000_000_000), new BN(200_000_000_000)];
      const maxCollateral = new BN(350_000_000_000);

      const user1Position = await env.getUserPosition(env.user1, env.marketId);

      // user1이 토큰 구매
      await env.program.methods
        .buyTokens(new BN(env.marketId), binIndices, amounts, maxCollateral)
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // user2 포지션 계정 생성
      const user2Position = await env.getUserPosition(env.user2, env.marketId);

      // 포지션 이전 (user1 -> user2)
      // 일부 포지션만 이전 (빈 0의 50%만 이전)
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

      // 이전 후 포지션 확인
      const user1PositionAfter =
        await env.program.account.userMarketPosition.fetch(user1Position);
      const user2PositionAfter =
        await env.program.account.userMarketPosition.fetch(user2Position);

      // user1 포지션 - bin 0은 50% 감소, bin 1은 그대로
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

      // user2 포지션 - bin 0에만 이전된 수량이 있어야 함
      expect(user2PositionAfter.bins.length).to.equal(1);
      expect(user2PositionAfter.bins[0].index).to.equal(binIndices[0]);
      expect(user2PositionAfter.bins[0].amount.toString()).to.equal(
        transferAmount.toString()
      );

      // 마켓의 총 수량은 변함없이 유지되어야 함
      const marketInfo = await env.program.account.market.fetch(env.market);
      expect(marketInfo.bins[binIndices[0]].toString()).to.equal(
        amounts[0].toString()
      );
      expect(marketInfo.bins[binIndices[1]].toString()).to.equal(
        amounts[1].toString()
      );
    });

    it("자신에게 이전하는 것은 실패해야 합니다", async () => {
      // 먼저 포지션 생성
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
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 자신에게 이전 시도
      try {
        await env.program.methods
          .transferPosition(new BN(env.marketId), [0], [new BN(50_000_000_000)])
          .accounts({
            fromUser: env.user1.publicKey,
            toUser: env.user1.publicKey, // 자신에게 이전
          })
          .signers([env.user1])
          .rpc();

        expect.fail("자신에게 이전하는 것은 실패해야 함");
      } catch (e) {
        expect(e.toString()).to.include("Cannot transfer to self");
      }
    });

    it("보유한 수량보다 많은 양을 이전하는 것은 실패해야 합니다", async () => {
      // 먼저 포지션 생성
      const user1Position = await env.getUserPosition(env.user1, env.marketId);
      const user2Position = await env.getUserPosition(env.user2, env.marketId);

      const amount = new BN(100_000_000_000);

      await env.program.methods
        .buyTokens(new BN(env.marketId), [0], [amount], new BN(150_000_000_000))
        .accounts({
          user: env.user1.publicKey,
          userTokenAccount: env.userTokenAccounts.user1, // 실제 사용자의 토큰 계정 사용
          vault: env.vault,
        })
        .signers([env.user1])
        .rpc();

      // 보유한 수량보다 많은 양을 이전 시도
      try {
        await env.program.methods
          .transferPosition(
            new BN(env.marketId),
            [0],
            [amount.add(new BN(10_000_000))] // 보유량보다 더 많음
          )
          .accounts({
            fromUser: env.user1.publicKey,
            toUser: env.user2.publicKey,
          })
          .signers([env.user1])
          .rpc();

        expect.fail("보유한 수량보다 많은 양을 이전하는 것은 실패해야 함");
      } catch (e) {
        expect(e.toString()).to.include("Insufficient balance");
      }
    });
  });
});
