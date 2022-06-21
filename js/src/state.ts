import { PublicKey, Connection } from '@solana/web3.js';
import { Schema, deserializeUnchecked } from 'borsh';
import { PROGRAM_ID } from './constant';

export class NameAuction {
  isInitialized: number;
  quoteMint: PublicKey;
  signerNonce: number;
  auctionAccount: PublicKey;

  static schema: Schema = new Map([
    [
      NameAuction,
      {
        kind: 'struct',
        fields: [
          ['isInitialized', 'u8'],
          ['quoteMint', [32]],
          ['signerNonce', 'u8'],
          ['auctionAccount', [32]],
        ],
      },
    ],
  ]);

  constructor(obj: {
    isInitialized: number;
    quoteMint: Uint8Array;
    signerNonce: number;
    auctionAccount: Uint8Array;
  }) {
    this.isInitialized = obj.isInitialized;
    this.quoteMint = new PublicKey(obj.quoteMint);
    this.signerNonce = obj.signerNonce;
    this.auctionAccount = new PublicKey(obj.auctionAccount);
  }

  static async retrieve(
    connection: Connection,
    nameAccount: PublicKey
  ): Promise<NameAuction> {
    let [stateAccount] = await PublicKey.findProgramAddress(
      [nameAccount.toBuffer()],
      PROGRAM_ID
    );

    let data = await connection.getAccountInfo(stateAccount, 'processed');
    if (data === null) {
      throw new Error('No name auction found');
    }

    let res: NameAuction = deserializeUnchecked(
      this.schema,
      NameAuction,
      data.data
    );
    return res;
  }
}
