import { Workspace } from "near-workspaces-ava";
import { BN, NearAccount } from "near-workspaces";

const DEFAULT_GAS: string = "30000000000000";
const DEFAULT_DEPOSIT: string = "8550000000000000000000";

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
    // pay for storage
    await alice.call(
      market_contract,
      "storage_deposit",
      {},
      defaultCallOptions(DEFAULT_GAS, "10000000000000000000000") // Requires minimum deposit of 10000000000000000000000
    );

    const sale_price = '"300000000000000000000000"'; // sale price string in yoctoNEAR is 0.3 NEAR
    await approveNFT(
      market_contract,
      alice,
      nft_contract,
      '{"sale_conditions": ' + sale_price + " }" // msg triggers XCC
    );

    // bob makes offer on listed nft
    const alice_balance_before_offer = (await alice.balance()).total.toBigInt();
    const offer_payload = {
      nft_contract_id: nft_contract,
      token_id: "TEST123",
    };
    await bob.call(
      market_contract,
      "offer",
      offer_payload,
      defaultCallOptions(
        DEFAULT_GAS + "0", // 10X default amount for XCC
        sale_price.replaceAll('"', "") // Attached deposit must be greater than or equal to the current price
      )
    );
    const alive_balance_after_offer = (await alice.balance()).total.toBigInt();
    const alice_balance_difference = (
      alive_balance_after_offer - alice_balance_before_offer
    ).toString();

    // assert expectations
    // alice gets paid
    test.is(
      alice_balance_difference.substring(0, 2),
      sale_price.replaceAll('"', "").substring(0, 2),
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
  async (test, { nft_contract, market_contract, alice, bob }) => {
    await mintNFT(alice, nft_contract);
    // pay for storage
    await alice.call(
      market_contract,
      "storage_deposit",
      {},
      defaultCallOptions(DEFAULT_GAS, "10000000000000000000000") // Requires minimum deposit of 10000000000000000000000
    );

    const sale_price = '"300000000000000000000000"'; // sale price string in yoctoNEAR is 0.3 NEAR
    await approveNFT(
      market_contract,
      alice,
      nft_contract,
      '{"sale_conditions": ' + sale_price + " }" // msg triggers XCC
    );

    // attempt to transfer NFT
    test.log("attempting to transfer NFT");
    const transfer_payload = {
      receiver_id: bob,
      token_id: "TEST123",
      approval_id: 0, // first and only approval done in line 224
    };
    const result = await market_contract.call_raw(
      nft_contract,
      "nft_transfer",
      transfer_payload,
      defaultCallOptions(DEFAULT_GAS, "1") // Requires attached deposit of exactly 1 yoctoNEAR
    );
    test.log("result: ", result);
    test.log("errors: ", result.promiseErrorMessages);

    // assert expectations
    // NFT has same owner
    const view_payload = {
      token_id: "TEST123",
    };
    const token_info = await nft_contract.view("nft_token", view_payload);
    test.is(
      token_info["owner_id"],
      alice.accountId,
      "NFT should have not changed owners"
    );
    // error should be found
    test.assert(
      result.promiseErrorMessages.length > 0,
      "expected to find error messages"
    );
  }
);

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

async function mintNFT(user: NearAccount, nft_contract: NearAccount) {
  const mint_payload = {
    token_id: "TEST123",
    metadata: {
      title: "LEEROYYYMMMJENKINSSS",
      description: "Alright time's up, let's do this.",
      media:
        "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1",
    },
    receiver_id: user,
  };
  await user.call(nft_contract, "nft_mint", mint_payload, defaultCallOptions());
}
