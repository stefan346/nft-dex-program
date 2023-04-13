import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { NftClob } from "../target/types/nft_clob";

describe("nft-clob", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.NftClob as Program<NftClob>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
