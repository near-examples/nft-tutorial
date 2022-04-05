import { defaultCallOptions, mintNFT, approveNFT } from "./utils";
import { workspace } from "./workspace";

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
