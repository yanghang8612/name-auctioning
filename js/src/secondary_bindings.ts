import {
  ConfirmedSignaturesForAddress2Options,
  ConfirmedTransaction,
  Connection,
  PublicKey,
  TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import { deserialize } from "borsh";
import { PERPS_PROGRAM_ID } from "./bindings";
import {
  parseInstructionData,
  PerpInstruction,
  PositionType,
  PerpTradeInstruction,
  crankLiquidationInstruction,
} from "./instructions";
import { Oracle } from "./oracle_utils";
import { MarketState, OpenPosition, UserAccount } from "./state";
import { getFilteredProgramAccounts } from "./utils";

export interface Order {
  market: PublicKey;
  openPositionAccount: PublicKey;
  position: OpenPosition;
  position_index: number;
}

export async function getOrders(connection: Connection, owner: PublicKey) {
  let orders: Order[] = [];
  const accounts = await getUserAccounts(connection, owner);
  for (let p of accounts) {
    try {
      let parsed = UserAccount.parse(p.publicKey, p.accountInfo.data);
      let market = parsed.market;
      for (let idx = 0; idx < parsed.openPositions.length; idx++) {
        orders.push({
          openPositionAccount: p.publicKey,
          position: parsed.openPositions[idx],
          position_index: idx,
          market,
        });
      }
    } catch {
      console.log(
        'Found corrupted UserAccount at ',
        p.publicKey.toBase58(),
        '. Skipping.',
      );
    }
  }
  return orders;
}

async function getUserAccounts(connection: Connection, owner: PublicKey) {
  const filters = [
    {
      memcmp: {
        offset: 1,
        bytes: owner.toBase58(),
      },
    },
  ];
  return await getFilteredProgramAccounts(
    connection,
    PERPS_PROGRAM_ID,
    filters
  );
}

export async function getUserAccountsForOwner(
  connection: Connection,
  owner: PublicKey,
): Promise<(UserAccount | undefined)[]> {
  return (await getUserAccounts(connection, owner)).map(p => {
    console.log(p.publicKey.toBase58());
    try {
      return UserAccount.parse(p.publicKey, p.accountInfo.data);
    } catch {
      console.log(
        'Found corrupted UserAccount at ',
        p.publicKey.toBase58(),
        '. Skipping.',
      );
    }
  });
}

export async function getMarketState(
  connection: Connection,
  marketAddress: PublicKey
) {
  return await MarketState.retrieve(connection, marketAddress);
}

export async function getOraclePrice(
  connection: Connection,
  oracleAddress: PublicKey
) {
  let oracle_data = (await connection.getAccountInfo(oracleAddress))?.data;
  if (!oracle_data) {
    throw "Unable to retrieve oracle data";
  }

  let oracle: Oracle = deserialize(Oracle.schema, Oracle, oracle_data);

  return oracle.answer_median / 10 ** oracle.decimals;
}

// export async function getLiquidationTransaction(
//   connection: Connection,
//   closeSignature: string,
//   marketAddress: PublicKey,
//   side: PositionType,
//   userAccount: PublicKey,
// ) {
//   // TODO: UNFINISHED
//   let re = /Program log: Order not found, it was liquidated at index: (?<liquidationIndex>.*), with collateral (?<collateral>.*), with parent node slot (?<parentNodeSlot>.*)/;
//   let tx = await connection.getParsedConfirmedTransaction(closeSignature);
//   let logMessages = tx?.meta?.logMessages;
//   if (!logMessages) {
//     throw 'Failed to parse transaction';
//   }
//   let liquidationIndex: number;
//   let parentNodeSlot: number;
//   let positionType: PositionType;
//   for (let l of logMessages) {
//     let m = l.match(re);
//     if (!!m) {
//       liquidationIndex = parseInt((m.groups as any)['liquidationIndex']);
//     }
//   }

//   let re2 =/Program log: Liquidation index: (?<liquidationIndex>.*)/;;
//   let pages = await findTransactionPaging(connection, marketAddress, closeSignature, lastKnownValidTransaction)

//   let rawInstructions = await getPastInstructionsRaw(
//     connection,
//     marketAddress,
//     { before: closeSignature },
//   );
//   let liqInstructions = rawInstructions
//     .filter(i => i.instruction.data[0] === 8)
//     .findIndex(v => {
//       let i = v.log?;
//     });
//   return 'DUMMYSIGNATURE';
// }

// async function findTransactionPaging(connection: Connection, address: PublicKey, before: string, after: string): Promise<string[]>{
//   let transactionPages: string[] = [];
//   let current = before;
//   while (true){
//     let sigs = await connection.getConfirmedSignaturesForAddress2(address, {before: current});
//     let search_result = sigs.find((v) => v.signature == after);
//     transactionPages.push(current);
//     if (!!search_result){
//       break;
//     }
//     current = sigs[-1].signature;
//   }
//   return transactionPages
// }

export interface PastInstruction {
  instruction: PerpInstruction;
  slot: number;
  time: number;
  tradeIndex: number;
  signature: string;
  log?: string[] | null;
  feePayer?: PublicKey;
}

export interface PastTrade {
  instruction: PastInstruction;
  markPrice?: number;
  orderSize?: number;
}
interface PastInstructionRaw {
  signature: string;
  instruction: TransactionInstruction;
  slot: number;
  time: number;
  tradeIndex: number;
  log?: string[] | null;
  feePayer?: PublicKey;
}

export async function getPastInstructions(
  connection: Connection,
  marketAddress: PublicKey,
  options?: ConfirmedSignaturesForAddress2Options
): Promise<PastInstruction[]> {
  let sigs = await connection.getConfirmedSignaturesForAddress2(
    marketAddress,
    options
  );
  console.log("Retrieved signatures: ", sigs.length);
  let pastInstructions: PastInstruction[] = (
    await getPastInstructionsRaw(connection, marketAddress, options)
  ).map(parseRawInstruction);

  return pastInstructions;
}

async function getPastInstructionsRaw(
  connection: Connection,
  marketAddress: PublicKey,
  options?: ConfirmedSignaturesForAddress2Options
): Promise<PastInstructionRaw[]> {
  let sigs = await connection.getConfirmedSignaturesForAddress2(
    marketAddress,
    options
  );
  console.log("Retrieved signatures: ", sigs.length);
  let pastInstructions: PastInstructionRaw[] = [];

  for (let s of sigs) {
    let tx_null = await connection.getConfirmedTransaction(s.signature);
    if (!tx_null || !tx_null?.meta?.err) {
      continue;
    }
    let tx = tx_null as ConfirmedTransaction;
    let tradeIndex = 0;
    tx.transaction.instructions.forEach((i, idx) => {
      if (i.programId.toBase58() !== PERPS_PROGRAM_ID.toBase58()) {
        // console.log("skipped with programId: ", i.programId.toBase58());
        return;
      }
      let log = tx.meta?.logMessages;
      pastInstructions.push({
        signature: s.signature,
        instruction: i,
        time: tx.blockTime as number,
        slot: tx.slot,
        log,
        feePayer: tx.transaction.feePayer,
        tradeIndex,
      });
      if (i.data[0] in [2, 5, 6]) {
        tradeIndex++;
      }
    });
  }
  return pastInstructions;
}

export async function getPastTrades(
  connection: Connection,
  marketAddress: PublicKey,
  options?: ConfirmedSignaturesForAddress2Options
): Promise<PastTrade[]> {
  let pastInstructions = await getPastInstructionsRaw(
    connection,
    marketAddress,
    options
  );
  let filtered = pastInstructions.filter(
    (i) => i.instruction.data[0] in [2, 5, 6]
  );
  let parsed = filtered.map(parseRawInstruction).map(extractTradeInfo);
  return parsed;
}

function parseRawInstruction(i: PastInstructionRaw): PastInstruction {
  return {
    instruction: parseInstructionData(i.instruction.data),
    slot: i.slot,
    time: i.time,
    log: i.log,
    feePayer: i.feePayer,
    tradeIndex: i.tradeIndex,
    signature: i.signature,
  };
}

export async function extractTradeInfoFromTransaction(connection: Connection, txSig: string): Promise<PastTrade[]>{
  let tx_null = await connection.getConfirmedTransaction(txSig);
  if (!tx_null){
    throw "Could not retrieve transaction"
  }
  let tx = tx_null as ConfirmedTransaction;
  let instructions = tx_null.transaction.instructions.filter((i) => {i.programId.toBase58() === PERPS_PROGRAM_ID.toBase58()});
  instructions = instructions.filter((v) => {v.data[0] in [2, 5, 6]});
  let parsedInstructions: PastInstructionRaw[] = instructions.map((v, idx) => {
    return {
      signature: txSig,
      instruction: v,
      time: tx.blockTime as number,
      slot: tx.slot,
      log: tx.meta?.logMessages,
      feePayer: tx.transaction.feePayer,
      tradeIndex: idx
    }
  });
  return parsedInstructions.map(parseRawInstruction).map(extractTradeInfo)
}

const markPriceExtractionRe = /Program log: Mark price for this transaction \(FP32\): (?<markPrice>.*), with size: (?<orderSize>.*)/;

function extractTradeInfo(i: PastInstruction): PastTrade {
  if (!i.log) {
    throw 'Unable to parse mark price due to empty log';
  }
  let currentTradeIndex = 0;
  let markPrice: number | undefined;
  let orderSize: number | undefined;
  for (let l of i.log) {
    let results = l.match(markPriceExtractionRe);
    if (!results) {
      continue;
    }
    currentTradeIndex++;
    if (currentTradeIndex === i.tradeIndex) {
      let markPriceStr = results.groups?.markPrice as string;
      let orderSizeStr = results.groups?.markPrice as string;
      markPrice = parseInt(markPriceStr) / 2 ** 32;
      orderSize = parseInt(orderSizeStr) / 2 ** 32;
    }
  }
  return {
    instruction: i,
    markPrice,
    orderSize,
  };
}
