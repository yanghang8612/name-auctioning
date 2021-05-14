import { getHashedName, getNameAccountKey } from "@bonfida/spl-name-service";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction, Connection, Account, Transaction, AccountInfo } from "@solana/web3.js";
import BN from "bn.js";
import { PROGRAM_ID, ROOT_DOMAIN_ACCOUNT } from "./bindings";

export async function getState(connection: Connection, name: string){
  let hashedName = await getHashedName(name);

  let nameAccount = await getNameAccountKey(hashedName, ROOT_DOMAIN_ACCOUNT);
  let [stateAccount, signerNonce] = await PublicKey.findProgramAddress(
    [nameAccount.toBuffer()],
    PROGRAM_ID
  );

  let data = await connection.getAccountInfo(stateAccount);
  
}