import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralTokenFaucet } from "../target/types/collateral_token_faucet";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  getAccount,
  AuthorityType,
  setAuthority,
} from "@solana/spl-token";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("collateral_token_faucet", () => {
  // Provider와 Connection 설정
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 프로그램 가져오기
  const program = anchor.workspace
    .CollateralTokenFaucet as Program<CollateralTokenFaucet>;

  // 테스트용 계정들
  let mintAuthority: Keypair;
  let payer: Keypair; // 테스트 트랜잭션 지불자 및 토큰 수령자
  let mint: PublicKey; // 테스트용 SPL 토큰의 Mint 주소
  let faucetPda: PublicKey; // 프로그램의 PDA 주소
  let bump: number; // PDA bump seed
  let receiverAta: PublicKey; // Payer의 ATA 주소

  const MINT_AMOUNT = 10_000_000; // 10 토큰 (decimals 6 가정)

  before(async () => {
    console.log(`Program ID: ${program.programId}`);

    // 테스트에 필요한 키페어 생성
    mintAuthority = Keypair.generate();
    payer = Keypair.generate();

    console.log(`Payer: ${payer.publicKey}`);
    console.log(`Mint Authority: ${mintAuthority.publicKey}`);

    // Payer에게 SOL 에어드랍 (테스트넷/로컬넷에서만 가능)
    const signature = await provider.connection.requestAirdrop(
      payer.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature);

    // mintAuthority에게도 SOL 에어드랍 (토큰 생성 및 관리 목적)
    const signature2 = await provider.connection.requestAirdrop(
      mintAuthority.publicKey,
      1 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(signature2);

    console.log("SOL airdropped to test accounts");

    // 테스트용 SPL Mint 생성
    mint = await createMint(
      provider.connection,
      payer, // Mint 생성 비용 지불자
      mintAuthority.publicKey, // Mint Authority (초기에는 mintAuthority 키페어)
      null, // Freeze Authority (없음)
      6 // Decimals
    );
    console.log(`Test mint created: ${mint.toBase58()}`);

    // Faucet PDA 주소 및 bump 찾기
    [faucetPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_faucet")],
      program.programId
    );
    console.log(`Faucet PDA: ${faucetPda.toBase58()} (bump: ${bump})`);

    // Payer의 ATA 생성
    receiverAta = await createAssociatedTokenAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey
    );
    console.log(`Receiver ATA: ${receiverAta.toBase58()}`);

    // 중요: 생성된 SPL 토큰의 Mint Authority를 Faucet PDA로 이전
    await setAuthority(
      provider.connection,
      payer,
      mint,
      mintAuthority, // MintTokens 권한을 가진 현재 mint authority
      AuthorityType.MintTokens,
      faucetPda // 새로운 mint authority (PDA)
    );
    console.log(
      `Mint authority for ${mint.toBase58()} transferred to PDA: ${faucetPda.toBase58()}`
    );
  });

  it("Mints collateral tokens to the user", async () => {
    // 초기 ATA 잔액 확인 (0이어야 함)
    let receiverAccountInfo = await getAccount(
      provider.connection,
      receiverAta
    );
    console.log(`Initial token balance: ${receiverAccountInfo.amount}`);
    expect(receiverAccountInfo.amount.toString()).to.equal("0");

    // mint_collateral_tokens 호출
    const tx = await program.methods
      .mintCollateralTokens(new anchor.BN(MINT_AMOUNT))
      .accounts({
        mint: mint,
        user: payer.publicKey,
      })
      .signers([payer]) // payer가 instruction 실행 및 ATA 생성 비용 지불
      .rpc();

    console.log(`Transaction signature: ${tx}`);

    // 민팅 후 ATA 잔액 확인
    receiverAccountInfo = await getAccount(provider.connection, receiverAta);
    console.log(`Final token balance: ${receiverAccountInfo.amount}`);

    // 지정한 민팅 금액 확인 (10_000_000 = 10 토큰 with 6 decimals)
    expect(receiverAccountInfo.amount.toString()).to.equal(
      MINT_AMOUNT.toString()
    );
    console.log(
      `Successfully minted ${
        MINT_AMOUNT / 10 ** 6
      } tokens to ${receiverAta.toBase58()}`
    );
  });

  it("Mints custom amount of collateral tokens to the user", async () => {
    const customAmount = 5_000_000; // 5 토큰 (decimals 6 가정)

    // 현재 ATA 잔액 확인
    let initialBalance = (await getAccount(provider.connection, receiverAta))
      .amount;
    console.log(`Initial token balance: ${initialBalance}`);

    // mint_collateral_tokens 호출 (사용자 지정 금액)
    const tx = await program.methods
      .mintCollateralTokens(new anchor.BN(customAmount))
      .accounts({
        mint: mint,
        user: payer.publicKey,
      })
      .signers([payer])
      .rpc();

    console.log(`Transaction signature: ${tx}`);

    // 민팅 후 ATA 잔액 확인
    const receiverAccountInfo = await getAccount(
      provider.connection,
      receiverAta
    );
    console.log(`Final token balance: ${receiverAccountInfo.amount}`);

    // 기존 잔액 + 사용자 지정 금액만큼 증가했는지 확인
    const expectedBalance =
      BigInt(initialBalance.toString()) + BigInt(customAmount);
    expect(receiverAccountInfo.amount.toString()).to.equal(
      expectedBalance.toString()
    );
    console.log(
      `Successfully minted additional ${
        customAmount / 10 ** 6
      } tokens to ${receiverAta.toBase58()}`
    );
  });
});
