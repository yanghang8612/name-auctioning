import { getHashedName, getNameAccountKey } from "@bonfida/spl-name-service";
import { PublicKey, Connection } from "@solana/web3.js";
import { PROGRAM_ID, ROOT_DOMAIN_ACCOUNT } from "./bindings";

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
