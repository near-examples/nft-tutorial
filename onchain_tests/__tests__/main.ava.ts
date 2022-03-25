/**
 * Welcome to near-workspaces-ava!
 *
 * This is a working test which checks the functionality of [the status-message
 * contract][1]. For quick reference, here's the contract's implementation:
 *
 *     impl StatusMessage {
 *         pub fn set_status(&mut self, message: String) {
 *             let account_id = env::signer_account_id();
 *             self.records.insert(&account_id, &message);
 *         }

 *         pub fn get_status(&self, account_id: String) -> Option<String> {
 *             return self.records.get(&account_id);
 *         }
 *     }
 *
 * As you can see, this contract only has two methods, a setter and a getter.
 * The setter sets a status message for the account that signed the call to the
 * contract. The getter accepts an `account_id` param and returns the status for
 * that account.
 *
 * The tests below create a local blockchain with this contract deployed to
 * one account and two more accounts which store statuses in the contract.
 *
 *   [1]: https://github.com/near-examples/rust-status-message/tree/4e4767db257b748950bb3393352e2fff6c8e9b17
 */

/**
 * Start off by importing Workspace from near-workspaces-ava.
 */
import {Workspace} from 'near-workspaces-ava';

/**
 * Initialize a new workspace. In local sandbox mode, this will:
 *
 *   - Create a new local blockchain
 *   - Create the root account for that blockchain (see `root` below)
 *   - Execute any actions passed to the function
 *   - Shut down the newly created blockchain, but *save the data*
 */
const workspace = Workspace.init(async ({root}) => {
  // Create a subaccount of the root account, like `alice.sandbox`
  // (the actual account name is not guaranteed; you can get it with `alice.accountId`)
  const alice = await root.createAccount('alice');

  // Create a subaccount of the root account, and also deploy a contract to it
  const contract = await root.createAndDeploy(
    // Subaccount name
    'status-message',

    // Relative path (from package.json location) to the compiled contract file
    // which will be deployed to this account
    'compiled-contracts/status_message.wasm',

    // Provide `method` and `args` to call in the same transaction as the deploy
    // {
    //   method: 'init',
    //   args: {owner_id: root},
    // },
  );

  // You can call any contract methods that you want executed before the
  // beginning of all subsequent tests. In this example, Alice sets her status.
  // Don't forget to `await` your calls!
  await alice.call(contract, 'set_status', {message: 'hello'});

  // Return the accounts that you want available in subsequent tests
  // (`root` is always available)
  return {alice, contract};
});

/**
 * Now you can write some tests! In local sandbox mode, each `workspace.test` will:
 *
 *   - start a new local blockchain
 *   - copy the state from the blockchain created in `Workspace.init`
 *   - get access to the accounts created in `Workspace.init` using the same variable names
 *   - run concurrently with all other `workspace.test` calls, keeping data isolated
 *   - shut down at the end, forgetting all new data created
 *
 * It's also worth knowing that `workspace.test` is syntax sugar added by
 * near-workspaces-ava. With raw AVA + near-workspaces, here's how to write a test:
 *
 *     import avaTest from 'ava';
 *     import {Workspace} from 'near-workspaces';
 *     // Alternatively, you can import Workspace and ava both from near-workspaces-ava:
 *     // import {ava as avaTest, Workspace} from 'near-workspaces-ava';
 *
 *     const workspace = Workspace.init(...);
 *
 *     avaTest('root sets status', async test => {
 *       await workspace.fork(async ({contract, root}) => {
 *         ...
 *       });
 *     });
 *
 * Instead, with the syntax sugar, you can write this as you see it below –
 * saving an indentation level and avoiding one extra `await`.
 * (Extra credit: try rewriting this test using the "sugar-free" syntax.)
*/
workspace.test('root sets status', async (test, {contract, root}) => {
  // Don't forget to `await` your calls!
  await root.call(contract, 'set_status', {message: 'lol'});

  // Assert that two things are identical using `test.is`
  test.is(
    // Note that Root called the contract with `root.call(contract, ...)`, but
    // you view the contract with `contract.view`, since the account doing the
    // viewing is irrelevant.
    await contract.view('get_status', {account_id: root}),
    'lol',
  );
});

workspace.test('statuses initialized in Workspace.init', async (test, {alice, contract, root}) => {
  // If you want to store a `view` in a local variable, you can inform
  // TypeScript what sort of return value you expect.
  const aliceStatus: string = await contract.view('get_status', {account_id: alice});
  const rootStatus: null = await contract.view('get_status', {account_id: root});

  test.is(aliceStatus, 'hello');

  // Note that the test above sets a status for `root`, but here it's still
  // null! This is because tests run concurrently in isolated environments.
  test.is(rootStatus, null);
});

workspace.test('extra goodies', async (test, {alice, contract, root}) => {
  /**
   * AVA's `test` object has all sorts of handy functions. For example: `test.log`.
   * This is better than `console.log` in a couple ways:
   *
   *   - The log output only shows up if you pass `--verbose` or if the test fails.
   *   - The output is nicely-formatted, right below the rest of the test output.
   *
   * Try it out using `npm run test -- --verbose` (with yarn: `yarn test --verbose`),
   * or by adding `--verbose` to the `test` script in package.json
   */
  test.log({
    alice: alice.accountId,
    contract: contract.accountId,
    root: root.accountId,
  });

  /**
   * The Account class from near-workspaces overrides `toJSON` so that removing
   * `.accountId` from the lines above gives the same behavior.
   * (This explains something about the example `contract.view` calls above:
   * you may have noticed that they use things like `{account_id: root}`
   * instead of `{account_id: root.accountId}`.)
   * Here's a test to prove it; try updating the `test.log` above to see it.
   */
  test.is(
    JSON.stringify({alice}), // This is JS shorthand for `{ alice: alice }`
    JSON.stringify({alice: alice.accountId}),
  );
});

// For more example tests, see:
// https://github.com/near/workspaces-js/tree/main/__tests__
