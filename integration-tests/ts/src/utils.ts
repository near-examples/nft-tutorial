import { BN, NearAccount } from "near-workspaces";

export const DEFAULT_GAS: string = "30000000000000";
export const DEFAULT_DEPOSIT: string = "9050000000000000000000";


export async function purchaseListedNFT(
  nft_contract: NearAccount,
  bidder_account: NearAccount,
  market_contract: NearAccount,
  bid_price: string
) {
  const offer_payload = {
    nft_contract_id: nft_contract,
    token_id: "TEST123",
  };
  await bidder_account.callRaw(
    market_contract,
    "offer",
    offer_payload,
    defaultCallOptions(DEFAULT_GAS + "0", bid_price)
  );
}

export async function placeNFTForSale(
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

export function defaultCallOptions(
  gas: string = DEFAULT_GAS,
  deposit: string = DEFAULT_DEPOSIT
) {
  return {
    gas: new BN(gas),
    attachedDeposit: new BN(deposit),
  };
}
export async function approveNFT(
  account_to_approve: NearAccount,
  owner: NearAccount,
  nft_contract: NearAccount,
  message?: string
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

export async function mintNFT(
  user: NearAccount,
  nft_contract: NearAccount,
  royalties?: object
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

export async function payForStorage(
  alice: NearAccount,
  market_contract: NearAccount
) {
  await alice.call(
    market_contract,
    "storage_deposit",
    {},
    defaultCallOptions(DEFAULT_GAS, "10000000000000000000000") // Requires minimum deposit of 10000000000000000000000
  );
}

export async function transferNFT(
  receiver: NearAccount,
  sender: NearAccount,
  nft_contract: NearAccount
) {
  const transfer_payload = {
    receiver_id: receiver,
    token_id: "TEST123",
    approval_id: 0, // first and only approval done in line 224
  };
  await sender.call(
    nft_contract,
    "nft_transfer",
    transfer_payload,
    defaultCallOptions(DEFAULT_GAS, "1")
  );
}
