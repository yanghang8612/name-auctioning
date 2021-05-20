import {
  Account,
  Connection,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  BONFIDA_BNB,
  claimInstruction,
  createInstruction,
  initInstruction,
} from './instructions';
import BN from 'bn.js';
import { NameAuction } from './state';

// devnet
export const PROGRAM_ID = new PublicKey(
  'jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR'
);

export const NAMING_SERVICE_PROGRAM_ID = new PublicKey(
  'namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX'
);

export const AUCTION_PROGRAM_ID = new PublicKey(
  'D9uCagAnYcETohUeg4CkrYHTMYZ3hAW8bToAFSEfAYw'
);

export const BASE_AUCTION_DATA_SIZE =
  32 + 32 + 32 + 9 + 9 + 9 + 9 + 1 + 32 + 1 + 8 + 8;

export const ROOT_DOMAIN_ACCOUNT = new PublicKey(
  '98vSQyPBwaG8wFnGJesg6BX2kThhrbbx5MReMd7WDDC3'
  // "4MpujQVQLPPsC8ToEcSepSvtYCf5ZBf2odxZkZ2Qz8QH"
);

// const SLOT_SIZE = 33;

export type PrimedTransaction = [Account[], TransactionInstruction[]];

// const MARKET_STATE_SPACE = 5000; // Size enough for more than 40 active leverage types with 10 memory pages each.

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
  nameAccount: PublicKey,
  hashedName: Buffer,
  feePayer: PublicKey,
  quoteMint: PublicKey,
  tldAuthority: PublicKey
): Promise<PrimedTransaction> {
  let auctionSeeds = [
    Buffer.from('auction', 'utf-8'),
    AUCTION_PROGRAM_ID.toBuffer(),
    nameAccount.toBuffer(),
  ];

  let [auctionAccount, _] = await PublicKey.findProgramAddress(
    auctionSeeds,
    AUCTION_PROGRAM_ID
  );

  let [stateAccount] = await PublicKey.findProgramAddress(
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
    tldAuthority,
    nameAccount,
    SystemProgram.programId,
    AUCTION_PROGRAM_ID,
    auctionAccount,
    stateAccount,
    feePayer,
    quoteMint
  );

  let instructions = [initCentralStateInstruction];

  return [[], instructions];
}

export async function claimName(
  connection: Connection,
  nameAccount: PublicKey,
  hashedName: Buffer,
  feePayer: PublicKey,
  quoteMint: PublicKey,
  bidderWallet: PublicKey,
  bidderPot: PublicKey,
  bidderPotTokenAccount: PublicKey,
  lamports: BN,
  space: number,
  tldAuthority: PublicKey
): Promise<PrimedTransaction> {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

  let state = await NameAuction.retrieve(connection, nameAccount);

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
    tldAuthority,
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
