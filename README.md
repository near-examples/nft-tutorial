# NEAR NFT-Tutorial

Welcome to NEAR's NFT tutorial, where we will help you parse the details around NEAR's [NEP-171 standard](https://nomicon.io/Standards/NonFungibleToken/Core.html) (Non-Fungible Token Standard), and show you how to build your own NFT smart contract from the ground up, improving your understanding about the NFT standard along the way. 

## Prerequisites

* [NEAR Wallet Account](wallet.testnet.near.org)
* [Rust Toolchain](https://docs.near.org/develop/prerequisites)
* [NEAR-CLI](https://docs.near.org/tools/near-cli#setup)
* [yarn](https://classic.yarnpkg.com/en/docs/install#mac-stable)

## Tutorial Stages 

Each branch you will find in this repo corresponds to various stages of this tutorial with a partially completed contract at each stage. You are welcome to start from any stage you want to learn the most about. 



| Branch        | Docs Tutorial                                                                                    | Description |
| ------------- | ------------------------------------------------------------------------------------------------ | ----------- |
| 1.skeleton    | [Contract Architecture](https://docs.near.org/docs/tutorials/contracts/nfts/rs/skeleton) | You'll learn the basic architecture of the NFT smart contract, and you'll compile this skeleton code with the Rust toolchain.            |
| 2.minting     | [Minting](https://docs.near.org/docs/tutorials/contracts/nfts/rs/minting)                |Here you'll flesh out the skeleton so the smart contract can mint a non-fungible token             |
| 3.enumeration | [Enumeration](https://docs.near.org/docs/tutorials/contracts/nfts/rs/enumeration)        |    Here you'll find different enumeration methods that can be used to return the smart contract's states.          |
| 4.core        | [Core](https://docs.near.org/docs/tutorials/contracts/nfts/rs/core)                      | In this tutorial you'll extend the NFT contract using the core standard, which will allow you to transfer non-fungible tokens.             |
| 5.approval    | [Approval](https://docs.near.org/docs/tutorials/contracts/nfts/rs/approvals)             | Here you'll expand the contract allowing other accounts to transfer NFTs on your behalf.            |
| 6.royalty     | [Royalty](https://docs.near.org/docs/tutorials/contracts/nfts/rs/royalty)                                                                                        |Here you'll add the ability for non-fungible tokens to have royalties. This will allow people to get a percentage of the purchase price when an NFT is purchased.             |
| 7.events      | -----------                                                                                        |   This allows indexers to know what functions are being called and make it easier and more reliable to keep track of information that can be used to populate the collectibles tab in the wallet for example. (tutorial docs have yet to be implemented )          |
| 8.marketplace | -----------                                                                                      | -----------             |


The tutorial series also contains a very helpful section on [**Upgrading Smart Contracts**](https://docs.near.org/docs/tutorials/contracts/nfts/rs/upgrade-contract). Definitely go and check it out as this is a common pain point.

# Quick-Start 

If you want to see the full completed contract go ahead and clone and build this repo using 

```=bash
git clone https://github.com/near-examples/nft-tutorial.git 
cd nft-tutorial
git switch 6.royalty
yarn build
```

Now that you've cloned and built the contract we can try a few things. 

## Mint An NFT

Once you've created your near wallet go ahead and login to your wallet with your cli and follow the on-screen prompts

```=bash
near login
```

Once your logged in you have to deploy the contract. Make a subaccount with the name of your choosing 

```=bash 
near create-account nft-example.your-account.testnet --masterAccount your-account.testnet --initialBalance 10
```

After you've created your sub account deploy the contract to that sub account, set this variable to your sub account name

```=bash
NFT_CONTRACT_ID=nft-example.your-account.testnet

MAIN_ACCOUNT=your-account.testnet
```

Verify your new variable has the correct value
```=bash
echo $NFT_CONTRACT_ID

echo $MAIN_ACCOUNT
```


### Deploy Your Contract
```=bash
near deploy --accountId $NFT_CONTRACT_ID --wasmFile out/main.wasm
```

### Initialize Your Contract 

```=bash
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$NFT_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
```

### View Contracts Meta Data

```=bash
near view $NFT_CONTRACT_ID nft_metadata
```
### Minting Token

```bash=
near call $NFT_CONTRACT_ID nft_mint '{"token_id": "token-1", "metadata": {"title": "My Non Fungible Team Token", "description": "The Team Most Certainly Goes :)", "media": "https://bafybeiftczwrtyr3k7a2k4vutd3amkwsmaqyhrdzlhvpt33dyjivufqusq.ipfs.dweb.link/goteam-gif.gif"}, "receiver_id": "'$MAIN_ACCOUNT'"}' --accountId $MAIN_ACCOUNT --amount 0.1
```

After you've minted the token go to wallet.testnet.near.org to `your-account.testnet` and look in the collections tab and check out your new sample NFT! 



## View NFT Information

After you've minted your NFT you can make a view call to get a response containing the `token_id` `owner_id` and the `metadata`

```bash=
near view $NFT_CONTRACT_ID nft_token '{"token_id": "token-1"}'
```

## Transfering NFTs

To transfer an NFT go ahead and make another [testnet wallet account](https://wallet.testnet.near.org).

Then run the following
```bash=
MAIN_ACCOUNT_2=your-second-wallet-account.testnet
```

Verify the correct variable names with this

```=bash
echo $NFT_CONTRACT_ID

echo $MAIN_ACCOUNT

echo $MAIN_ACCOUNT_2
```

To initiate the transfer..

```bash=
near call $NFT_CONTRACT_ID nft_transfer '{"receiver_id": "$MAIN_ACCOUNT_2", "token_id": "token-1", "memo": "Go Team :)"}' --accountId $MAIN_ACCOUNT --depositYocto 1
```

In this call you are depositing 1 yoctoNEAR for security and so that the user will be redirected to the NEAR wallet.

## Errata

Large Changes:
* **2022-06-21**: updated the rust SDK to version 4.0.0. PR found [here](https://github.com/near-examples/nft-tutorial/pull/32)

* **2022-02-12**: updated the enumeration methods `nft_tokens` and `nft_tokens_for_owner` to no longer use any `to_vector` operations to save GAS. In addition, the default limit was changed from 0 to 50. PR found [here](https://github.com/near-examples/nft-tutorial/pull/17). 

Small Changes:
* **2022-02-22**: changed `token_id` parameter type in nft_payout from `String` to `TokenId` for consistency as per pythonicode's suggestion
