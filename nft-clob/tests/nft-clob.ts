import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftClob } from "../target/types/nft_clob";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("nft-clob", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NftClob as Program<NftClob>;
  const authority = (program.provider as any).wallet
    .payer as anchor.web3.Keypair;

  // let instrmtGrp: anchor.web3.PublicKey;
  // const rbCrank = anchor.web3.Keypair.generate();
  // before(async () => {
  //   const airdrop1 = await program.provider.connection.requestAirdrop(
  //     authority.publicKey,
  //     LAMPORTS_PER_SOL
  //   );

  //   [instrmtGrp] = await anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("instrmt-grp"), authority.publicKey.toBuffer()],
  //     program.programId
  //   );
  // });

  // it("should init instrmt_grp", async () => {
  //   let tx = await program.methods
  //     .newInstrmtGrp()
  //     .accounts({
  //       authority: authority.publicKey,
  //       instrmtGrp: instrmtGrp,
  //       rbCrank: rbCrank.publicKey,
  //     })
  //     .preInstructions([
  //       await program.account.ringBufferCrank.createInstruction(rbCrank),
  //     ])
  //     .signers([rbCrank, authority])
  //     .rpc();
  //   console.log(tx);
  // });
});
