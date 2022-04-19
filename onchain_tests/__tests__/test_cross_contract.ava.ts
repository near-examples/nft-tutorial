import { NEAR } from "near-workspaces-ava";
import { expect, toChangeNearBalance } from "./assertions";
import { DEFAULT_GAS, defaultCallOptions, mintNFT, payForStorage, placeNFTForSale, purchaseListedNFT, transferNFT } from "./utils";
import { workspace } from "./workspace";

workspace.test("cross contract: sell NFT listed on marketplace", async (test, { nft_contract, market_contract, alice, bob }) => {
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);

  const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
  await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

  const result1 = await expect(
    async () => await purchaseListedNFT(nft_contract, bob, market_contract, sale_price),
    toChangeNearBalance(alice, "0.3 N"),
    toChangeNearBalance(bob, "-0.3 N")
  );
  test.deepEqual(result1.actual, result1.expected, "Should change user balances when doing NFT 1");

  // NFT has new owner
  const view_payload = {
    token_id: "TEST123",
  };
  const token_info = await nft_contract.view("nft_token", view_payload);
  test.is(token_info["owner_id"], bob.accountId, "NFT should have been sold");
  // nothing left for sale on market
  const sale_supply = await market_contract.view("get_supply_sales");
  test.is(sale_supply, "0", "Expected no sales to be left on market");
});

workspace.test("cross contract: transfer NFT when listed on marketplace", async (test, { nft_contract, market_contract, alice, bob, charlie }) => {
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
  test.is(
    token_info["owner_id"],
    bob.accountId, // NFT was transferred to bob
    "NFT should have bob as owner"
  );
  // Unauthorized error should be found
  test.regex(result.promiseErrorMessages.join("\n"), /Unauthorized+/);
});

workspace.test("cross contract: approval revoke", async (test, { nft_contract, market_contract, alice, bob }) => {
  await mintNFT(alice, nft_contract);
  await payForStorage(alice, market_contract);
  await placeNFTForSale(market_contract, alice, nft_contract, "300000000000000000000000");

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
    approval_id: 0,
  };
  const result = await market_contract.call_raw(nft_contract, "nft_transfer", transfer_payload, defaultCallOptions(DEFAULT_GAS, "1"));

  // assert expectations
  // Unauthorized error should be found
  test.regex(result.promiseErrorMessages.join("\n"), /Unauthorized+/);
});

workspace.test("cross contract: reselling and royalties", async (test, { nft_contract, market_contract, alice, bob, charlie }) => {
  const royalties_string = `{"${alice.accountId}":2000}`;
  const royalties = JSON.parse(royalties_string);
  await mintNFT(alice, nft_contract, royalties);
  await payForStorage(alice, market_contract);
  const ask_price = "300000000000000000000000";
  await placeNFTForSale(market_contract, alice, nft_contract, ask_price);

  const bid_price = ask_price + "0";

  const result1 = await expect(
    async () => await purchaseListedNFT(nft_contract, bob, market_contract, bid_price),
    toChangeNearBalance(alice, "3 N"),
    toChangeNearBalance(bob, "-3 N")
  );
  test.deepEqual(result1.actual, result1.expected, "Should change user balances when doing NFT 1");

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
  await bob.call(market_contract, "update_price", update_price_payload, defaultCallOptions(DEFAULT_GAS, "1"));

  const result2 = await expect(
    async () => await purchaseListedNFT(nft_contract, charlie, market_contract, resell_ask_price),
    toChangeNearBalance(alice, "6 N"),
    toChangeNearBalance(bob, "24 N"),
    toChangeNearBalance(charlie, "-30 N")
  );
  test.deepEqual(result2.actual, result2.expected, "Should change user balances when doing NFT");
});
