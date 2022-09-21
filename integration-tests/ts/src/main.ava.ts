import anyTest, { TestFn } from "ava";
import { NEAR, NearAccount, Worker, BN } from "near-workspaces";
import path from "path";
import {
  approveNFT, defaultCallOptions, DEFAULT_GAS, mintNFT, payForStorage,
  placeNFTForSale, purchaseListedNFT, transferNFT
} from "./utils";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async (t) => {
  const worker = await Worker.init();
  const root = worker.rootAccount;

  const nftContractLocation = path.join(__dirname, "../../../out/main.wasm");
  const nft_contract = await root.devDeploy(
    nftContractLocation,
    {
      method: "new_default_meta",
      args: { owner_id: root },
      initialBalance: NEAR.parse("100 N").toJSON()
    }
  );

  const marketContractLocation = path.join(__dirname, "../../../out/market.wasm");
  const market_contract = await root.devDeploy(
    marketContractLocation,
    {
      method: "new",
      args: { owner_id: root },
      initialBalance: NEAR.parse("100 N").toJSON()
    }
  );

  const alice = await root.createSubAccount("alice", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  const bob = await root.createSubAccount("bob", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  const charlie = await root.createSubAccount("charlie", {
    initialBalance: NEAR.parse("100 N").toJSON(),
  });

  t.context.worker = worker;
  t.context.accounts = { root, nft_contract, market_contract, alice, bob, charlie };
});

test.afterEach.always(async (t) => {
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
  await payForStorage(alice, market_contract);

  // approve NFT
  const approve_payload = {
    token_id: "TEST123",
    account_id: market_contract,
    msg: "sample message".repeat(10 * 1024),
  };
  const result = await alice.callRaw(
    nft_contract,
    "nft_approve",
    approve_payload,
    defaultCallOptions()
  );
  t.regex(result.receiptFailureMessages.join("\n"), /Not valid SaleArgs+/);

  // test if approved
  const view_payload = {
    token_id: "TEST123",
    approved_account_id: market_contract,
  };
  const approved = await nft_contract.view("nft_is_approved", view_payload);
  t.true(approved, "NFT approval apss without sale args");
});

test("cross contract: sell NFT listed on marketplace", async (t) => {
  const { alice, nft_contract, market_contract, bob } = t.context.accounts;
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
  await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

  const alice_balance_before = await alice.availableBalance();
  const bob_balance_before = await bob.availableBalance();
  await purchaseListedNFT(nft_contract, bob, market_contract, sale_price);
  const alice_balance_after = await alice.availableBalance();
  const bob_balance_after = await bob.availableBalance();

  // assert alice balance increased by sale price
  const test_precision_dp_near = 1;
  const slice_val = test_precision_dp_near - 24;
  t.is(
    alice_balance_after.toString().slice(0, slice_val),
    alice_balance_before.add(NEAR.from(sale_price)).toString().slice(0, slice_val),
    "Alice balance should increase by sale price"
  );
  // bob balance should decrease by sale price
  t.is(
    bob_balance_after.toString().slice(0, slice_val),
    bob_balance_before.sub(NEAR.from(sale_price)).toString().slice(0, slice_val),
    "Bob balance should decrease by sale price"
  );

  // NFT has new owner
  const view_payload = {
    token_id: "TEST123",
  };
  const token_info: any = await nft_contract.view("nft_token", view_payload);
  t.is(token_info.owner_id, bob.accountId, "NFT should have been sold");
  // nothing left for sale on market
  const sale_supply = await market_contract.view("get_supply_sales");
  t.is(sale_supply, "0", "Expected no sales to be left on market");
});

test("cross contract: transfer NFT when listed on marketplace", async (t) => {
  const { alice, nft_contract, market_contract, bob, charlie } = t.context.accounts;
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
  const result = await charlie.callRaw(
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
  const token_info: any = await nft_contract.view("nft_token", view_payload);
  t.is(
    token_info.owner_id,
    bob.accountId, // NFT was transferred to bob
    "NFT should have bob as owner"
  );
  // Unauthorized error should be found
  t.regex(result.receiptFailureMessages.join("\n"), /Unauthorized+/);
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
  const result = await market_contract.callRaw(
    nft_contract,
    "nft_transfer",
    transfer_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );

  // assert expectations
  // Unauthorized error should be found
  t.regex(result.receiptFailureMessages.join("\n"), /Unauthorized+/);
});

test("cross contract: reselling and royalties", async (t) => {
  const { alice, nft_contract, market_contract, bob, charlie } = t.context.accounts;
  const royalties_string = `{"${alice.accountId}":2000}`;
  const royalties = JSON.parse(royalties_string);
  await mintNFT(alice, nft_contract, royalties);
  await payForStorage(alice, market_contract);
  const ask_price = "300000000000000000000000";
  await placeNFTForSale(market_contract, alice, nft_contract, ask_price);

  const bid_price = ask_price + "0";

  const alice_balance_before = await alice.availableBalance();
  const bob_balance_before = await bob.availableBalance();
  await purchaseListedNFT(nft_contract, bob, market_contract, bid_price);
  const alice_balance_after = await alice.availableBalance();
  const bob_balance_after = await bob.availableBalance();

  const test_precision_dp_near = 1;
  const slice_val = test_precision_dp_near - 24;
  t.is(
    alice_balance_after.toString().slice(0, slice_val),
    alice_balance_before.add(NEAR.from(bid_price)).toString().slice(0, slice_val),
    "Alice balance should increase by sale price"
  );
  t.is(
    bob_balance_after.toString().slice(0, slice_val),
    bob_balance_before.sub(NEAR.from(bid_price)).toString().slice(0, slice_val),
    "Bob balance should decrease by sale price"
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

  const alice_balance_before_2 = await alice.availableBalance();
  const bob_balance_before_2 = await bob.availableBalance();
  const charlie_balance_before_2 = await charlie.availableBalance();
  await purchaseListedNFT(nft_contract, charlie, market_contract, resell_ask_price);
  const alice_balance_after_2 = await alice.availableBalance();
  const bob_balance_after_2 = await bob.availableBalance();
  const charlie_balance_after_2 = await charlie.availableBalance();

  t.is(
    alice_balance_after_2.sub(alice_balance_before_2).toHuman(),
    "6 N",
    "Alice balance should increase by royalty fee of 20% of sale price"
  )
  t.is(
    bob_balance_after_2.sub(bob_balance_before_2).toHuman(),
    "24.00031 N",
    "Bob balance should decrease by sale price minus royalty fee of 20% of sale price"
  )
  t.is(
    charlie_balance_before_2.sub(charlie_balance_after_2).toHuman().slice(0, 2),
    "30",
    "Charlie balance should decrease by sale price"
  )
});
