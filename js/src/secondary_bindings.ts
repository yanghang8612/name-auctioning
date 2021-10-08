import {
  getFilteredProgramAccounts,
  getHashedName,
  getNameAccountKey,
  NameRegistryState,
  NAME_PROGRAM_ID,
} from '@bonfida/spl-name-service';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { AUCTION_PROGRAM_ID, PROGRAM_ID } from './bindings';
import { getMultipleAccountInfo } from './utils';

export async function findActiveAuctionsForUser(
  connection: Connection,
  userAccount: PublicKey
) {
  const filters = [
    {
      memcmp: {
        offset: 32,
        bytes: userAccount.toBase58(),
      },
    },
  ];
  const accounts = await getFilteredProgramAccounts(
    connection,
    AUCTION_PROGRAM_ID,
    filters
  );
  return accounts.map((a) => {
    return new PublicKey(a.accountInfo.data.slice(64, 96));
  });
}

export async function findEndingAuctions(
  connection: Connection,
  interval: number // Interval of time in seconds within which we're looking for expiring auctions
): Promise<{ publicKey: PublicKey; accountInfo: AccountInfo<Buffer> }[]> {
  const currentTime = new Date().getTime() / 1000;
  const maxTime = currentTime + interval;
  const timeMask = new BN(currentTime ^ maxTime);
  const truncateN = timeMask.byteLength();
  const timeMaskBytes = timeMask.toArrayLike(Buffer, 'le', 8).slice(truncateN);
  let filters = [
    {
      memcmp: {
        offset: 96,
        bytes: '1',
      },
    },
    {
      memcmp: {
        offset: 98 + truncateN,
        bytes: bs58.encode(timeMaskBytes),
      },
    },
  ];
  let accounts = await getFilteredProgramAccounts(
    connection,
    AUCTION_PROGRAM_ID,
    filters
  );
  filters = [
    {
      memcmp: {
        offset: 96,
        bytes: '2',
      },
    },
    {
      memcmp: {
        offset: 106 + truncateN,
        bytes: bs58.encode(timeMaskBytes),
      },
    },
  ];

  accounts = accounts.concat(
    await getFilteredProgramAccounts(connection, AUCTION_PROGRAM_ID, filters)
  );
  return accounts;
}

export async function findOwnedNameAccountsForUser(
  connection: Connection,
  userAccount: PublicKey
): Promise<PublicKey[]> {
  const filters = [
    {
      memcmp: {
        offset: 32,
        bytes: userAccount.toBase58(),
      },
    },
  ];
  const accounts = await getFilteredProgramAccounts(
    connection,
    NAME_PROGRAM_ID,
    filters
  );
  return accounts.map((a) => a.publicKey);
}

export async function performReverseLookup(
  connection: Connection,
  nameAccount: PublicKey
): Promise<string> {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );
  let hashedReverseLookup = await getHashedName(nameAccount.toBase58());
  let reverseLookupAccount = await getNameAccountKey(
    hashedReverseLookup,
    centralState
  );

  let name = await NameRegistryState.retrieve(connection, reverseLookupAccount);
  if (!name.data) {
    throw 'Could not retrieve name data';
  }
  let nameLength = new BN(name.data.slice(0, 4), 'le').toNumber();
  return name.data.slice(4, 4 + nameLength).toString();
}

export async function performReverseLookupBatch(
  connection: Connection,
  nameAccounts: PublicKey[]
): Promise<(string | undefined)[]> {
  const [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );
  let reverseLookupAccounts: PublicKey[] = [];
  for (let nameAccount of nameAccounts) {
    const hashedReverseLookup = await getHashedName(nameAccount.toBase58());
    const reverseLookupAccount = await getNameAccountKey(
      hashedReverseLookup,
      centralState
    );
    reverseLookupAccounts.push(reverseLookupAccount);
  }

  let names = await NameRegistryState.retrieveBatch(
    connection,
    reverseLookupAccounts
  );

  return names.map((name) => {
    if (name === undefined || name.data === undefined) {
      return undefined;
    }
    let nameLength = new BN(name.data.slice(0, 4), 'le').toNumber();
    return name.data.slice(4, 4 + nameLength).toString();
  });
}

export async function getDestinationTokenAccount(
  connection: Connection,
  nameAccount: PublicKey
): Promise<PublicKey> {
  let [resellingStateAccount] = await PublicKey.findProgramAddress(
    [nameAccount.toBytes(), Uint8Array.from([1, 1])],
    PROGRAM_ID
  );
  let destinationTokenData = (
    await connection.getAccountInfo(resellingStateAccount)
  )?.data;
  if (!destinationTokenData) {
    throw 'Could not retrieve reselling state. Is this a reselling auction?';
  }
  return new PublicKey(destinationTokenData);
}

export const getDestinationTokenAccountBatch = async (
  connection: Connection,
  nameAccounts: PublicKey[]
) => {
  let resellingStateAccounts: PublicKey[] = [];
  for (let nameAccount of nameAccounts) {
    let [resellingStateAccount] = await PublicKey.findProgramAddress(
      [nameAccount.toBytes(), Uint8Array.from([1, 1])],
      PROGRAM_ID
    );
    resellingStateAccounts.push(resellingStateAccount);
  }
  const destinationTokenData = (
    await getMultipleAccountInfo(connection, resellingStateAccounts)
  ).map((e) => e?.data);

  // @ts-ignore
  return destinationTokenData.map((e) => (e ? new PublicKey(e) : undefined));
};
