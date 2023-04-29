import * as anchor from "@project-serum/anchor";
import { Program, BN } from "@project-serum/anchor";
import { NftClob } from "../target/types/nft_clob";
import { Keypair, LAMPORTS_PER_SOL, Transaction } from "@solana/web3.js";
import {
  createInitializeMintInstruction,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  getMinimumBalanceForRentExemptMint,
  createMint,
  transferChecked,
  mintToChecked,
} from "@solana/spl-token";

describe("nft-clob", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NftClob as Program<NftClob>;
  const authority = anchor.web3.Keypair.generate();

  let instrmtGrp: anchor.web3.PublicKey;
  let instrmt: anchor.web3.PublicKey;
  let baseMint: anchor.web3.PublicKey;
  let baseAta: anchor.web3.PublicKey;
  let quoteMint: anchor.web3.PublicKey;
  let quoteAta: anchor.web3.PublicKey;
  let baseVault: anchor.web3.PublicKey;
  let quoteVault: anchor.web3.PublicKey;

  const rbCrank = anchor.web3.Keypair.generate();
  const rbFilledExecReports = anchor.web3.Keypair.generate();
  const book = anchor.web3.Keypair.generate();

  console.log(program.programId.toBase58());
  before(async () => {
    const airdrop1 = await program.provider.connection.requestAirdrop(
      authority.publicKey,
      LAMPORTS_PER_SOL * 20
    );
    await program.provider.connection.confirmTransaction(airdrop1, "confirmed");

    [instrmtGrp] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("instrmt-grp"), authority.publicKey.toBuffer()],
      program.programId
    );

    [instrmt] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("instrmt"), book.publicKey.toBuffer()],
      program.programId
    );

    [baseVault] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("base-vault"), instrmt.toBuffer()],
      program.programId
    );

    [quoteVault] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("quote-vault"), instrmt.toBuffer()],
      program.programId
    );

    baseMint = await createMint(
      program.provider.connection, // conneciton
      authority, // fee payer
      authority.publicKey, // mint authority
      authority.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
      0 // decimals
    );

    baseAta = await createAssociatedTokenAccount(
      program.provider.connection, // connection
      authority, // fee payer
      baseMint, // mint
      authority.publicKey // owner,
    );

    let txhash = await mintToChecked(
      program.provider.connection, // connection
      authority, // fee payer
      baseMint, // mint
      baseAta, // receiver (sholud be a token account)
      authority, // mint authority
      100, // amount. if your decimals is 8, you mint 10^8 for 1 token.
      0 // decimals
    );

    quoteMint = await createMint(
      program.provider.connection, // conneciton
      authority, // fee payer
      authority.publicKey, // mint authority
      authority.publicKey, // freeze authority (you can use `null` to disable it. when you disable it, you can't turn it on again)
      8 // decimals
    );

    quoteAta = await createAssociatedTokenAccount(
      program.provider.connection, // connection
      authority, // fee payer
      quoteMint, // mint
      authority.publicKey // owner,
    );

    let txhash2 = await mintToChecked(
      program.provider.connection, // connection
      authority, // fee payer
      quoteMint, // mint
      quoteAta, // receiver (sholud be a token account)
      authority, // mint authority
      100_0000_0000, // amount. if your decimals is 8, you mint 10^8 for 1 token.
      8 // decimals
    );
  });

  it("should init instrmt_grp", async () => {
    const tx = await program.methods
      .newInstrmtGrp()
      .accounts({
        authority: authority.publicKey,
        instrmtGrp: instrmtGrp,
        rbCrank: rbCrank.publicKey,
      })
      .preInstructions([
        await program.account.ringBufferCrank.createInstruction(rbCrank),
      ])
      .signers([rbCrank, authority])
      .rpc();

    console.log(tx);
  });

  it("should init instrmt", async () => {
    const tx = await program.methods
      .newInstrmt({
        baseSymbol: "BASEBASE",
        quoteSymbol: "QUOTEQUOTE",
      })
      .accounts({
        authority: authority.publicKey,
        instrmtGrp: instrmtGrp,
        instrmt: instrmt,
        rbFilledExecReports: rbFilledExecReports.publicKey,
        book: book.publicKey,
        baseMint: baseMint,
        quoteMint: quoteMint,
        baseVault: baseVault,
        quoteVault: quoteVault,
      })
      .preInstructions([
        await program.account.book.createInstruction(book),
        await program.account.ringBufferFilledExecReport.createInstruction(
          rbFilledExecReports
        ),
      ])
      .signers([book, rbFilledExecReports, authority])
      .rpc();
    console.log(tx);
  });

  it("should place new order single", async () => {
    let orderType = { gtc: {} } as never;

    const tx = await program.methods
      .newOrderSingle({
        isBuy: true,
        limit: new BN(2),
        size: new BN(4),
        orderType: orderType,
      })
      .accounts({
        authority: authority.publicKey,
        instrmt: instrmt,
        instrmtGrp: instrmtGrp,
        rbFilledExecReports: rbFilledExecReports.publicKey,
        rbCrank: rbCrank.publicKey,
        baseVault: baseVault,
        quoteVault: quoteVault,
        baseUserTokenAccount: baseAta,
        quoteUserTokenAccount: quoteAta,
        book: book.publicKey,
      })
      .signers([authority])
      .rpc();
    console.log(tx);
  });
});
