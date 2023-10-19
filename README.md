## atomic-swaps

### Make Offer
If a maker wants to make an offer for an atomic swaps, they start out by making two transactions: 
1. An authz message to grant the contract permission to take necessary tokens from the Maker's account in the future.
   * This involves two authorizations, a `CoontractExecutionAuthorization` and a `SendAuthorization` 
   * These two authorizations enable the contract to execute against the contract on behalf of the `maker` and spend funds from the maker's wallet up to the specified limit.
   * These authorizations are handled outside the contract logic, ideally in the UI prompting for the maker to sign the tx to enable an easy UX, or they can be executed with `osmosisd` by the `maker` via their cli, as shown below: 
```shell
# Contract Execution: Allowing contract to execute itself on behalf of maker
osmosisd tx wasm grant [contract_addr] execution [contract_addr] --from=maker --[add-flags-as-needed]

# Bank Send: Allowing contract to spend maker's funds up to the provided spend limit
osmosisd tx authz grant [contract_addr] send --spend-limit=[maker_coin] --from=maker --[add-flags-as-needed]
```
2. A contract execute `MakeOffer` with a message specifying the offer to be saved in storage as:
```rust
pub struct Offer {
    pub maker: String,
    pub taker: Option<String>,
    pub maker_coin: Coin,
    pub taker_coin: Coin,
}
```

### Take Offer 
1. If a taker wants to browse current offers, they just query the chain with `AllOffers` or can view completed swaps with `FulfilledOffers`: 
```rust
pub enum QueryMsg {
   #[returns(Vec<Offer>)]
   AllOffers {
      start_after: Option<String>,
      limit: Option<u32>,
   },
   #[returns(Offer)]
   FulfilledOffers { offer_id: u64 },
}
```
2. Once the taker finds an offer they want to match, they execute `OfferTaker`, and they must send the correct funds in order to get their match request accepted. 
3. If the match is accepted, the contract executes `FulfillOffer` to swap both assets simultaneously in one transaction. 