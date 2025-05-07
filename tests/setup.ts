import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SystemProgram, Keypair, Connection } from "@solana/web3.js";
import { RangeBetProgram } from "../target/types/range_bet_program";
import { BN } from "bn.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
  mintTo,
  createAssociatedTokenAccountInstruction,
  getAccount,
  createAccount,
} from "@solana/spl-token";

export interface TestEnv {
  provider: anchor.AnchorProvider;
  program: Program<RangeBetProgram>;
  admin: any;
  user1: Keypair;
  user2: Keypair;
  user3: Keypair;
  user4: Keypair;
  user5: Keypair;
  programState: anchor.web3.PublicKey;
  market: anchor.web3.PublicKey;
  collateralMint: anchor.web3.PublicKey;
  vault: anchor.web3.PublicKey;
  vaultAuthority: anchor.web3.PublicKey;
  vaultAuthorityBump: number;
  marketId: number;
  tickSpacing: number;
  minTick: number;
  maxTick: number;
  closeTime: typeof BN.prototype;
  getUserPosition: (
    user: Keypair,
    marketId: number
  ) => Promise<anchor.web3.PublicKey>;
  userTokenAccounts: {
    admin: anchor.web3.PublicKey;
    user1: anchor.web3.PublicKey;
    user2: anchor.web3.PublicKey;
    user3: anchor.web3.PublicKey;
    user4: anchor.web3.PublicKey;
    user5: anchor.web3.PublicKey;
  };
  // 새로운 기능: 시장 리셋 및 효율적인 테스트 환경 관리
  resetMarket: () => Promise<void>;
  createNewMarket: (params?: {
    tickSpacing?: number;
    minTick?: number;
    maxTick?: number;
    closeTime?: number;
  }) => Promise<{
    market: anchor.web3.PublicKey;
    marketId: number;
  }>;
  replenishTokens: (user: Keypair, amount?: number) => Promise<void>;
  closeMarketsSequentially: (
    targetMarketId: number,
    winningBin: number
  ) => Promise<void>;
}

/**
 * 전체 테스트 환경 구성 - 처음 한 번만 호출하는 것이 효율적
 */
export async function setupTestEnvironment(): Promise<TestEnv> {
  console.log("🔄 전체 테스트 환경 구성 시작...");

  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RangeBetProgram as Program<RangeBetProgram>;

  // 관리자(프로그램 소유자) 키페어
  const admin = provider.wallet;

  // 테스트 유저 생성
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();
  const user4 = Keypair.generate();
  const user5 = Keypair.generate();

  console.log("💰 테스트 유저에게 SOL 에어드롭 중...");

  // 테스트 유저에게 SOL 에어드롭 (여유있게 10 SOL)
  for (const user of [user1, user2, user3, user4, user5]) {
    const airdropSig = await provider.connection.requestAirdrop(
      user.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);
  }

  // 프로그램 상태 계정
  const [programState, programStateBump] =
    await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("range-bet-state")],
      program.programId
    );

  // 프로그램 초기화 - 아직 초기화되지 않았을 경우
  try {
    await program.account.programState.fetch(programState);
    console.log("✅ 프로그램 상태 이미 초기화됨");
  } catch (e) {
    // 프로그램 상태가 없으면 초기화
    console.log("🔨 프로그램 상태 초기화 중...");
    await program.methods
      .initializeProgram()
      .accounts({
        initializer: admin.publicKey,
      })
      .rpc();
    console.log("✅ 프로그램 상태 초기화 완료");
  }

  console.log("💲 담보 토큰 Mint 생성 중...");

  // 담보 토큰 Mint 생성
  const collateralMint = await createMint(
    provider.connection,
    admin.payer,
    admin.publicKey,
    null,
    9 // 9 decimals
  );

  console.log("💳 각 사용자의 토큰 계정(ATA) 생성 중...");

  // 각 사용자의 토큰 계정(ATA) 생성
  const adminTokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    admin.publicKey
  );

  const user1TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user1.publicKey
  );

  const user2TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user2.publicKey
  );

  const user3TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user3.publicKey
  );

  const user4TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user4.publicKey
  );

  const user5TokenAccount = await createAssociatedTokenAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    user5.publicKey
  );

  console.log("💵 각 사용자에게 토큰 민팅 중...");

  // 각 사용자에게 토큰 민팅
  const mintAmount = 10000_000_000_000; // 10,000 tokens (넉넉하게)

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    adminTokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user1TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user2TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user3TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user4TokenAccount,
    admin.publicKey,
    mintAmount
  );

  await mintTo(
    provider.connection,
    admin.payer,
    collateralMint,
    user5TokenAccount,
    admin.publicKey,
    mintAmount
  );

  // 마켓 생성에 필요한 기본값 정의
  const tickSpacing = 60;
  const minTick = -360;
  const maxTick = 360;
  const closeTime = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60; // 일주일 후

  // 첫 번째 마켓 ID (0)에 대한 vault authority PDA 계산
  let marketId = 0;
  const [vaultAuthority, vaultAuthorityBump] =
    await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), new BN(marketId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

  console.log("🏦 Vault Authority PDA 계산 완료:", vaultAuthority.toString());

  // PDA가 소유자인 토큰 계정을 생성 (관리자가 지불)
  // 여기서는 계정만 생성하고, 자금은 사용자가 buyTokens를 통해 채움
  const vault = await createAccount(
    provider.connection,
    admin.payer,
    collateralMint,
    vaultAuthority, // PDA가 소유자
    Keypair.generate() // 새 계정 키페어 생성
  );

  console.log("🏦 Vault 계정 설정 완료:", vault.toString());

  // 마켓 계정 주소 (PDA) 계산
  const [market, marketBump] =
    await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(marketId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

  // 마켓 생성 (최초 한 번)
  async function createMarketIfNeeded() {
    try {
      // 이미 마켓이 있는지 확인
      await program.account.market.fetch(market);
      console.log("✅ 마켓 ID", marketId, "이미 존재합니다.");
      return false;
    } catch (e) {
      // 마켓이 없으면 생성
      console.log("🔨 마켓 ID", marketId, "생성 중...");
      await program.methods
        .createMarket(
          tickSpacing,
          new BN(minTick),
          new BN(maxTick),
          new BN(closeTime)
        )
        .accounts({
          owner: admin.publicKey,
          collateralMint: collateralMint,
        })
        .rpc();
      console.log("✅ 마켓 ID", marketId, "생성 완료!");
      return true;
    }
  }

  await createMarketIfNeeded();

  // 유저 포지션 계정 주소 (PDA) 계산
  async function getUserPosition(user: Keypair, marketId: number) {
    const [userPosition, userPositionBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("pos"),
          user.publicKey.toBuffer(),
          new BN(marketId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    return userPosition;
  }

  // 마켓 리셋 함수 (새로운 마켓을 생성하여 깨끗한 테스트 환경 제공)
  async function resetMarketInternal() {
    try {
      // 기존 마켓을 닫지 않고 새 마켓을 생성
      // 이 방식은 프로그램의 마켓 종료 순서 제약을 우회합니다
      console.log("🔄 새 테스트 마켓 생성 중...");
      const {
        market: newMarket,
        marketId: newMarketId,
        vault: newVault,
        vaultAuthority: newVaultAuthority,
        vaultAuthorityBump: newVaultAuthorityBump,
      } = await createNewMarket();

      // 반환할 객체에 설정할 수 있도록 새 값 저장
      updatedMarket = newMarket;
      updatedMarketId = newMarketId;
      updatedVault = newVault;
      updatedVaultAuthority = newVaultAuthority;
      updatedVaultAuthorityBump = newVaultAuthorityBump;
      console.log("✅ 새 마켓 ID", newMarketId, "생성 완료 (테스트용)");
    } catch (e) {
      console.log("⚠️ 새 마켓 생성 중 오류 발생:", e.message);
    }
  }

  // 마켓 업데이트를 위한 임시 변수
  let updatedMarket = market;
  let updatedMarketId = marketId;
  let updatedVault = vault;
  let updatedVaultAuthority = vaultAuthority;
  let updatedVaultAuthorityBump = vaultAuthorityBump;

  // 새 마켓 생성 함수
  async function createNewMarket(params?: {
    tickSpacing?: number;
    minTick?: number;
    maxTick?: number;
    closeTime?: number;
  }) {
    // 프로그램 상태에서 현재 마켓 카운트 가져오기
    const state = await program.account.programState.fetch(programState);
    const newMarketId = state.marketCount.toNumber();

    // 새 마켓 계정 주소 계산
    const [newMarket] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("market"), new BN(newMarketId).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // 새 마켓에 대한 vault authority 계산
    const [newVaultAuthority, newVaultAuthorityBump] =
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("vault"),
          new BN(newMarketId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    // 새 마켓용 토큰 vault 생성
    const newVault = await createAccount(
      provider.connection,
      admin.payer,
      collateralMint,
      newVaultAuthority,
      Keypair.generate()
    );

    console.log(
      `🏦 새 마켓 ID ${newMarketId}의 Vault 계정 설정:`,
      newVault.toString()
    );

    // 새 마켓 생성
    console.log("🔨 새 마켓 ID", newMarketId, "생성 중...");
    await program.methods
      .createMarket(
        params?.tickSpacing ?? tickSpacing,
        new BN(params?.minTick ?? minTick),
        new BN(params?.maxTick ?? maxTick),
        new BN(params?.closeTime ?? closeTime)
      )
      .accounts({
        owner: admin.publicKey,
        collateralMint: collateralMint,
      })
      .rpc();
    console.log("✅ 새 마켓 ID", newMarketId, "생성 완료!");

    // 업데이트된 값들 저장
    updatedVault = newVault;
    updatedVaultAuthority = newVaultAuthority;
    updatedVaultAuthorityBump = newVaultAuthorityBump;

    return {
      market: newMarket,
      marketId: newMarketId,
      vault: newVault,
      vaultAuthority: newVaultAuthority,
      vaultAuthorityBump: newVaultAuthorityBump,
    };
  }

  // 토큰 보충 함수
  async function replenishTokens(user: Keypair, amount = mintAmount) {
    // 토큰 계정 찾기
    let tokenAccount;
    if (user.publicKey.equals(admin.publicKey)) {
      tokenAccount = adminTokenAccount;
    } else if (user.publicKey.equals(user1.publicKey)) {
      tokenAccount = user1TokenAccount;
    } else if (user.publicKey.equals(user2.publicKey)) {
      tokenAccount = user2TokenAccount;
    } else if (user.publicKey.equals(user3.publicKey)) {
      tokenAccount = user3TokenAccount;
    } else if (user.publicKey.equals(user4.publicKey)) {
      tokenAccount = user4TokenAccount;
    } else if (user.publicKey.equals(user5.publicKey)) {
      tokenAccount = user5TokenAccount;
    } else {
      throw new Error("알 수 없는 사용자입니다");
    }

    // 토큰 민팅
    await mintTo(
      provider.connection,
      admin.payer,
      collateralMint,
      tokenAccount,
      admin.publicKey,
      amount
    );
  }

  console.log("🎉 전체 테스트 환경 구성 완료!");

  // 테스트 환경 객체 구성
  const testEnv: TestEnv = {
    provider,
    program,
    admin,
    user1,
    user2,
    user3,
    user4,
    user5,
    programState,
    market: updatedMarket,
    collateralMint,
    vault: updatedVault,
    vaultAuthority: updatedVaultAuthority,
    vaultAuthorityBump: updatedVaultAuthorityBump,
    marketId: updatedMarketId,
    tickSpacing,
    minTick,
    maxTick,
    closeTime: new BN(closeTime),
    getUserPosition,
    userTokenAccounts: {
      admin: adminTokenAccount,
      user1: user1TokenAccount,
      user2: user2TokenAccount,
      user3: user3TokenAccount,
      user4: user4TokenAccount,
      user5: user5TokenAccount,
    },
    resetMarket: async () => {
      await resetMarketInternal();
      // resetMarket 내에서 업데이트한 값으로 객체 속성 갱신
      testEnv.market = updatedMarket;
      testEnv.marketId = updatedMarketId;
      testEnv.vault = updatedVault;
      testEnv.vaultAuthority = updatedVaultAuthority;
      testEnv.vaultAuthorityBump = updatedVaultAuthorityBump;
    },
    createNewMarket,
    replenishTokens,
    closeMarketsSequentially: async (
      targetMarketId: number,
      winningBin: number = 0
    ) => {
      // 프로그램 상태 조회하여 last_closed_market 값 확인
      const state = await program.account.programState.fetch(programState);
      let lastClosed = state.lastClosedMarket
        ? state.lastClosedMarket.toNumber()
        : -1;

      // 마켓 ID가 순차적으로 닫혀야 하므로
      // last_closed_market+1부터 target 마켓까지 순서대로 닫기
      for (let id = lastClosed + 1; id <= targetMarketId; id++) {
        try {
          // 마켓 계정 주소 계산
          const [marketToClose] =
            await anchor.web3.PublicKey.findProgramAddressSync(
              [Buffer.from("market"), new BN(id).toArrayLike(Buffer, "le", 8)],
              program.programId
            );

          console.log(`마켓 ID ${id} 닫는 중...`);

          // 마켓 정보 확인
          try {
            const marketInfo = await program.account.market.fetch(
              marketToClose
            );

            // 이미 닫힌 마켓은 건너뛰기
            if (marketInfo.closed) {
              console.log(`마켓 ID ${id}는 이미 닫혀 있습니다.`);
              continue;
            }

            // 마켓 닫기
            const closeBin = id === targetMarketId ? winningBin : 0; // 타겟 마켓만 지정된 winning bin으로 설정
            await program.methods
              .closeMarket(new BN(id), closeBin)
              .accounts({
                owner: admin.publicKey,
              })
              .rpc();

            console.log(`마켓 ID ${id} 성공적으로 닫힘.`);
          } catch (e) {
            // 마켓이 존재하지 않으면 warning 만 표시하고 계속 진행
            console.log(
              `마켓 ID ${id}가 존재하지 않거나 처리 중 오류 발생: ${e.message}`
            );
          }
        } catch (e) {
          console.error(`마켓 ID ${id} 닫기 실패: ${e.message}`);
          throw e;
        }
      }
    },
  };

  return testEnv;
}
