import { PublicKey, TransactionInstruction } from '@solana/web3.js';
import BN from 'bn.js';
import { Schema, serialize } from 'borsh';

export enum PositionType {
  Short = 0,
  Long = 1,
}

export const BONFIDA_BNB = new PublicKey(
  'DmSyHDSM9eSLyvoLsPvDr5fRRFZ7Bfr3h3ULvWpgQaq7'
);

export class Instruction {
  state_nonce?: number;
  hashed_name?: Uint8Array;
  lamports?: BN;
  space?: number;
}

export class initInstruction {
  tag: number;
  stateNonce: number;
  static schema: Schema = new Map([
    [
      initInstruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['stateNonce', 'u8'],
        ],
      },
    ],
  ]);

  constructor(obj: { stateNonce: number }) {
    this.tag = 0;
    this.stateNonce = obj.stateNonce;
  }

  serialize(): Uint8Array {
    return serialize(initInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    stateAccount: PublicKey,
    systemProgram: PublicKey,
    feePayer: PublicKey,
    rentSysvarAccount: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys = [
      {
        pubkey: stateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: systemProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: feePayer,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: rentSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
    ];

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class createInstruction {
  tag: number;
  name: string;

  static schema: Schema = new Map([
    [
      createInstruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['name', 'string'],
        ],
      },
    ],
  ]);

  constructor(obj: { name: string }) {
    this.tag = 1;
    this.name = obj.name;
  }

  serialize(): Uint8Array {
    return serialize(createInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    rentSysvarAccount: PublicKey,
    clockSysvarAccount: PublicKey,
    namingServiceProgram: PublicKey,
    rootDomain: PublicKey,
    nameAccount: PublicKey,
    reverseLookupAccount: PublicKey,
    systemProgram: PublicKey,
    auctionProgram: PublicKey,
    auctionAccount: PublicKey,
    centralStateAccount: PublicKey,
    stateAccount: PublicKey,
    feePayer: PublicKey,
    quoteMint: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys = [
      {
        pubkey: rentSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: clockSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: namingServiceProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: rootDomain,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: reverseLookupAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: systemProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auctionProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auctionAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: centralStateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: stateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: quoteMint,
        isSigner: false,
        isWritable: false,
      },
    ];

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class claimInstruction {
  tag: number;
  hashed_name: Uint8Array;
  lamports: BN;
  space: number;

  static schema: Schema = new Map([
    [
      claimInstruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['hashed_name', [32]],
          ['lamports', 'u64'],
          ['space', 'u32'],
        ],
      },
    ],
  ]);

  constructor(obj: { hashed_name: Uint8Array; lamports: BN; space: number }) {
    this.tag = 2;
    this.hashed_name = obj.hashed_name;
    this.lamports = obj.lamports;
    this.space = obj.space;
  }

  serialize(): Uint8Array {
    return serialize(claimInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    splTokenProgram: PublicKey,
    clockSysvarAccount: PublicKey,
    namingServiceProgram: PublicKey,
    rootDomain: PublicKey,
    nameAccount: PublicKey,
    systemProgram: PublicKey,
    auctionProgram: PublicKey,
    auctionAccount: PublicKey,
    centralStateAccount: PublicKey,
    stateAccount: PublicKey,
    feePayer: PublicKey,
    quoteMint: PublicKey,
    destinationTokenAccount: PublicKey,
    bidderWallet: PublicKey,
    bidderPot: PublicKey,
    bidderPotTokenAccount: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys = [
      {
        pubkey: clockSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: splTokenProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: namingServiceProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: rootDomain,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: systemProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auctionProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auctionAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: centralStateAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: stateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: quoteMint,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: destinationTokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bidderWallet,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: bidderPot,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bidderPotTokenAccount,
        isSigner: false,
        isWritable: true,
      },
    ];

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class resellInstruction {
  tag: number;
  name: string;
  minimumPrice: number;

  static schema: Schema = new Map([
    [
      createInstruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['name', 'string'],
          ['minimumPrice', 'u64'],
        ],
      },
    ],
  ]);

  constructor(obj: { name: string; minimumPrice: number }) {
    this.tag = 4;
    this.name = obj.name;
    this.minimumPrice = obj.minimumPrice;
  }

  serialize(): Uint8Array {
    return serialize(createInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    rentSysvarAccount: PublicKey,
    clockSysvarAccount: PublicKey,
    namingServiceProgram: PublicKey,
    rootDomain: PublicKey,
    nameAccount: PublicKey,
    nameOwnerAccount: PublicKey,
    reverseLookupAccount: PublicKey,
    systemProgram: PublicKey,
    auctionProgram: PublicKey,
    auctionAccount: PublicKey,
    centralStateAccount: PublicKey,
    stateAccount: PublicKey,
    resellingStateAccount: PublicKey,
    destinationTokenAccount: PublicKey,
    feePayer: PublicKey,
    quoteMint: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys = [
      {
        pubkey: rentSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: clockSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: namingServiceProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: rootDomain,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameOwnerAccount,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: reverseLookupAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: systemProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auctionProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auctionAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: centralStateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: stateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: resellingStateAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: destinationTokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: quoteMint,
        isSigner: false,
        isWritable: false,
      },
    ];

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
