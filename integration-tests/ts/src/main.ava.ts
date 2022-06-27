import { Worker, NearAccount, NEAR } from "near-workspaces";
import anyTest, { TestFn } from "ava";
import {
  defaultCallOptions,
  mintNFT,
  approveNFT,
  DEFAULT_GAS,
  payForStorage,
  placeNFTForSale,
  purchaseListedNFT,
  transferNFT,
} from "./utils";
import { expect, toChangeNearBalance } from "./assertions";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async (t) => {
  const worker = await Worker.init();
  const root = worker.rootAccount;

  const nft_contract = await root.createAndDeploy(
    root.getSubAccount("nft-contract").accountId,
    "../../out/main.wasm",
    { initialBalance: NEAR.parse("100 N").toJSON() }
  );
  await nft_contract.call(nft_contract, "new_default_meta", { owner_id: root });

  const market_contract = await root.createAndDeploy(
    root.getSubAccount("nft-market").accountId,
    "../../out/market.wasm",
    { initialBalance: NEAR.parse("100 N").toJSON() }
  );
  await market_contract.call(market_contract, "new", { owner_id: root });

  const alice = await root.createSubAccount("alice", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  t.context.worker = worker;
  t.context.accounts = { root, nft_contract, market_contract, alice };
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to tear down the worker:", error);
  });
});

test("nft contract: nft metadata view", async (t) => {
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
  t.deepEqual(
    await nft_contract.view("nft_metadata", { account_id: root }),
    expected
  );
});

test("nft contract: nft mint call", async (t) => {
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
  await alice.call(
    nft_contract,
    "nft_mint",
    request_payload,
    defaultCallOptions()
  );

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

test("nft contract: nft approve call", async (t) => {
  const { alice, nft_contract, market_contract } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await approveNFT(market_contract, alice, nft_contract);

  // test if approved
  const view_payload = {
    token_id: "TEST123",
    approved_account_id: market_contract,
  };
  const approved = await nft_contract.view("nft_is_approved", view_payload);
  t.true(approved, "Failed to approve NFT");
});

test("nft contract: nft approve call long msg string", async (t) => {
  const { alice, nft_contract, market_contract } = t.context.accounts;
  await mintNFT(alice, nft_contract);

  // approve NFT
  const approve_payload = {
    token_id: "TEST123",
    account_id: market_contract,
    msg: "sample message".repeat(10 * 1024),
  };
  const result = await alice.call_raw(
    nft_contract,
    "nft_approve",
    approve_payload,
    defaultCallOptions()
  );
  t.regex(result.promiseErrorMessages.join("\n"), /Exceeded the prepaid gas+/);

  // test if approved
  const view_payload = {
    token_id: "TEST123",
    approved_account_id: market_contract,
  };
  const approved = await nft_contract.view("nft_is_approved", view_payload);
  t.false(approved, "NFT approval should have failed");
});

test("cross contract: sell NFT listed on marketplace", async (t) => {
  const { alice, nft_contract, market_contract, bob } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
  await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

  const result1 = await expect(
    async () =>
      await purchaseListedNFT(nft_contract, bob, market_contract, sale_price),
    toChangeNearBalance(alice, "0.3 N"),
    toChangeNearBalance(bob, "-0.3 N")
  );
  t.deepEqual(
    result1.actual,
    result1.expected,
    "Should change user balances by sale_price of 0.3 N"
  );

  // NFT has new owner
  const view_payload = {
    token_id: "TEST123",
  };
  const token_info = await nft_contract.view("nft_token", view_payload);
  t.is(token_info["owner_id"], bob.accountId, "NFT should have been sold");
  // nothing left for sale on market
  const sale_supply = await market_contract.view("get_supply_sales");
  t.is(sale_supply, "0", "Expected no sales to be left on market");
});

test("cross contract: transfer NFT when listed on marketplace", async (t) => {
  const { alice, nft_contract, market_contract, bob, charlie } =
    t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
  await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

  await transferNFT(bob, market_contract, nft_contract);

  // purchase NFT
  const offer_payload = {
    nft_contract_id: nft_contract,
    token_id: "TEST123",
  };
  const result = await charlie.call_raw(
    market_contract,
    "offer",
    offer_payload,
    defaultCallOptions(
      DEFAULT_GAS + "0", // 10X default amount for XCC
      sale_price // Attached deposit must be greater than or equal to the current price
    )
  );

  // assert expectations
  // NFT has same owner
  const view_payload = {
    token_id: "TEST123",
  };
  const token_info = await nft_contract.view("nft_token", view_payload);
  t.is(
    token_info["owner_id"],
    bob.accountId, // NFT was transferred to bob
    "NFT should have bob as owner"
  );
  // Unauthorized error should be found
  t.regex(result.promiseErrorMessages.join("\n"), /Unauthorized+/);
});

test("cross contract: approval revoke", async (t) => {
  const { alice, nft_contract, market_contract, bob } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);
  await placeNFTForSale(
    market_contract,
    alice,
    nft_contract,
    "300000000000000000000000"
  );

  // revoke approval
  const revoke_payload = {
    token_id: "TEST123",
    account_id: market_contract, // revoke market contract authorization
  };
  await alice.call(
    nft_contract,
    "nft_revoke",
    revoke_payload,
    defaultCallOptions(DEFAULT_GAS, "1") // Requires attached deposit of exactly 1 yoctoNEAR
  );

  // transfer NFT
  const transfer_payload = {
    receiver_id: bob,
    token_id: "TEST123",
    approval_id: 1,
  };
  const result = await market_contract.call_raw(
    nft_contract,
    "nft_transfer",
    transfer_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );

  // assert expectations
  // Unauthorized error should be found
  t.regex(result.promiseErrorMessages.join("\n"), /Unauthorized+/);
});

test("cross contract: reselling and royalties", async (t) => {
  const { alice, nft_contract, market_contract, bob, charlie } =
    t.context.accounts;
  const royalties_string = `{"${alice.accountId}":2000}`;
  const royalties = JSON.parse(royalties_string);
  await mintNFT(alice, nft_contract, royalties);
  await payForStorage(alice, market_contract);
  const ask_price = "300000000000000000000000";
  await placeNFTForSale(market_contract, alice, nft_contract, ask_price);

  const bid_price = ask_price + "0";

  const purchase_result = await expect(
    async () =>
      await purchaseListedNFT(nft_contract, bob, market_contract, bid_price),
    toChangeNearBalance(alice, "3 N"),
    toChangeNearBalance(bob, "-3 N")
  );
  t.deepEqual(
    purchase_result.actual,
    purchase_result.expected,
    "Should change user balances when doing NFT 1"
  );

  // bob relists NFT for higher price
  await payForStorage(bob, market_contract);
  const resell_ask_price = bid_price + "0";
  await placeNFTForSale(market_contract, bob, nft_contract, resell_ask_price);

  // bob updates price to lower ask
  const lowered_resell_ask_price = "600000000000000000000000";
  const update_price_payload = {
    nft_contract_id: nft_contract,
    token_id: "TEST123",
    price: lowered_resell_ask_price,
  };
  await bob.call(
    market_contract,
    "update_price",
    update_price_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );

  const purchase_result_2 = await expect(
    async () =>
      await purchaseListedNFT(
        nft_contract,
        charlie,
        market_contract,
        resell_ask_price
      ),
    toChangeNearBalance(alice, "6 N"),
    toChangeNearBalance(bob, "24 N"),
    toChangeNearBalance(charlie, "-30 N")
  );
  t.deepEqual(
    purchase_result_2.actual,
    purchase_result_2.expected,
    "Should change balances by a total of 30 N"
  );
});
