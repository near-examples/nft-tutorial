import { Worker, NearAccount, NEAR } from 'near-workspaces';
import anyTest, { TestFn } from 'ava';
import { defaultCallOptions, mintNFT, approveNFT } from "./utils";

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
        { initialBalance: NEAR.parse('100 N').toJSON() },
    );
    await nft_contract.call(nft_contract, 'new_default_meta', { owner_id: root });

    const market_contract = await root.createAndDeploy(
        root.getSubAccount("nft-market").accountId,
        "../../out/market.wasm",
        { initialBalance: NEAR.parse('100 N').toJSON() },
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

test("nft contract: nft mint call", async t => {
    const { alice, nft_contract } = t.context.accounts;
    const request_payload = {
        token_id: "TEST123",
        metadata: {
            title: "LEEROYYYMMMJENKINSSS",
            description: "Alright time's up, let's do this.",
            media:
                "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1",
        },
        receiver_id: alice,
    };
    await alice.call(nft_contract, "nft_mint", request_payload, defaultCallOptions());

    const tokens = await nft_contract.view("nft_tokens");
    const expected = [
        {
            approved_account_ids: {},
            metadata: {
                copies: null,
                description: "Alright time's up, let's do this.",
                expires_at: null,
                extra: null,
                issued_at: null,
                media:
                    "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1",
                media_hash: null,
                reference: null,
                reference_hash: null,
                starts_at: null,
                title: "LEEROYYYMMMJENKINSSS",
                updated_at: null,
            },
            owner_id: alice.accountId,
            royalty: {},
            token_id: "TEST123",
        },
    ];
    t.deepEqual(tokens, expected, "Expected to find one minted NFT");
});

test("nft contract: nft approve call", async t => {
    const { alice, nft_contract, market_contract } = t.context.accounts;
    await mintNFT(alice, nft_contract);
    await approveNFT(market_contract, alice, nft_contract);

    // test if approved
    const view_payload = {
        token_id: "TEST123",
        approved_account_id: market_contract,
    };
    const approved = await nft_contract.view("nft_is_approved", view_payload);
    test.true(approved, "Failed to approve NFT");
});

test("nft contract: nft approve call long msg string", async (test, { nft_contract, market_contract, alice }) => {
    await mintNFT(alice, nft_contract);

    // approve NFT
    const approve_payload = {
        token_id: "TEST123",
        account_id: market_contract,
        msg: "sample message".repeat(10 * 1024),
    };
    const result = await alice.call_raw(nft_contract, "nft_approve", approve_payload, defaultCallOptions());
    test.regex(result.promiseErrorMessages.join("\n"), /Exceeded the prepaid gas+/);

    // test if approved
    const view_payload = {
        token_id: "TEST123",
        approved_account_id: market_contract,
    };
    const approved = await nft_contract.view("nft_is_approved", view_payload);
    test.false(approved, "NFT approval should have failed");
});
