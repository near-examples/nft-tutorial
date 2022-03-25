import { Workspace } from "near-workspaces-ava";

const workspace = Workspace.init(async ({ root }) => {
  const alice = await root.createAccount("alice");

  const main_contract = await root.createAndDeploy(
    "nft-contract",
    "../out/main.wasm",
    {
      method: "new_default_meta",
      args: { owner_id: root },
    }
  );

  // await alice.call(main_contract, 'set_status', {message: 'hello'});

  const market_contract = await root.createAndDeploy(
    "nft-market",
    "../out/market.wasm"

    // TODO
    // Provide `method` and `args` to call in the same transaction as the deploy
    // {
    //   method: 'init',
    //   args: {owner_id: root},
    // },
  );

  return { alice, main_contract, market_contract };
});

workspace.test("main contract: nft metadata view call", async (test, { main_contract, root }) => {
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
});
