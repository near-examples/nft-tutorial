import { Workspace } from "near-workspaces-ava";
import { BN } from "near-workspaces";

const workspace = Workspace.init(async ({ root }) => {
  const alice = await root.createAccount("alice");
  const bob = await root.createAccount("bob");
  const charlie = await root.createAccount("charlie");

  const main_contract = await root.createAndDeploy(
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

  return { alice, bob, charlie, main_contract, market_contract };
});

workspace.test(
  "main contract: nft metadata view",
  async (test, { main_contract, root }) => {
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
      await main_contract.view("nft_metadata", { account_id: root }),
      expected
    );
  }
);

workspace.test(
  "main contract: nft mint call",
  async (test, { main_contract, alice, root }) => {
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
    test.log("Request payload: ", request_payload);
    const options = {
      gas: new BN("75000000000000"), // min gas: https://stackoverflow.com/questions/70088651/near-executionerrorexceeded-the-prepaid-gas
      attachedDeposit: new BN("8550000000000000000001"), // Must attach 8550000000000000000000 yoctoNEAR to cover storage
    };
    test.log("Options: ", options);
    await root.call(main_contract, "nft_mint", request_payload, options);

    const tokens = await main_contract.view("nft_tokens");
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
  "main contract: nft approve call",
  async (test, { main_contract, market_contract, alice, root }) => {
    // mint NFT
    const mint_payload = {
      token_id: "TEST123",
      metadata: {
        title: "LEEROYYYMMMJENKINSSS",
        description: "Alright time's up, let's do this.",
        media:
          "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1",
      },
      receiver_id: alice,
    };
    const mint_options = {
      gas: new BN("75000000000000"), // min gas: https://stackoverflow.com/questions/70088651/near-executionerrorexceeded-the-prepaid-gas
      attachedDeposit: new BN("8550000000000000000001"), // Must attach 8550000000000000000000 yoctoNEAR to cover storage
    };
    await root.call(main_contract, "nft_mint", mint_payload, mint_options);

    // approve NFT
    const approve_payload = {
      token_id: "TEST123",
      account_id: market_contract,
    };
    const approve_options = {
      gas: new BN("75000000000000"), // min gas: https://stackoverflow.com/questions/70088651/near-executionerrorexceeded-the-prepaid-gas
      attachedDeposit: new BN("8550000000000000000001"), // Must attach 8550000000000000000000 yoctoNEAR to cover storage
    };
    await alice.call(
      main_contract,
      "nft_approve",
      approve_payload,
      approve_options
    );

    // test if approved
    const view_payload = {
      token_id: "TEST123",
      approved_account_id: market_contract,
    };
    const approved = await main_contract.view("nft_is_approved", view_payload);
    test.true(approved, "Failed to approve NFT");
  }
);

workspace.test(
  "main contract: nft approve call long msg string",
  async (test, { main_contract, market_contract, alice, root }) => {
    // mint NFT
    const mint_payload = {
      token_id: "TEST123",
      metadata: {
        title: "LEEROYYYMMMJENKINSSS",
        description: "Alright time's up, let's do this.",
        media:
          "https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Ftse3.mm.bing.net%2Fth%3Fid%3DOIP.Fhp4lHufCdTzTeGCAblOdgHaF7%26pid%3DApi&f=1",
      },
      receiver_id: alice,
    };
    const mint_options = {
      gas: new BN("75000000000000"), // min gas: https://stackoverflow.com/questions/70088651/near-executionerrorexceeded-the-prepaid-gas
      attachedDeposit: new BN("10000000000000000000001"), // Insufficient storage paid: 0, for 1 sales at 10000000000000000000000 rate of per sale
    };
    await root.call(main_contract, "nft_mint", mint_payload, mint_options);

    // approve NFT
    const approve_payload = {
      token_id: "TEST123",
      account_id: market_contract,
      msg: "sample message".repeat(10 * 1024),
    };
    const approve_options = {
      gas: new BN("75000000000000"), // min gas: https://stackoverflow.com/questions/70088651/near-executionerrorexceeded-the-prepaid-gas
      attachedDeposit: new BN("10000000000000000000001"), // Insufficient storage paid: 0, for 1 sales at 10000000000000000000000 rate of per sale
    };
    const result = await alice.call_raw(
      main_contract,
      "nft_approve",
      approve_payload,
      approve_options
    );
    test.regex(result.promiseErrorMessages.join("\n"), /Exceeded the prepaid gas+/);

    // test if approved
    const view_payload = {
      token_id: "TEST123",
      approved_account_id: market_contract,
    };
    const approved = await main_contract.view("nft_is_approved", view_payload);
    test.false(approved, "NFT approval should have failed");
  }
);
