import { getHashedName, getNameAccountKey } from '@bonfida/spl-name-service';
import { PublicKey, Connection, AccountInfo } from '@solana/web3.js';
import { PROGRAM_ID, ROOT_DOMAIN_ACCOUNT } from './bindings';

export async function getState(connection: Connection, name: string) {
  let hashedName = await getHashedName(name);

  let nameAccount = await getNameAccountKey(hashedName, ROOT_DOMAIN_ACCOUNT);
  let [stateAccount] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer()],
    PROGRAM_ID
  );

  let data = await connection.getAccountInfo(stateAccount);
  return data;
}

export async function getReverseLookupAddress(
  nameAccount: PublicKey
): Promise<PublicKey> {
  let [centralState] = await PublicKey.findProgramAddress(
    [PROGRAM_ID.toBuffer()],
    PROGRAM_ID
  );

  let hashedReverseLookup = await getHashedName(nameAccount.toBase58());
  let reverseLookupAccount = await getNameAccountKey(
    hashedReverseLookup,
    centralState
  );
  return reverseLookupAccount;
}

export async function getDNSRecordAddress(
  nameAccount: PublicKey,
  type: string
) {
  let hashedName = await getHashedName('\0'.concat(type));
  let recordAccount = await getNameAccountKey(
    hashedName,
    undefined,
    nameAccount
  );
  return recordAccount;
}

/** Cannot pass more than 100 accounts to connection.getMultipleAccountsInfo */
export const getMultipleAccountInfo = async (
  connection: Connection,
  keys: PublicKey[]
) => {
  let result: (AccountInfo<Buffer> | null)[] = [];
  while (keys.length > 0) {
    result.push(
      ...(await connection.getMultipleAccountsInfo(keys.splice(0, 100)))
    );
  }
  return result;
};
