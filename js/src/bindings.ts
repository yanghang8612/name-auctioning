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
  BONFIDA_FIDA_BNB,
  BONFIDA_USDC_BNB,
  claimInstruction,
  createInstruction,
  createReverseInstruction,
  initInstruction,
  resellInstruction,
  endAuctionInstruction,
  createV2Instruction,
} from './instructions';
import BN from 'bn.js';
import { NameAuction } from './state';
import { getHashedName, getNameAccountKey } from '@bonfida/spl-name-service';

// devnet
export const PROGRAM_ID = new PublicKey(
  'jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR'
);

export const NAMING_SERVICE_PROGRAM_ID = new PublicKey(
  'namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX'
);

export const AUCTION_PROGRAM_ID = new PublicKey(
  'AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT'
);

export const BASE_AUCTION_DATA_SIZE =
  32 + 32 + 32 + 9 + 9 + 9 + 9 + 1 + 32 + 1 + 8 + 8;

export const ROOT_DOMAIN_ACCOUNT = new PublicKey(
  '58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx'
  // "4MpujQVQLPPsC8ToEcSepSvtYCf5ZBf2odxZkZ2Qz8QH"
);

export const PYTH_FIDDA_PRICE_ACC = new PublicKey(
  'ETp9eKXVv1dWwHSpsXRUuXHmw24PwRkttCGVgpZEY9zF'
);

// const SLOT_SIZE = 33;

export type PrimedTransaction = [Account[], TransactionInstruction[]];

// const MARKET_STATE_SPACE = 5000; // Size enough for more than 40 active leverage types with 10 memory pages each.

export const BONFIDA_SOL_VAULT = new PublicKey(
  'GcWEQ9K78FV7LEHteFVciYApERk5YvQuFDQPk1yYJVXi'
);

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
  name: string,
  feePayer: PublicKey,
  quoteMint: PublicKey,
  tldAuthority: PublicKey
): Promise<PrimedTransaction> {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

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

  let hashedReverseLookup = await getHashedName(nameAccount.toBase58());
  let reverseLookupAccount = await getNameAccountKey(
    hashedReverseLookup,
    centralState
  );

  let initCentralStateInstruction = new createInstruction({
    name,
  }).getInstruction(
    PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    SYSVAR_CLOCK_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    tldAuthority,
    nameAccount,
    reverseLookupAccount,
    SystemProgram.programId,
    AUCTION_PROGRAM_ID,
    auctionAccount,
    centralState,
    stateAccount,
    feePayer,
    quoteMint
  );

  let instructions = [initCentralStateInstruction];

  return [[], instructions];
}

export async function reclaimName(
  connection: Connection,
  nameAccount: PublicKey,
  name: string,
  feePayer: PublicKey,
  quoteMint: PublicKey,
  ownerWallet: PublicKey,
  tldAuthority: PublicKey,
  isResell: boolean,
  destinationTokenAccount: PublicKey,
  buyNow: PublicKey,
  bonfidaSolVault: PublicKey,
  discountAccount: PublicKey,
  isUsdc: boolean
): Promise<PrimedTransaction> {
  return await claimName(
    connection,
    nameAccount,
    name,
    feePayer,
    quoteMint,
    ownerWallet,
    PublicKey.default,
    PublicKey.default,
    new BN(0),
    0,
    tldAuthority,
    isResell,
    discountAccount,
    buyNow,
    bonfidaSolVault,
    isUsdc,
    destinationTokenAccount
  );
}

export async function claimName(
  connection: Connection,
  nameAccount: PublicKey,
  name: string,
  feePayer: PublicKey,
  quoteMint: PublicKey,
  bidderWallet: PublicKey,
  bidderPot: PublicKey,
  bidderPotTokenAccount: PublicKey,
  lamports: BN,
  space: number,
  tldAuthority: PublicKey,
  isResell: boolean,
  discountAccount: PublicKey,
  buyNow: PublicKey,
  bonfidaSolVault: PublicKey,
  isUsdc: boolean,
  destinationTokenAccount?: PublicKey,
  referrer?: PublicKey
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

  let hashed_name = await getHashedName(name);

  let [resellingStateAccount] = await PublicKey.findProgramAddress(
    [nameAccount.toBytes(), Uint8Array.from([1, 1])],
    PROGRAM_ID
  );

  let claimNameInstruction = new claimInstruction({
    hashed_name,
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
    resellingStateAccount,
    stateAccount,
    feePayer,
    quoteMint,
    destinationTokenAccount
      ? destinationTokenAccount
      : isUsdc
      ? BONFIDA_USDC_BNB
      : BONFIDA_FIDA_BNB,
    bidderWallet,
    bidderPot,
    bidderPotTokenAccount,
    isResell,
    discountAccount,
    buyNow,
    bonfidaSolVault,
    isUsdc,
    referrer
  );

  let instructions = [claimNameInstruction];

  return [[], instructions];
}

export async function resellDomain(
  nameAccount: PublicKey,
  name: string,
  feePayer: PublicKey,
  nameOwnerAccount: PublicKey,
  destinationTokenAccount: PublicKey,
  tldAuthority: PublicKey,
  minimumPrice: BN, // with precision
  endAuctionAt: number, // Unix timestamp in s,
  maxPrice?: BN, // price for buy now
  buyNow?: PublicKey
): Promise<PrimedTransaction> {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

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

  let [resellingStateAccount] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer(), Buffer.from([1, 1])],
    PROGRAM_ID
  );

  let hashedReverseLookup = await getHashedName(nameAccount.toBase58());
  let reverseLookupAccount = await getNameAccountKey(
    hashedReverseLookup,
    centralState
  );

  let initCentralStateInstruction = new resellInstruction({
    name,
    minimumPrice,
    endAuctionAt,
    maxPrice,
  }).getInstruction(
    PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    SYSVAR_CLOCK_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    tldAuthority,
    nameAccount,
    nameOwnerAccount,
    reverseLookupAccount,
    SystemProgram.programId,
    AUCTION_PROGRAM_ID,
    auctionAccount,
    centralState,
    stateAccount,
    resellingStateAccount,
    destinationTokenAccount,
    feePayer,
    buyNow
  );

  let instructions = [initCentralStateInstruction];

  return [[], instructions];
}

export async function createReverseName(
  nameAccount: PublicKey,
  name: string,
  feePayer: PublicKey,
  tldAuthority: PublicKey
): Promise<PrimedTransaction> {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

  let hashedReverseLookup = await getHashedName(nameAccount.toBase58());
  let reverseLookupAccount = await getNameAccountKey(
    hashedReverseLookup,
    centralState
  );

  let initCentralStateInstruction = new createReverseInstruction({
    name,
  }).getInstruction(
    PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    tldAuthority,
    reverseLookupAccount,
    centralState,
    feePayer
  );

  let instructions = [initCentralStateInstruction];

  return [[], instructions];
}

export const endAuction = async (
  name: string,
  rootDomain: PublicKey,
  nameAccount: PublicKey,
  auctionAccount: PublicKey,
  auctionCreator: PublicKey,
  destinationTokenAccount: PublicKey
) => {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

  let [stateAccount] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer()],
    PROGRAM_ID
  );

  let [resellingStateAccount] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer(), Buffer.from([1, 1])],
    PROGRAM_ID
  );

  const ix = new endAuctionInstruction({ name }).getInstruction(
    PROGRAM_ID,
    SYSVAR_CLOCK_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    rootDomain,
    nameAccount,
    AUCTION_PROGRAM_ID,
    auctionAccount,
    centralState,
    stateAccount,
    auctionCreator,
    resellingStateAccount,
    destinationTokenAccount,
    BONFIDA_SOL_VAULT,
    SystemProgram.programId
  );

  return [[], [ix]];
};

export const createV2 = async (
  name: string,
  space: number,
  nameAccount: PublicKey,
  reverseLookupAccount: PublicKey,
  buyer: PublicKey,
  buyerTokenAccount: PublicKey
) => {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );
  const ix = new createV2Instruction({ name, space }).getInstruction(
    PROGRAM_ID,
    SYSVAR_RENT_PUBKEY,
    NAMING_SERVICE_PROGRAM_ID,
    ROOT_DOMAIN_ACCOUNT,
    nameAccount,
    reverseLookupAccount,
    centralState,
    buyer,
    buyerTokenAccount,
    BONFIDA_FIDA_BNB
  );

  return [[], [ix]];
};
