import { Connection } from '@solana/web3.js';
import { findEndingAuctions } from './secondary_bindings';

async function test() {
  let connection = new Connection('https://api.devnet.solana.com');
  let auctions = await findEndingAuctions(connection, 3600);
  console.log(auctions.length);
}

test();
