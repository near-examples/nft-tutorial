import { Workspace } from "near-workspaces-ava";

export const workspace = Workspace.init(async ({ root }) => {
  const alice = await root.createAccount("alice", {
    initialBalance: "300000000000000000000000000",
  });
  const bob = await root.createAccount("bob", {
    initialBalance: "300000000000000000000000000",
  });
  const charlie = await root.createAccount("charlie", {
    initialBalance: "300000000000000000000000000",
  });

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
