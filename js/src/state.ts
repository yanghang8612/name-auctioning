import { PublicKey, Connection } from '@solana/web3.js';
import { Schema, deserializeUnchecked } from 'borsh';
import { getHashedName, getNameAccountKey } from '@bonfida/spl-name-service';
import { ROOT_DOMAIN_ACCOUNT, AUCTION_PROGRAM_ID } from './bindings';

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
    name: string
  ): Promise<NameAuction> {
    let hashedName = await getHashedName(name);

    let nameAccount = await getNameAccountKey(
      hashedName,
      undefined,
      ROOT_DOMAIN_ACCOUNT
    );
    let auctionSeeds = [
      Buffer.from('auction', 'utf-8'),
      AUCTION_PROGRAM_ID.toBuffer(),
      nameAccount.toBuffer(),
    ];
    let [auctionAccount] = await PublicKey.findProgramAddress(
      auctionSeeds,
      AUCTION_PROGRAM_ID
    );
    let data = await connection.getAccountInfo(auctionAccount, 'processed');
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
