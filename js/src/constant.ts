import { PublicKey } from '@solana/web3.js';

// devnet
export const PROGRAM_ID = new PublicKey(
  'jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR'
);

export const NAMING_SERVICE_PROGRAM_ID = new PublicKey(
  'namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX'
);

export const AUCTION_PROGRAM_ID = new PublicKey(
  'AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT'
);

export const BASE_AUCTION_DATA_SIZE =
  32 + 32 + 32 + 9 + 9 + 9 + 9 + 1 + 32 + 1 + 8 + 8;

export const ROOT_DOMAIN_ACCOUNT = new PublicKey(
  '58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx'
  // "4MpujQVQLPPsC8ToEcSepSvtYCf5ZBf2odxZkZ2Qz8QH"
);

export const PYTH_FIDDA_PRICE_ACC = new PublicKey(
  'ETp9eKXVv1dWwHSpsXRUuXHmw24PwRkttCGVgpZEY9zF'
);

// const MARKET_STATE_SPACE = 5000; // Size enough for more than 40 active leverage types with 10 memory pages each.

export const BONFIDA_SOL_VAULT = new PublicKey(
  'GcWEQ9K78FV7LEHteFVciYApERk5YvQuFDQPk1yYJVXi'
);
