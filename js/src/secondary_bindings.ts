import {
  getFilteredProgramAccounts,
  NAME_SERVICE_PROGRAM_ID,
} from '@bonfida/spl-name-service';
import { AccountInfo, Connection, PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import bs58 from 'bs58';
import { AUCTION_PROGRAM_ID } from './bindings';

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
    NAME_SERVICE_PROGRAM_ID,
    filters
  );
  return accounts.map((a) => {
    return new PublicKey(a.accountInfo.data.slice(64, 96));
  });
}
