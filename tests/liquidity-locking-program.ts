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
import BN from "bn.js";

const dammIdl = require("../idls/damm_v2.json"); // path from /tests to /idls

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

  // Shared variables for sequential tests
  let positionNftMint: Keypair;

  before(async () => {
    const lamports = 100 * 10 ** 9; // 100 SOL

    // Airdrop SOL using surfnet_setAccount
    await rpcCall("surfnet_setAccount", [user.toBase58(), { lamports }]).catch(
      (err) => console.log("Error airdropping SOL", err)
    );

    const slerfAmount = 10000000 * 10 ** 9; // 10,000,000 SLERF (9 decimals)
    const usdcAmount = 10000 * 10 ** 6; // 10,000 USDC (6 decimals)

    // Set SLERF balance using surfnet_setTokenAccount (matching example format)
    await rpcCall("surfnet_setTokenAccount", [
      user.toBase58(), // owner
      SLERF_MINT.toBase58(), // mint
      { amount: slerfAmount }, // minimal update
      TOKEN_PROGRAM_ID.toBase58(), // tokenProgram
    ]).catch((err) => console.log("Error setting SLERF balance", err));

    // Set USDC balance using surfnet_setTokenAccount (matching example format)
    await rpcCall("surfnet_setTokenAccount", [
      user.toBase58(), // owner
      USDC_MINT.toBase58(), // mint
      { amount: usdcAmount }, // minimal update
      TOKEN_PROGRAM_ID.toBase58(), // tokenProgram
    ]).catch((err) => console.log("Error setting USDC balance", err));

    console.log("User funded with SOL, SLERF, and USDC");
  });

  it("Create Position", async () => {
    const pool = new PublicKey("8yswq8vqEDeTrN2Ez1Bdq2hRekzvFZgMxrdfUKVaNBtQ"); // SLERF-USDC pool

    // Generate the position NFT mint as a keypair (not a PDA)
    positionNftMint = Keypair.generate(); // Assign to shared variable
    console.log("Position NFT Mint:", positionNftMint.publicKey.toBase58());

    // Use SDK to derive position address (correct seeds)
    const position = derivePositionAddress(positionNftMint.publicKey);

    // Position NFT account is an ATA for the position NFT mint (Token2022)
    const positionNftAccount = derivePositionNftAccount(
      positionNftMint.publicKey
    );

    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("__event_authority")],
      METEORA_PROGRAM_ID
    );

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });

    const tx = await program.methods
      .createPositionIx()
      .accounts({
        owner: user, // #1
        positionNftMint: positionNftMint.publicKey, // #2
        positionNftAccount, // #3
        pool, // #4
        position, // #5
        poolAuthority: new PublicKey(
          "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC"
        ), // #6
        payer: user, // #7
        tokenProgram: TOKEN_2022_PROGRAM_ID, // #8
        eventAuthority, // #9
        dammProgram: METEORA_PROGRAM_ID, // #10
      })
      .preInstructions([computeUnitIx])
      .signers([userKeypair, positionNftMint])
      .rpc();

    logTxnSignature(tx);
  });

  it("Add Liquidity", async () => {
    const pool = new PublicKey("8yswq8vqEDeTrN2Ez1Bdq2hRekzvFZgMxrdfUKVaNBtQ"); // SLERF-USDC pool

    // Choose liquidity delta (user-configurable)
    const liquidityDelta = new BN(100); // Change this value as needed

    // Fetch pool state to get vaults and mints
    const poolState = await cpAmm.fetchPoolState(pool);
    const tokenAMint = poolState.tokenAMint;
    const tokenBMint = poolState.tokenBMint;
    const tokenAVault = poolState.tokenAVault;
    const tokenBVault = poolState.tokenBVault;

    // User's token accounts (ATAs)
    const tokenAAccount = await getAssociatedTokenAddress(
      tokenAMint,
      user,
      false,
      TOKEN_PROGRAM_ID
    ); // ATA for token A
    const tokenBAccount = await getAssociatedTokenAddress(
      tokenBMint,
      user,
      false,
      TOKEN_PROGRAM_ID
    ); // ATA for token B

    // Log token assignments and balances for debugging
    console.log("Token A Mint:", tokenAMint.toBase58());
    console.log("Token B Mint:", tokenBMint.toBase58());
    console.log("Token A Account:", tokenAAccount.toBase58());
    console.log("Token B Account:", tokenBAccount.toBase58());

    try {
      const tokenABalance = await connection.getTokenAccountBalance(
        tokenAAccount
      );
      console.log(
        "Token A Balance:",
        tokenABalance.value.uiAmount,
        tokenABalance.value.amount
      );
    } catch (err) {
      console.log("Error fetching Token A balance:", err);
    }

    try {
      const tokenBBalance = await connection.getTokenAccountBalance(
        tokenBAccount
      );
      console.log(
        "Token B Balance:",
        tokenBBalance.value.uiAmount,
        tokenBBalance.value.amount
      );
    } catch (err) {
      console.log("Error fetching Token B balance:", err);
    }

    // Derive position and position NFT account from shared positionNftMint
    const position = derivePositionAddress(positionNftMint.publicKey);
    const positionNftAccount = derivePositionNftAccount(
      positionNftMint.publicKey
    );

    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("__event_authority")],
      METEORA_PROGRAM_ID
    );

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });

    const tx = await program.methods
      .addLiquidityIx(liquidityDelta)
      .accounts({
        pool, // #1
        position, // #2
        tokenAAccount, // #3
        tokenBAccount, // #4
        tokenAVault, // #5
        tokenBVault, // #6
        tokenAMint, // #7
        tokenBMint, // #8
        positionNftAccount, // #9
        owner: user, // #10
        tokenAProgram: TOKEN_PROGRAM_ID, // #11
        tokenBProgram: TOKEN_PROGRAM_ID, // #12
        eventAuthority, // #13
        dammProgram: METEORA_PROGRAM_ID, // #14
      })
      .preInstructions([computeUnitIx])
      .signers([userKeypair])
      .rpc();

    logTxnSignature(tx);
  });

  it("Lock Position", async () => {
    const pool = new PublicKey("8yswq8vqEDeTrN2Ez1Bdq2hRekzvFZgMxrdfUKVaNBtQ");

    const durationMonths = 3;

    const position = derivePositionAddress(positionNftMint.publicKey);
    const positionNftAccount = derivePositionNftAccount(
      positionNftMint.publicKey
    );

    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("__event_authority")],
      METEORA_PROGRAM_ID
    );

    const positionData = await cpAmm.fetchPositionState(position);

    // CHANGED: cpAmm position state has unlockedLiquidity/vestedLiquidity/etc, not `liquidity`
    const totalLiquidity = new BN(positionData.unlockedLiquidity.toString());

    // Match the successful tx invariants:
    // cliff_unlock_liquidity + liquidity_per_period * number_of_period == totalLiquidity
    const cliffUnlockLiquidity = new BN(1);
    if (totalLiquidity.lte(cliffUnlockLiquidity)) {
      throw new Error(
        `Position liquidity too small to lock: ${totalLiquidity.toString()}`
      );
    }

    const periodFrequency = 2_628_000; // ~1 month, matches successful tx
    const now = Math.floor(Date.now() / 1000);

    // end time = now + duration
    const end = now + periodFrequency * durationMonths;

    // With numberOfPeriod=1, Meteora ends at cliffPoint + periodFrequency
    const cliffPoint = end - periodFrequency;

    const vestingParams = {
      cliffPoint: new BN(cliffPoint), // Option<u64> (Some)
      periodFrequency: new BN(periodFrequency), // u64
      cliffUnlockLiquidity, // u128
      liquidityPerPeriod: totalLiquidity.sub(cliffUnlockLiquidity), // u128
      numberOfPeriod: 1, // u16
    };

    // Optional debug to compare vs explorer example
    console.log("totalLiquidity", totalLiquidity.toString());
    console.log("vestingParams", {
      cliffPoint,
      periodFrequency,
      cliffUnlockLiquidity: cliffUnlockLiquidity.toString(),
      liquidityPerPeriod: totalLiquidity.sub(cliffUnlockLiquidity).toString(),
      numberOfPeriod: 1,
      end,
    });

    const vesting = Keypair.generate();
    console.log("Vesting (new signer):", vesting.publicKey.toBase58());

    const computeUnitIx = ComputeBudgetProgram.setComputeUnitLimit({
      units: 400_000,
    });

    const tx = await program.methods
      .lockPositionIx(vestingParams)
      .accounts({
        pool,
        position,
        vesting: vesting.publicKey,
        positionNftAccount,
        owner: user,
        payer: user,
        eventAuthority,
        dammProgram: METEORA_PROGRAM_ID,
      })
      .signers([userKeypair, vesting])
      .preInstructions([computeUnitIx])
      .rpc();

    logTxnSignature(tx);
  });
});
