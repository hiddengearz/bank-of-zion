# Introduction

The Bank Of Zion team has hired you to audit their bank_of_zion Automated Money Maker program. They've created a created a state of the art, bug free algorithim that utilizes oracles to get the market price of tokens and utilizes the price data when calculating the protocol's price for that token.

The program has 2 roles and 6 instructions.

Admin:
* Initialize: Initialize the swap pool
* AdminDeposit: Deposit tokens priced at market price in exchange for swap pool tokens. To be used after initializing the swap pool or during an emergency.

User:
* Deposit: Deposit tokens priced at protocol price in exchange for swap pool tokens.
* Withdraw: Exchange swap tokens for tokens of equal or less value.
* Swap: Swap token A for Token B.

Scope:
* programs/bank_of_zion

Out Of Scope:
* programs/pyth

Known Issues:
* Fee's haven't been implemented in this version of the program.

### Appendix

* Market Price: The price provided by the oracle for a token
* Protocol/Local price: The price the program values the token at.
