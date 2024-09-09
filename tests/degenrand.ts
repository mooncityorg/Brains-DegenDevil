// import * as anchor from '@project-serum/anchor';
// import { assert, expect } from "chai";
// import { UserSession, MockOracleSession as OracleSession } from "./sessions.js";
// import { setup } from './setup';
// import { getKeypair } from './utils';

// describe('degenrand', () => {
//     // const ENV = 'http://localhost:8899';
//     const ENV = "https://api.devnet.solana.com";
//     const AIRDROP = 1000000000;
//     const FEE = 495000; // In lamports, defined in lib.rs

//     const oracleKeypair = getKeypair("oracle");
//     const userKeypair = getKeypair("bob");
//     const notOracleKeypair = getKeypair("id");

//     const oracleSession = new OracleSession(oracleKeypair, anchor.workspace.Degenrand.idl, anchor.workspace.Degenrand.programId, ENV);
//     const userSession = new UserSession(userKeypair, anchor.workspace.Degenrand.idl, anchor.workspace.Degenrand.programId, oracleKeypair.publicKey, ENV);
//     const notOracleSession = new OracleSession(notOracleKeypair, anchor.workspace.Degenrand.idl, anchor.workspace.Degenrand.programId, ENV);

//     async function getRequester(oraclePubKey) {
//         let requesters = await userSession.program.account.requester.all();
//         return requesters.filter(req => req.account.oracle.toString() == oraclePubKey.toString());
//     }

//     it('Setting up tests', async () => {

//         let userBalance = await userSession.getBalance();
//         let oracleBalance = await oracleSession.getBalance();
//         let notOracleBalance = await notOracleSession.getBalance();

//         console.table([{ userBalance: userBalance }, { oracleBalance: oracleBalance }, { notOracleBalance: notOracleBalance }]);

//     });

//     it('Initializes properly', async () => {
//         // Set accounts
//         await userSession.setAccounts();

//         const beforeBalance = await userSession.getBalance();
//         await userSession.initializeAccount();
//         const afterBalance = await userSession.getBalance();

//         let requesters = await getRequester(oracleKeypair.publicKey);

//         assert(requesters.length == 1);

//         let requester = requesters[0];

//         console.table(requesters);
//         assert(requester.account.count.toNumber() == 0);
//         assert(requester.account.authority.toString() == userKeypair.publicKey.toString());
//         assert(!requester.account.activeRequest);

//         console.log(`Cost of initialization is ${beforeBalance - afterBalance}`);
//     });

//     it('Creates a random request', async () => {
//         const beforeBalance = await userSession.getBalance();
//         const oldOracleBalance = await oracleSession.getBalance();
//         await userSession.requestRandom();
//         const afterBalance = await userSession.getBalance();
//         const newOracleBalance = await oracleSession.getBalance();

//         let requesters = await getRequester(oracleKeypair.publicKey);

//         console.table(requesters);
//         assert(requesters.length == 1);

//         let requester = requesters[0];
//         assert(requester.account.count.toNumber() == 1); // Before 0, now 1
//         assert(requester.account.authority.toString() == userKeypair.publicKey.toString()); // Still the same owner
//         assert(requester.account.activeRequest); // Now true

//         console.log(`Cost of request is ${beforeBalance - afterBalance}`);
//         assert((newOracleBalance - oldOracleBalance) == FEE);
//     });

//     // Should Fail
//     it('Cannot make multiple requests in a row', async () => {
//         try {
//             await userSession.requestRandom();
//         } catch (e) {
//             assert(e.message);
//         }
//     });

//     // Should Fail
//     it('Wrong Oracle cannot respond to request', async () => {
//         let requesters = await getRequester(oracleKeypair.publicKey);

//         assert(requesters.length == 1);

//         try {
//             await notOracleSession.publishRandom(requesters[0]);
//         } catch (e) {
//             assert(e.message);
//         }
//     });

//     it('Oracle can respond to request', async () => {
//         let requesters = await getRequester(oracleKeypair.publicKey);

//         assert(requesters.length == 1);
//         console.table(requesters);
//         const oldOracleBalance = await oracleSession.getBalance();
//         await oracleSession.publishRandom(requesters[0]);
//         const newOracleBalance = await oracleSession.getBalance();

//         requesters = await getRequester(oracleKeypair.publicKey);
//         assert(requesters.length == 1);

//         let requester = requesters[0];
//         assert(requester.account.count.toNumber() == 1); // Still 1
//         assert(requester.account.authority.toString() == userKeypair.publicKey.toString()); // Still the same owner
//         assert(!requester.account.activeRequest); // Before true, Now false
//         console.table(requesters);

//         console.log(`Cost of Oracle Response is ${oldOracleBalance - newOracleBalance}`);
//     });

//     // Should Fail
//     it('Oracle cannot respond multiple times to request', async () => {
//         let requesters = await getRequester(oracleKeypair.publicKey);
//         try {
//             await oracleSession.publishRandom(requesters[0]);
//         } catch (e) {
//             assert(e.message);
//         }
//     });
// });