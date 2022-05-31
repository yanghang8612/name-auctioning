import {
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from '@solana/web3.js';
import BN from 'bn.js';
import { Schema, serialize } from 'borsh';
import { NAMING_SERVICE_PROGRAM_ID, PYTH_FIDDA_PRICE_ACC } from './bindings';
import { TOKEN_PROGRAM_ID } from './token';

export enum PositionType {
  Short = 0,
  Long = 1,
}

export const BONFIDA_FIDA_BNB = new PublicKey(
  'AUoZ3YAhV3b2rZeEH93UMZHXUZcTramBvb4d9YEVySkc'
);

export const BONFIDA_USDC_BNB = new PublicKey(
  'DmSyHDSM9eSLyvoLsPvDr5fRRFZ7Bfr3h3ULvWpgQaq7'
);

export const CENTRAL_STATE = new PublicKey(
  '33m47vH6Eav6jr5Ry86XjhRft2jRBLDnDgPSHoquXi2Z'
);

export const ROOT_TLD = new PublicKey(
  '58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx'
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
        pubkey: PYTH_FIDDA_PRICE_ACC,
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
  space: number;

  static schema: Schema = new Map([
    [
      claimInstruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['hashed_name', [32]],
          ['space', 'u32'],
        ],
      },
    ],
  ]);

  constructor(obj: { hashed_name: Uint8Array; space: number }) {
    this.tag = 2;
    this.hashed_name = obj.hashed_name;
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
    resellingState: PublicKey,
    stateAccount: PublicKey,
    feePayer: PublicKey,
    quoteMint: PublicKey,
    destinationTokenAccount: PublicKey,
    bidderWallet: PublicKey,
    bidderPot: PublicKey,
    bidderPotTokenAccount: PublicKey,
    isResell: boolean,
    discountAccount: PublicKey,
    buyNow: PublicKey,
    bonfidaSolVault: PublicKey,
    isUsdc: boolean,
    referrer?: PublicKey
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
        pubkey: resellingState,
        isSigner: false,
        isWritable: false,
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
        isSigner: !isResell,
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
      {
        pubkey: isUsdc ? BONFIDA_USDC_BNB : BONFIDA_FIDA_BNB,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: discountAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: buyNow,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bonfidaSolVault,
        isSigner: false,
        isWritable: true,
      },
    ];

    if (referrer) {
      keys.push({
        pubkey: referrer,
        isSigner: false,
        isWritable: true,
      });
    }

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
  minimumPrice: BN;
  endAuctionAt: BN;
  maxPrice?: BN;

  static schema: Schema = new Map([
    [
      resellInstruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['name', 'string'],
          ['minimumPrice', 'u64'],
          ['endAuctionAt', 'u64'],
          ['maxPrice', { kind: 'option', type: 'u64' }],
        ],
      },
    ],
  ]);

  constructor(obj: {
    name: string;
    minimumPrice: BN;
    endAuctionAt: number;
    maxPrice?: BN;
  }) {
    this.tag = 4;
    this.name = obj.name;
    this.minimumPrice = obj.minimumPrice;
    this.endAuctionAt = new BN(obj.endAuctionAt);
    this.maxPrice = obj.maxPrice;
  }

  serialize(): Uint8Array {
    return serialize(resellInstruction.schema, this);
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
    buyNow?: PublicKey
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
        isWritable: true,
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
        isWritable: false,
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
    ];

    if (buyNow) {
      keys.push({
        pubkey: buyNow,
        isSigner: false,
        isWritable: true,
      });
    }

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class createReverseInstruction {
  tag: number;
  name: string;

  static schema: Schema = new Map([
    [
      createReverseInstruction,
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
    this.tag = 5;
    this.name = obj.name;
  }

  serialize(): Uint8Array {
    return serialize(createReverseInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    rentSysvarAccount: PublicKey,
    namingServiceProgram: PublicKey,
    rootDomain: PublicKey,
    reverseLookupAccount: PublicKey,
    centralStateAccount: PublicKey,
    feePayer: PublicKey,
    parentName?: PublicKey,
    parentNameOwner?: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys = [
      {
        pubkey: rentSysvarAccount,
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
        pubkey: reverseLookupAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: PublicKey.default,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: centralStateAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: feePayer,
        isSigner: true,
        isWritable: true,
      },
    ];

    if (parentName) {
      if (!parentNameOwner) {
        throw new Error('Missing parent name owner');
      }
      keys.push({
        pubkey: parentName,
        isSigner: false,
        isWritable: true,
      });
      keys.push({
        pubkey: parentNameOwner,
        isSigner: true,
        isWritable: false,
      });
    }

    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}

export class endAuctionInstruction {
  tag: number;
  name: string;

  static schema: Schema = new Map([
    [
      endAuctionInstruction,
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
    this.tag = 8;
    this.name = obj.name;
  }

  serialize(): Uint8Array {
    return serialize(endAuctionInstruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    clockSysvarAccount: PublicKey,
    nameProgramId: PublicKey,
    rootDomain: PublicKey,
    nameAccount: PublicKey,
    auctionProgram: PublicKey,
    auction: PublicKey,
    centralState: PublicKey,
    state: PublicKey,
    auctionCreator: PublicKey,
    resellingState: PublicKey,
    destinationToken: PublicKey,
    bonfidaSolVault: PublicKey,
    systemProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());

    const keys = [
      {
        pubkey: clockSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameProgramId,
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
        pubkey: auctionProgram,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: auction,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: centralState,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: state,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: auctionCreator,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: resellingState,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: destinationToken,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: bonfidaSolVault,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: systemProgram,
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

export class createV2Instruction {
  tag: number;
  name: string;
  space: number;

  static schema: Schema = new Map([
    [
      createV2Instruction,
      {
        kind: 'struct',
        fields: [
          ['tag', 'u8'],
          ['name', 'string'],
          ['space', 'u32'],
        ],
      },
    ],
  ]);

  constructor(obj: { name: string; space: number }) {
    this.tag = 9;
    this.name = obj.name;
    this.space = obj.space;
  }

  serialize(): Uint8Array {
    return serialize(createV2Instruction.schema, this);
  }

  getInstruction(
    programId: PublicKey,
    rentSysvarAccount: PublicKey,
    nameProgramId: PublicKey,
    rootDomain: PublicKey,
    nameAccount: PublicKey,
    reverseLookupAccount: PublicKey,
    centralState: PublicKey,
    buyer: PublicKey,
    buyerTokenAccount: PublicKey,
    fidaVault: PublicKey,
    state: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    const keys = [
      {
        pubkey: rentSysvarAccount,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameProgramId,
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
        pubkey: reverseLookupAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: centralState,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: buyer,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: buyerTokenAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: PYTH_FIDDA_PRICE_ACC,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: fidaVault,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: state,
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

export class takeBackInstruction {
  tag: number;

  static schema: Schema = new Map([
    [
      takeBackInstruction,
      {
        kind: 'struct',
        fields: [['tag', 'u8']],
      },
    ],
  ]);

  constructor() {
    this.tag = 10;
  }

  serialize(): Uint8Array {
    return serialize(takeBackInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    admin: PublicKey,
    nameAccount: PublicKey
  ) {
    const data = Buffer.from(this.serialize());
    const keys = [
      {
        pubkey: admin,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: NAMING_SERVICE_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: nameAccount,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: CENTRAL_STATE,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: PublicKey.default,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: ROOT_TLD,
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
