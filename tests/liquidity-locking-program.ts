import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LiquidityLockingProgram } from "../target/types/liquidity_locking_program";
import {
  Connection,
  Keypair,
  PublicKey,
  ComputeBudgetProgram,
  Transaction,
} from "@solana/web3.js";
import {
  createAssociatedTokenAccountInstruction,
  getAccount,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { config } from "dotenv";
import bs58 from "bs58";
import {
  CpAmm,
  derivePositionAddress,
  derivePositionNftAccount,
} from "@meteora-ag/cp-amm-sdk";

config({ path: "./tests/.env" });

describe("liquidity-locking-program", () => {
  // Define METEORA_PROGRAM_ID (or import from constants if exported)
  const METEORA_PROGRAM_ID = new PublicKey(
    "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG"
  );

  const userKeypair = Keypair.fromSecretKey(
    bs58.decode(process.env.USER_PRIVATE_KEY!)
  );
  const user = userKeypair.publicKey;

  // Mints
  const SLERF_MINT = new PublicKey(
    "9999FVbjHioTcoJpoBiSjpxHW6xEn3witVuXKqBh2RFQ"
  );
  const USDC_MINT = new PublicKey(
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
  );

  const connection = new Connection(process.env.NETWORK!);
  const cpAmm = new CpAmm(connection);

  // Configure the client to use the custom connection
  const wallet = new anchor.Wallet(userKeypair);
  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    anchor.AnchorProvider.defaultOptions()
  );
  anchor.setProvider(provider);

  const program = anchor.workspace
    .liquidityLockingProgram as Program<LiquidityLockingProgram>;

  const logTxnSignature = (tx: string) => {
    console.log(
      "Your transaction signature",
      `https://explorer.solana.com/tx/${tx}?cluster=surfnet` // Adjust for Surfpool explorer
    );
  };

  // RPC call helper for Surfpool
  const rpcCall = async (method: string, params: any) => {
    return fetch(program.provider.connection.rpcEndpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
      }),
    });
  };

  before(async () => {
    const lamports = 100 * 10 ** 9; // 100 SOL

    // Airdrop SOL
    await rpcCall("setAccount", [
      user,
      {
        lamports,
        owner: anchor.web3.SystemProgram.programId,
        data: Buffer.alloc(0),
      },
    ]).catch((err) => console.log("Error airdropping SOL", err));

    // Calculate rent-exempt lamports for Token accounts (165 bytes)
    const rentExempt = await connection.getMinimumBalanceForRentExemption(165);

    // Fund SLERF and USDC
    const slerfATA = await getAssociatedTokenAddress(SLERF_MINT, user);
    const usdcATA = await getAssociatedTokenAddress(USDC_MINT, user);

    const slerfAmount = 1000000 * 10 ** 9; // 1,000,000 SLERF (9 decimals)
    const usdcAmount = 1000 * 10 ** 6; // 1,000 USDC (6 decimals)

    // Create SLERF ATA if it doesn't exist
    try {
      await getAccount(connection, slerfATA);
    } catch {
      const createSlerfIx = createAssociatedTokenAccountInstruction(
        user, // payer
        slerfATA,
        user, // owner
        SLERF_MINT
      );
      const tx = new Transaction().add(createSlerfIx);
      await program.provider.sendAndConfirm(tx, [userKeypair]);
    }

    // Create USDC ATA if it doesn't exist
    try {
      await getAccount(connection, usdcATA);
    } catch {
      const createUsdcIx = createAssociatedTokenAccountInstruction(
        user, // payer
        usdcATA,
        user, // owner
        USDC_MINT
      );
      const tx = new Transaction().add(createUsdcIx);
      await program.provider.sendAndConfirm(tx, [userKeypair]);
    }

    // Set SLERF balance
    await rpcCall("setAccount", [
      slerfATA,
      {
        lamports: rentExempt,
        data: {
          mint: SLERF_MINT,
          owner: user,
          amount: slerfAmount,
          delegate: null,
          state: 1,
          isNative: null,
          delegatedAmount: 0,
          closeAuthority: null,
        },
        executable: false,
        owner: TOKEN_PROGRAM_ID,
      },
    ]).catch((err) => console.log("Error setting SLERF balance", err));

    // Set USDC balance
    await rpcCall("setAccount", [
      usdcATA,
      {
        lamports: rentExempt,
        data: {
          mint: USDC_MINT,
          owner: user,
          amount: usdcAmount,
          delegate: null,
          state: 1,
          isNative: null,
          delegatedAmount: 0,
          closeAuthority: null,
        },
        executable: false,
        owner: TOKEN_PROGRAM_ID,
      },
    ]).catch((err) => console.log("Error setting USDC balance", err));

    console.log("User funded with SOL, SLERF, and USDC");
  });

  it("Lock Liquidity", async () => {
    // Derive user token accounts (ATAs)
    const userTokenAAccount = await getAssociatedTokenAddress(USDC_MINT, user); // USDC
    const userTokenBAccount = await getAssociatedTokenAddress(SLERF_MINT, user); // SLERF

    // Pool and mints (essentials to pass)
    const pool = new PublicKey("8yswq8vqEDeTrN2Ez1Bdq2hRekzvFZgMxrdfUKVaNBtQ"); // SLERF-USDC pool
    const tokenAMint = USDC_MINT;
    const tokenBMint = SLERF_MINT;

    // Fetch pool account to get vault addresses
    const poolAccount = await cpAmm.fetchPoolState(pool);
    const tokenAVault = poolAccount.tokenAVault;
    const tokenBVault = poolAccount.tokenBVault;

    const durationMonths = 3; // User selects 3 months

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 800_000, // Higher for CPIs
    });

    // Generate the position NFT mint as a keypair (not a PDA)
    const positionNftMint = Keypair.generate();
    console.log("Position NFT Mint:", positionNftMint.publicKey.toBase58());

    // Use SDK to derive position address (correct seeds)
    const position = derivePositionAddress(positionNftMint.publicKey);

    // FIXED: position_nft_account is an ATA for the position NFT mint (Token2022)
    const positionNftAccount = derivePositionNftAccount(
      positionNftMint.publicKey
    );

    // Vesting must be a Keypair signer
    const vesting = Keypair.generate();
    console.log("Vesting Account:", vesting.publicKey.toBase58());

    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("__event_authority")],
      METEORA_PROGRAM_ID
    );

    // Pass all required accounts, including system programs
    const tx = await program.methods
      .dammV2LockLiquidity(durationMonths)
      .accounts({
        pool,
        userTokenAAccount,
        userTokenBAccount,
        tokenAMint,
        tokenBMint,
        tokenAVault,
        tokenBVault,
        positionNftMint: positionNftMint.publicKey,
        positionNftAccount,
        position,
        vesting: vesting.publicKey, // Use the keypair's public key
        payer: user,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        tokenAProgram: TOKEN_PROGRAM_ID,
        tokenBProgram: TOKEN_PROGRAM_ID,
        eventAuthority,
        dammProgram: METEORA_PROGRAM_ID,
      })
      .preInstructions([computeUnitIx])
      .signers([userKeypair, positionNftMint, vesting]) // Add vesting as a signer
      .rpc();

    logTxnSignature(tx);
  });
});
