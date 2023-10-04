## atomic-swaps

The atomic swaps protocol allows a user to offer a swap of tokens to anyone that can match their offer.
The user making the offer is called the "maker". The user that wants to match the offer is called
the "taker".
To avoid requiring makers from keeping their tokens in escrow until their offers are matched, we
can take advantage of authz so that the contract is authorized to move the required amount of
tokens from the user's account into the contract at the time the offer is matched.