import { Worker, NearAccount, NEAR } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';

const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async t => {
    const worker = await Worker.init();
    const root = worker.rootAccount;

    const nft_contract = await root.createAndDeploy(
        root.getSubAccount("nft-contract").accountId,
        "../../out/main.wasm",
        { initialBalance: NEAR.parse('3 N').toJSON() },
    );
    await nft_contract.call(nft_contract, 'new_default_meta', { owner_id: root });


    const market_contract = await root.createAndDeploy(
        root.getSubAccount("nft-market").accountId,
        "../../out/market.wasm",
        { initialBalance: NEAR.parse('3 N').toJSON() },
    );
    await market_contract.call(market_contract, 'new', { owner_id: root });

    const alice = await root.createSubAccount('alice', { initialBalance: NEAR.parse('100 N').toJSON() });

    t.context.worker = worker;
    t.context.accounts = { root, nft_contract, market_contract, alice };
});

test.afterEach(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

test("nft contract: nft metadata view", async t => {
    const { root, nft_contract } = t.context.accounts;
    const expected = {
        base_uri: null,
        icon: null,
        name: "NFT Tutorial Contract",
        reference: null,
        reference_hash: null,
        spec: "nft-1.0.0",
        symbol: "GOTEAM",
    };
    t.deepEqual(await nft_contract.view("nft_metadata", { account_id: root }), expected);
});