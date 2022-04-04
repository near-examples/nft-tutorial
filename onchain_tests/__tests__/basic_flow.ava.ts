import { Workspace } from "near-workspaces-ava";
import { BN, NearAccount } from "near-workspaces";

const DEFAULT_GAS: string = "30000000000000";
const DEFAULT_DEPOSIT: string = "9050000000000000000000";

const workspace = Workspace.init(async ({ root }) => {
  const alice = await root.createAccount("alice");
  const bob = await root.createAccount("bob");
  const charlie = await root.createAccount("charlie");

  const nft_contract = await root.createAndDeploy(
    "nft-contract",
    "../out/main.wasm",
    {
      method: "new_default_meta",
      args: { owner_id: root },
    }
  );

  const market_contract = await root.createAndDeploy(
    "nft-market",
    "../out/market.wasm",
    {
      method: "new",
      args: { owner_id: root },
    }
  );

  return { alice, bob, charlie, nft_contract, market_contract };
});

workspace.test(
  "nft contract: nft metadata view",
  async (test, { nft_contract, root }) => {
    const expected = {
      base_uri: null,
      icon: null,
      name: "NFT Tutorial Contract",
      reference: null,
      reference_hash: null,
      spec: "nft-1.0.0",
      symbol: "GOTEAM",
    };
    test.deepEqual(
      await nft_contract.view("nft_metadata", { account_id: root }),
      expected
    );
  }
);

workspace.test(
  "nft contract: nft mint call",
  async (test, { nft_contract, alice }) => {
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
    test.deepEqual(tokens, expected, "Expected to find one minted NFT");
  }
);

workspace.test(
  "nft contract: nft approve call",
  async (test, { nft_contract, market_contract, alice }) => {
    await mintNFT(alice, nft_contract);
    await approveNFT(market_contract, alice, nft_contract);

    // test if approved
    const view_payload = {
      token_id: "TEST123",
      approved_account_id: market_contract,
    };
    const approved = await nft_contract.view("nft_is_approved", view_payload);
    test.true(approved, "Failed to approve NFT");
  }
);

workspace.test(
  "nft contract: nft approve call long msg string",
  async (test, { nft_contract, market_contract, alice }) => {
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
    test.regex(
      result.promiseErrorMessages.join("\n"),
      /Exceeded the prepaid gas+/
    );

    // test if approved
    const view_payload = {
      token_id: "TEST123",
      approved_account_id: market_contract,
    };
    const approved = await nft_contract.view("nft_is_approved", view_payload);
    test.false(approved, "NFT approval should have failed");
  }
);

workspace.test(
  "cross contract: sell NFT on listed on marketplace",
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

workspace.test(
  "cross contract: bid above ask, reselling and royalties",
  async (test, { nft_contract, market_contract, alice, bob, charlie }) => {
    const royalties_string = `{"${alice.accountId}":2000}`;
    const royalties = JSON.parse(royalties_string);
    test.log("royalties: ", royalties);
    await mintNFT(alice, nft_contract, royalties);
    await payForStorage(alice, market_contract);
    const ask_price = "300000000000000000000000";
    await placeNFTForSale(market_contract, alice, nft_contract, ask_price);

    // offer for higher price
    const alice_balance_before_offer = (await alice.balance()).total.toBigInt();
    const bid_price = ask_price + "0";
    await purchaseListedNFT(nft_contract, bob, market_contract, bid_price);
    const alice_balance_after_offer = (await alice.balance()).total.toBigInt();
    const alice_balance_difference = (
      alice_balance_after_offer - alice_balance_before_offer
    ).toString();

    // assert alice gets paid
    test.is(
      alice_balance_difference.substring(0, 3),
      bid_price.substring(0, 3),
      "Expected alice balance to roughly increase by sale price"
    );

    // bob relists NFT for higher price
    test.log("bob paying for storage");
    await payForStorage(bob, market_contract);
    const resell_ask_price = bid_price + "000000000000";
    test.log("bob placing NFT for sale");
    await placeNFTForSale(market_contract, bob, nft_contract, resell_ask_price);
    // bob updates price to lower ask
    test.log("bob updating NFT price for lower");
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

    // charlie buys NFT from bob
    test.log("charlie purchasing bobs NFT");
    const alice_balance_before_offer_2 = (
      await alice.balance()
    ).total.toBigInt();
    const bob_balance_before_offer = (await bob.balance()).total.toBigInt();
    test.log("bob_balance_before_offer", bob_balance_before_offer);
    purchaseListedNFT(
      nft_contract,
      charlie,
      market_contract,
      lowered_resell_ask_price
    );
    const alice_balance_after_offer_2 = (
      await alice.balance()
    ).total.toBigInt();
    const alice_balance_difference_2 = (
      alice_balance_after_offer_2 - alice_balance_before_offer_2
    ).toString();
    const bob_balance_after_offer = (await bob.balance()).total.toBigInt();
    test.log("bob_balance_after_offer", bob_balance_after_offer);
    const bob_balance_difference = (
      bob_balance_after_offer - bob_balance_before_offer
    ).toString();

    // assert alice gets paid royalties
    // TODO: this should pass, currenlty fails
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
  }
);

async function purchaseListedNFT(
  nft_contract: NearAccount,
  bidder_account: NearAccount,
  market_contract: NearAccount,
  bid_price: string
) {
  const offer_payload = {
    nft_contract_id: nft_contract,
    token_id: "TEST123",
  };
  await bidder_account.call(
    market_contract,
    "offer",
    offer_payload,
    defaultCallOptions(DEFAULT_GAS + "0", bid_price)
  );
}

async function placeNFTForSale(
  market_contract: NearAccount,
  owner: NearAccount,
  nft_contract: NearAccount,
  ask_price: string // sale price string in yoctoNEAR
) {
  await approveNFT(
    market_contract,
    owner,
    nft_contract,
    '{"sale_conditions": ' + `"${ask_price}"` + " }" // msg string trigger XCC
  );
}

function defaultCallOptions(
  gas: string = DEFAULT_GAS,
  deposit: string = DEFAULT_DEPOSIT
) {
  return {
    gas: new BN(gas),
    attachedDeposit: new BN(deposit),
  };
}

async function approveNFT(
  account_to_approve: NearAccount,
  owner: NearAccount,
  nft_contract: NearAccount,
  message: string = null
) {
  const approve_payload = {
    token_id: "TEST123",
    account_id: account_to_approve,
    msg: message,
  };
  await owner.call(
    nft_contract,
    "nft_approve",
    approve_payload,
    defaultCallOptions()
  );
}

async function mintNFT(
  user: NearAccount,
  nft_contract: NearAccount,
  royalties: object = null
) {
  const mint_payload = {
    token_id: "TEST123",
    metadata: {
      title: "LEEROYYYMMMJENKINSSS",
      description: "Alright time's up, let's do this.",
      media:
        "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1",
    },
    receiver_id: user,
    perpetual_royalties: royalties,
  };
  await user.call(nft_contract, "nft_mint", mint_payload, defaultCallOptions());
}

async function payForStorage(alice: NearAccount, market_contract: NearAccount) {
  await alice.call(
    market_contract,
    "storage_deposit",
    {},
    defaultCallOptions(DEFAULT_GAS, "10000000000000000000000") // Requires minimum deposit of 10000000000000000000000
  );
}

async function transferNFT(
  transfer_to_account: NearAccount,
  executing_account: NearAccount,
  nft_contract: NearAccount
) {
  const transfer_payload = {
    receiver_id: transfer_to_account,
    token_id: "TEST123",
    approval_id: 0, // first and only approval done in line 224
  };
  await executing_account.call(
    nft_contract,
    "nft_transfer",
    transfer_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );
}
