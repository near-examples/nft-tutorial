import {
  DEFAULT_GAS,
  defaultCallOptions,
  mintNFT,
  payForStorage,
  placeNFTForSale,
  purchaseListedNFT,
  transferNFT,
} from "./utils";
import { workspace } from "./workspace";

workspace.test(
  "cross contract: sell NFT listed on marketplace",
  async (test, { nft_contract, market_contract, alice, bob }) => {
    await mintNFT(alice, nft_contract);
    await payForStorage(alice, market_contract);

    const sale_price = "300000000000000000000000"; // sale price string in yoctoNEAR is 0.3 NEAR
    await placeNFTForSale(market_contract, alice, nft_contract, sale_price);

    // bob makes offer on listed nft
    const alice_balance_before_offer = (await alice.balance()).total.toBigInt();
    await purchaseListedNFT(nft_contract, bob, market_contract, sale_price);
    const alice_balance_after_offer = (await alice.balance()).total.toBigInt();
    const alice_balance_difference = (
      alice_balance_after_offer - alice_balance_before_offer
    ).toString();

    // assert expectations
    // alice gets paid
    test.is(
      alice_balance_difference.substring(0, 2),
      sale_price.substring(0, 2),
      "Expected alice balance to roughly increase by sale price"
    );
    // NFT has new owner
    const view_payload = {
      token_id: "TEST123",
    };
    const token_info = await nft_contract.view("nft_token", view_payload);
    test.is(token_info["owner_id"], bob.accountId, "NFT should have been sold");
    // nothing left for sale on market
    const sale_supply = await market_contract.view("get_supply_sales");
    test.is(sale_supply, "0", "Expected no sales to be left on market");
  }
);

workspace.test(
  "cross contract: transfer NFT when listed on marketplace",
  async (test, { nft_contract, market_contract, alice, bob, charlie }) => {
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
  }
);

workspace.test(
  "cross contract: approval revoke",
  async (test, { nft_contract, market_contract, alice, bob }) => {
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
      approval_id: 0,
    };
    const result = await market_contract.call_raw(
      nft_contract,
      "nft_transfer",
      transfer_payload,
      defaultCallOptions(DEFAULT_GAS, "1")
    );

    // assert expectations
    // Unauthorized error should be found
    test.regex(result.promiseErrorMessages.join("\n"), /Unauthorized+/);
  }
);

// TODO: Unhandled rejection error is triggered by having charlie purchase NFT from bob on market 
// workspace.test(
//   "cross contract: reselling and royalties",
//   async (test, { nft_contract, market_contract, alice, bob, charlie }) => {
//     const royalties_string = `{"${alice.accountId}":2000}`;
//     const royalties = JSON.parse(royalties_string);
//     test.log("royalties: ", royalties);
//     await mintNFT(alice, nft_contract, royalties);
//     await payForStorage(alice, market_contract);
//     const ask_price = "300000000000000000000000";
//     await placeNFTForSale(market_contract, alice, nft_contract, ask_price);

//     // offer for higher price
//     const alice_balance_before_offer = (await alice.balance()).total.toBigInt();
//     const bid_price = ask_price + "0";
//     await purchaseListedNFT(nft_contract, bob, market_contract, bid_price);
//     const alice_balance_after_offer = (await alice.balance()).total.toBigInt();
//     const alice_balance_difference = (
//       alice_balance_after_offer - alice_balance_before_offer
//     ).toString();

//     // assert alice gets paid
//     test.is(
//       alice_balance_difference.substring(0, 3),
//       bid_price.substring(0, 3),
//       "Expected alice balance to roughly increase by sale price"
//     );

//     // bob relists NFT for higher price
//     test.log("bob paying for storage");
//     await payForStorage(bob, market_contract);
//     const resell_ask_price = bid_price + "0";
//     test.log("bob placing NFT for sale");
//     await placeNFTForSale(market_contract, bob, nft_contract, resell_ask_price);
  
    // // bob updates price to lower ask
    // test.log("bob updating NFT price for lower");
    // const lowered_resell_ask_price = "600000000000000000000000";
    // const update_price_payload = {
    //   nft_contract_id: nft_contract,
    //   token_id: "TEST123",
    //   price: lowered_resell_ask_price,
    // };
    // await bob.call(
    //   market_contract,
    //   "update_price",
    //   update_price_payload,
    //   defaultCallOptions(DEFAULT_GAS, "1")
    // );

    // // charlie buys NFT from bob
    // test.log("charlie purchasing bobs NFT");
    // const alice_balance_before_offer_2 = (
    //   await alice.balance()
    // ).total.toBigInt();
    // const bob_balance_before_offer = (await bob.balance()).total.toBigInt();
    // test.log("bob_balance_before_offer", bob_balance_before_offer);
    // purchaseListedNFT(
    //   nft_contract,
    //   charlie,
    //   market_contract,
    //   resell_ask_price
    // );
    // const alice_balance_after_offer_2 = (
    //   await alice.balance()
    // ).total.toBigInt();
    // const alice_balance_difference_2 = (
    //   alice_balance_after_offer_2 - alice_balance_before_offer_2
    // ).toString();
    // const bob_balance_after_offer = (await bob.balance()).total.toBigInt();
    // test.log("bob_balance_after_offer", bob_balance_after_offer);
    // const bob_balance_difference = (
    //   bob_balance_after_offer - bob_balance_before_offer
    // ).toString();

    // // assert alice gets paid royalties
    // // TODO: this fails on sandbox and should pass 
    // test.is(
    //   alice_balance_difference_2.substring(0, 2),
    //   "120", // 20% of lowered_resell_ask_price
    //   "Expected bob balance to roughly increase by 80% of sale price"
    // );
    // // assert bob gets paid
    // test.is(
    //   bob_balance_difference.substring(0, 2),
    //   "480", // 80% of lowered_resell_ask_price
    //   "Expected bob balance to roughly increase by 80% of sale price"
    // );
//   }
// );
