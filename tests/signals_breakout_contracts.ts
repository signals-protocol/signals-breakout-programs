import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";

describe("range_bet_program", () => {
  // 클라이언트 설정
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.range_bet_program as Program;
  const wallet = provider.wallet as anchor.Wallet;

  // 테스트용 변수들
  let programStatePda: PublicKey;
  let programStateBump: number;
  let marketPda: PublicKey;
  let marketBump: number;
  let collateralMint: PublicKey;
  let vaultAuthority: PublicKey;
  let vaultAuthBump: number;
  let userTokenAccount: PublicKey;
  let vaultTokenAccount: PublicKey;

  before(async () => {
    // 프로그램 상태 PDA 찾기
    [programStatePda, programStateBump] = await PublicKey.findProgramAddress(
      [Buffer.from("range-bet-state")],
      program.programId
    );

    // 코인 Mint 생성 (테스트 담보 토큰)
    const mintAuthority = Keypair.generate();
    await provider.connection.requestAirdrop(
      mintAuthority.publicKey,
      1000000000
    );

    // 테스트 토큰 생성
    collateralMint = await createMint(
      provider.connection,
      wallet.payer,
      mintAuthority.publicKey,
      null,
      6
    );

    // 사용자 토큰 계정 생성
    userTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      collateralMint,
      wallet.publicKey
    );

    // 사용자에게 토큰 발행
    await mintTo(
      provider.connection,
      wallet.payer,
      collateralMint,
      userTokenAccount,
      mintAuthority,
      1000000000
    );
  });

  it("프로그램 초기화", async () => {
    // 프로그램 초기화 트랜잭션 실행
    await program.methods
      .initializeProgram()
      .accounts({
        initializer: wallet.publicKey,
        programState: programStatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // 상태 확인
    const programState = await program.account.programState.fetch(
      programStatePda
    );
    assert.equal(programState.owner.toString(), wallet.publicKey.toString());
    assert.equal(programState.marketCount.toString(), "0");
    assert.equal(programState.lastClosedMarket.toString(), "-1");
  });

  it("마켓 생성", async () => {
    // 마켓 PDA 찾기
    [marketPda, marketBump] = await PublicKey.findProgramAddress(
      [Buffer.from("market"), Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])],
      program.programId
    );

    // Vault 권한 PDA 찾기
    [vaultAuthority, vaultAuthBump] = await PublicKey.findProgramAddress(
      [Buffer.from("vault"), Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])],
      program.programId
    );

    // Vault 토큰 계정 찾기
    vaultTokenAccount = await getAssociatedTokenAddress(
      collateralMint,
      vaultAuthority,
      true
    );

    // 마켓 생성 파라미터
    const tickSpacing = 60;
    const minTick = new BN(-360);
    const maxTick = new BN(360);
    const closeTs = new BN(Math.floor(Date.now() / 1000) + 86400); // 하루 후

    // 마켓 생성 트랜잭션 실행
    await program.methods
      .createMarket(tickSpacing, minTick, maxTick, closeTs)
      .accounts({
        owner: wallet.publicKey,
        programState: programStatePda,
        market: marketPda,
        collateralMint: collateralMint,
        vault: vaultTokenAccount,
        vaultAuthority: vaultAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    // 상태 확인
    const market = await program.account.market.fetch(marketPda);
    assert.isTrue(market.active);
    assert.isFalse(market.closed);
    assert.equal(market.tickSpacing, tickSpacing);
    assert.equal(market.minTick.toString(), minTick.toString());
    assert.equal(market.maxTick.toString(), maxTick.toString());
    assert.equal(market.tTotal.toString(), "0");
    assert.equal(market.collateralBalance.toString(), "0");
    assert.equal(market.winningBin.toString(), "0");
    assert.approximately(
      market.closeTs.toNumber(),
      closeTs.toNumber(),
      10 // 약간의 차이 허용
    );
    assert.isAbove(market.openTs.toNumber(), 0);
  });

  // 추가 테스트 케이스들...
  // 토큰 구매, 시장 마감, 보상 청구 등
});
