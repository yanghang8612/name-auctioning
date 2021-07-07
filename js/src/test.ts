// import { Connection } from '@solana/web3.js';
// import {
//   // findEndingAuctions,
//   findOwnedNameAccountsForUser,
//   performReverseLookup,
// } from './secondary_bindings';

import { Stream } from 'stream';

async function test() {
  // let connection = new Connection('https://api.mainnet-beta.solana.com');
  // let auctions = await findEndingAuctions(connection, 3600);
  // console.log(auctions.length);

  // let a = new PublicKey('ABwpdjw6XJZUodNMbVFUeKgGFCQLKas4eeqiyQPPQsuH');
  // let domains = await findOwnedNameAccountsForUser(connection, a);
  // console.log(
  //   await Promise.all(
  //     domains.map(async (a) => {
  //       try {
  //         return await performReverseLookup(connection, a);
  //       } catch {
  //         return undefined;
  //       }
  //     })
  //   )
  // );
  let b = Buffer.from([1, 2, 3, 4, 5, 6, 7, 245, 42]);
  console.log(b);
  let s = Stream.Readable.from(buffer_gen(b));
  s.pause();
  console.log(s.read(1));
  console.log(s.read(1));
}

async function* buffer_gen(b: Buffer) {
  let i = 0;
  while (i + 2 < b.length) {
    yield b.slice(i, i + 2);
  }
  if (i + 2 > b.length) {
    yield b.slice(i, b.length);
  }
}

test();
