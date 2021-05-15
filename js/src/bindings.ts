import {
  Account,
  Connection,
  PublicKey,
  SystemProgram,
  CreateAccountParams,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, MintLayout, MintInfo } from "@solana/spl-token";
import {
  BONFIDA_BNB,
  claimInstruction,
  createInstruction,
  initInstruction,
} from "./instructions";
import BN from "bn.js";
import { getHashedName, getNameAccountKey } from "@bonfida/spl-name-service";
import { NameAuction } from "./state";

// devnet
export const PROGRAM_ID = new PublicKey(
  "CrR5iPwBE4wEGe7NS3YA2j5NXityhqEnumvW23MwnTA9"
);

export const NAMING_SERVICE_PROGRAM_ID = new PublicKey(
  "namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX"
);

export const AUCTION_PROGRAM_ID = new PublicKey(
  "HLGetPpEUaagthEtF4px9S24hwJrwz3qvgRZxkWTw4ei"
);

export const BASE_AUCTION_DATA_SIZE =
  32 + 32 + 32 + 9 + 9 + 9 + 9 + 1 + 32 + 1 + 8 + 8;

export const ROOT_DOMAIN_ACCOUNT = new PublicKey(
  "4MpujQVQLPPsC8ToEcSepSvtYCf5ZBf2odxZkZ2Qz8QH"
);

const SLOT_SIZE = 33;

export type PrimedTransaction = [Account[], TransactionInstruction[]];

const MARKET_STATE_SPACE = 5000; // Size enough for more than 40 active leverage types with 10 memory pages each.

export async function initCentralState(
  feePayer: PublicKey
): Promise<PrimedTransaction> {
  let [centralState, stateNonce] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

  let initCentralStateInstruction = new initInstruction({
    stateNonce,
  }).getInstruction(
    PROGRAM_ID,
    centralState,
    SystemProgram.programId,
    feePayer,
    SYSVAR_RENT_PUBKEY
  );

  let instructions = [initCentralStateInstruction];

  return [[], instructions];
}

export async function createNameAuction(
  connection: Connection,
  name: string,
  feePayer: PublicKey,
  quoteMint: PublicKey
): Promise<PrimedTransaction> {
  let hashedName = await getHashedName(name);

  let nameAccount = await getNameAccountKey(hashedName, ROOT_DOMAIN_ACCOUNT);

  let auctionAccount = new Account();

  let lamports = await connection.getMinimumBalanceForRentExemption(
    BASE_AUCTION_DATA_SIZE
  );

  let allocateAuctionAccount = SystemProgram.createAccount({
    /** The account that will transfer lamports to the created account */
    fromPubkey: feePayer,
    /** Public key of the created account */
    newAccountPubkey: auctionAccount.publicKey,
    /** Amount of lamports to transfer to the created account */
    lamports,
    /** Amount of space in bytes to allocate to the created account */
    space: BASE_AUCTION_DATA_SIZE,
    /** Public key of the program to assign as the owner of the created account */
    programId: AUCTION_PROGRAM_ID,
  });

  let [stateAccount, signerNonce] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer()],
    PROGRAM_ID
  );

  let initCentralStateInstruction = new createInstruction({
    hashedName: hashedName,
  }).getInstruction(
    PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    SYSVAR_CLOCK_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    ROOT_DOMAIN_ACCOUNT,
    nameAccount,
    SystemProgram.programId,
    AUCTION_PROGRAM_ID,
    auctionAccount.publicKey,
    stateAccount,
    feePayer,
    quoteMint
  );

  let instructions = [allocateAuctionAccount, initCentralStateInstruction];

  return [[auctionAccount], instructions];
}

export async function claimName(
  connection: Connection,
  name: string,
  feePayer: PublicKey,
  quoteMint: PublicKey,
  bidderWallet: PublicKey,
  bidderPot: PublicKey,
  bidderPotTokenAccount: PublicKey,

  lamports: BN,
  space: number
): Promise<PrimedTransaction> {
  let [centralState, stateNonce] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

  let hashedName = await getHashedName(name);

  let nameAccount = await getNameAccountKey(hashedName, ROOT_DOMAIN_ACCOUNT);

  let state = await NameAuction.retrieve(connection, name);

  let [stateAccount, _] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer()],
    PROGRAM_ID
  );

  let claimNameInstruction = new claimInstruction({
    hashedName,
    lamports,
    space,
  }).getInstruction(
    PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    SYSVAR_CLOCK_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    ROOT_DOMAIN_ACCOUNT,
    nameAccount,
    SystemProgram.programId,
    AUCTION_PROGRAM_ID,
    state.auctionAccount,
    centralState,
    stateAccount,
    feePayer,
    quoteMint,
    BONFIDA_BNB,
    bidderWallet,
    bidderPot,
    bidderPotTokenAccount
  );

  let instructions = [claimNameInstruction];

  return [[], instructions];
}
