# Secure-Squads
A cli for secure interations with squads-v4

The following is an overview of commands available to interact with the Squads V4 program via CLI.

Overview

1. [Installation](#1-installation)
2. [Supported wallets](#2-supported-wallets)
3. [Commands](#3-commands)
   - [Create multisig](#multisig-create)
   - [Vote on proposals](#proposal-vote)
   - [Reclaim Vault Transaction rent](#vault-transaction-accounts-close)
   - [Execute Vault Transaction](#vault-transaction-execute)

# 1. Installation

You can install the CLI with Cargo.
For this an installation of Rust will be needed. You can find installation steps [here](https://www.rust-lang.org/tools/install).

Now, install the Squads CLI.

```bash
cargo install secure-squads
```

# 2. Supported wallets

The Squads CLI has exactly the same wallet support as the Solana CLI, meaning it supports file system wallets as well as Ledger hardware wallets.

### File system wallets

You can easily use your local filesystem wallet by using it as the "keypair" argument in commands.

```bash
secure-squads example-command --keypair /path/to/keypair.json
```

This specifies the path of the Keypair that you want to use to sign a CLI transaction.

### Ledger support

To use a Ledger with the Squads CLI, just specify the Ledger device URL in the "keypair" argument.

```bash
secure-squads example-command --keypair usb://ledger
```

This will use the default derivation path of your Ledger.

```bash
secure-squads example-command --keypair usb://ledger/BsNsvfXqQTtJnagwFWdBS7FBXgnsK8VZ5CmuznN85swK?key=0/0
```

This specifies a custom derivation path. You can read more about it [here](https://docs.solana.com/wallet-guide/hardware-wallets/ledger).

# 3. Commands

## Multisig Create

### Description

Creates a new multisig with initial members and threshold configuration.

### Syntax

```bash
multisig-create --rpc-url <RPC_URL> --program-id <PROGRAM_ID> --keypair <KEYPAIR_PATH> --config-authority <CONFIG_AUTHORITY> --members <MEMBER_1> <MEMBER_2> ... --threshold <THRESHOLD>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--program-id <PROGRAM_ID>`: (Optional) The ID of the multisig program. Defaults to a standard ID if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--config-authority <CONFIG_AUTHORITY>`: (Optional) Address of the Program Config Authority.
- `--members <MEMBER_...>`: List of members' public keys, separated by spaces.
- `--threshold <THRESHOLD>`: The threshold number of signatures required for executing multisig transactions.
- `--rent-collector <RENT_COLLECTOR>` : The Public key that will be able to reclaim rent from canceled and executed transactions.

### Example Usage

1. **Creating a Multisig with Two Members:**

   ```bash
   multisig-create --keypair /path/to/keypair.json --members "Member1PubKey,Permission1" "Member2PubKey,Permission2" --threshold 2
   ```

   Creates a new multisig account with two members and a threshold of 2.

2. **Creating a Multisig with Config Authority:**

   ```bash
   multisig-create --keypair /path/to/keypair.json --config-authority <CONFIG_AUTHORITY_PUBKEY> --members "Member1PubKey,Permission1" "Member2PubKey,Permission2" --threshold 1
   ```

   Initializes a multisig account with a specified config authority and a threshold of 1.

3. **Creating a Multisig with Rent Collector:**
   ```bash
   multisig-create --keypair /path/to/keypair.json --config-authority <RENT_COLLECTOR_PUBKEY> --members "Member1PubKey,Permission1" "Member2PubKey,Permission2" --threshold 1
   ```
   Initializes a multisig account with a specified rent collector and a threshold of 1.
```console
👀 You're about to create a multisig, please review the details:

RPC Cluster URL:   https://api.devnet.solana.com
Program ID:        SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
Your Public Key:       AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5
Config authority: None
Members:
  - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5: All

⚙️ Config Parameters

Config Authority:  None
Threshold:          1
⚠️ WARNING: A threshold of 1 means that any member can execute transactions without any other approvals.
Rent Collector:     AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5
Members amount:      1

Generated random keypair for multisig: DY9hkdhCDv5Pa9uP6ui7nnxUkf4FGuqAx7Lmatapg9fe
Derived Multisig Key: 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb
Derived Program Config PDA: BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr
Treasury Account: HM5y4mz3Bt9JY9mr1hkyhnvqxSH4H2u2451j7Hc2dtvK
Do you want to proceed? yes

⠒ Sending transaction...                                                                                                                                                                    🔐 SECURITY-CRITICAL ACCOUNT ROLES:
  🛡️  Mutable Signers (Can modify state AND sign):
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5
  🔒 Read-Only Signers (Can view but not modify state):
    - DY9hkdhCDv5Pa9uP6ui7nnxUkf4FGuqAx7Lmatapg9fe
  ⚠️  Mutable Unsigned (Can modify state but don't sign):
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb
    - HM5y4mz3Bt9JY9mr1hkyhnvqxSH4H2u2451j7Hc2dtvK
  👀 Read-Only Unsigned (Can view state but don't sign):
    - 11111111111111111111111111111111
    - ComputeBudget111111111111111111111111111111
    - SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
    - BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr

🔍 INSPECTING SQUADS INSTRUCTIONS:

🛡️ SQUADS INSTRUCTION #2
  Program ID: SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
  📛 Instruction: multisigCreateV2
  🔑 Accounts Involved:
    - BSTq9w3kZwNwpBXJEvTZz2G9ZTNyKBvoSeXMvwb4cNZr: programConfig (READONLY UNSIGNED)
    - DY9hkdhCDv5Pa9uP6ui7nnxUkf4FGuqAx7Lmatapg9fe: createKey (READONLY SIGNER)
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5: creator (MUTABLE SIGNER)
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb: multisig (MUTABLE UNSIGNED)
    - 11111111111111111111111111111111: systemProgram (READONLY UNSIGNED)
    - HM5y4mz3Bt9JY9mr1hkyhnvqxSH4H2u2451j7Hc2dtvK: treasury (MUTABLE UNSIGNED)
  🔓 Decoded Arguments:
{
  "args": {
    "configAuthority": null,
    "members": [
      {
        "key": "AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5",
        "permissions": {
          "mask": 7
        }
      }
    ],
    "memo": null,
    "rentCollector": "AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5",
    "threshold": 1,
    "timeLock": 0
  }
}
✅ Transaction details processed successfully!
⠋ Sending transaction...                                                                                                                                                                    Transaction confirmed: 58fV2i4rWWg8GoCbha9q39YdzdxTTtZe5dGaWGNN2rc5r3nzCnAETTijA5NEVzbM8MYWHyBVurrHyXZr4EJ3Mbf8


✅ Created Multisig: 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb. Signature: 58fV2i4rWWg8GoCbha9q39YdzdxTTtZe5dGaWGNN2rc5r3nzCnAETTijA5NEVzbM8MYWHyBVurrHyXZr4EJ3Mbf8

```
## Initiate Transfer

### Description

Move funds out of the Vault.

### Syntax

```bash
initiate-transfer --rpc-url <RPC_URL> --token-mint-address <TOKEN_MINT> --token-amount-u64 <AMOUNT_IN_LAMPORTS> --recipient <RECIPIENT_PUBKEY> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBKEY> --vault-index <VAULT_INDEX> 
```
### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--program-id <PROGRAM_ID>`: (Optional) The ID of the multisig program. Defaults to a standard ID if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--token-mint-address <TOKEN_MINT>`: Token Mint
- `--token-amount-u64 <AMOUNT_IN_LAMPORTS>`: Amount to Sent
- `--recipient <RECIPIENT_PUBKEY>` : Recipient Address
- `-vault-index <VAULT_INDEX> `: Index of the Vault


### Example Usage

1. **Sol Transfer:**

   ```bash
   initiate-transfer --rpc-url <RPC_URL> --token-mint-address So11111111111111111111111111111111111111112 --token-amount-u64 10000 --recipient <RECIPIENT_PUBKEY> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --vault-index 0  
   ```
```console
👀 You're about to create a vault transaction, please review the details:

RPC Cluster URL:   https://api.devnet.solana.com
Program ID:        SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
Your Public Key:       AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5

⚙️ Config Parameters
Multisig Key:       3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb
Transaction Index:       1
Vault Index:       0

Do you want to proceed? yes                                                                                                                                                                   Token Amount: 10000
Authority pubkey: 24iiwyZYoWWHwJpF8wBG8GH8kzvTLpeAkeUhsHaRc2Sq
🔐 SECURITY-CRITICAL ACCOUNT ROLES:
  🛡️  Mutable Signers (Can modify state AND sign):
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5
  🔒 Read-Only Signers (Can view but not modify state):
  ⚠️  Mutable Unsigned (Can modify state but don't sign):
    - 32VejooGNcswcQHeTGhkPMauHyXKeXwJKoWCCvAHKCZG
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb
    - 9pZA68kxFun6bVuxsC55i9thTtnZ9woC3B3Y1PZtRVnr
  👀 Read-Only Unsigned (Can view state but don't sign):
    - 11111111111111111111111111111111
    - ComputeBudget111111111111111111111111111111
    - SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf

🔍 INSPECTING SQUADS INSTRUCTIONS:

🛡️ SQUADS INSTRUCTION #2
  Program ID: SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
  📛 Instruction: vaultTransactionCreate
  🔑 Accounts Involved:
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5: rentPayer (MUTABLE SIGNER)
    - 11111111111111111111111111111111: systemProgram (READONLY UNSIGNED)
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb: multisig (MUTABLE UNSIGNED)
    - 9pZA68kxFun6bVuxsC55i9thTtnZ9woC3B3Y1PZtRVnr: transaction (MUTABLE UNSIGNED)
  🔓 Decoded Arguments:
{
  "args": {
    "ephemeralSigners": 0,
    "memo": null,
    "transactionMessage": [
      1,
      1,
      2,
      4,
      -,
      -,
      -,
      0
    ],
    "vaultIndex": 0
  }
}

🛡️ SQUADS INSTRUCTION #3
  Program ID: SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
  📛 Instruction: proposalCreate
  🔑 Accounts Involved:
    - 32VejooGNcswcQHeTGhkPMauHyXKeXwJKoWCCvAHKCZG: proposal (MUTABLE UNSIGNED)
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb: multisig (MUTABLE UNSIGNED)
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5: rentPayer (MUTABLE SIGNER)
    - 11111111111111111111111111111111: systemProgram (READONLY UNSIGNED)
  🔓 Decoded Arguments:
{
  "args": {
    "draft": false,
    "transactionIndex": 1
  }
}
TransactionMessage:
  Signers: total=1, writable=1, writable_non_signers=2
  Address Table Lookups:
parse toke ix info {"amount":"10000","destination":"2z9yxtP7bPARjRXPAeiR7HAR2onSP8UBtXABX9qQXKSK","multisigAuthority":"24iiwyZYoWWHwJpF8wBG8GH8kzvTLpeAkeUhsHaRc2Sq","signers":["24iiwyZYoWWHwJpF8wBG8GH8kzvTLpeAkeUhsHaRc2Sq"],"source":"HPDP1S4SW6bjzzAApdMu9PjRjYDytkqZe443DvjTXiD9"}
⠤ Sending transaction...                                                                                                                                                                    Transaction confirmed: zmecHjVw8Gw36EWMqEubTNimt73W4cHEZ4T5dhwiimsStN1EX61eXoGuRT6TYPML2JoWjqtayaK6gSb3yaVetWW


✅ Transaction created successfully. Signature: zmecHjVw8Gw36EWMqEubTNimt73W4cHEZ4T5dhwiimsStN1EX61eXoGuRT6TYPML2JoWjqtayaK6gSb3yaVetWW

```

   
## Proposal Vote

### Description

Casts a vote on a proposed transaction proposal. This command allows a member of a multisig to approve, reject, or cancel a transaction proposal.

### Syntax

```bash
proposal-vote --rpc_url <RPC_URL> --program-id <PROGRAM_ID> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index <TRANSACTION_INDEX> --action <ACTION> [--memo <MEMO>]
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--program-id <PROGRAM_ID>`: (Optional) The ID of the multisig program. Defaults to a standard ID if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-index <TRANSACTION_INDEX>`: The index of the transaction to vote on.
- `--action <ACTION>`: The vote action to cast (Approve, Reject, Cancel).
- `--memo <MEMO>`: (Optional) A memo for the vote.

### Example Usage

1. **Approving a Transaction:**

   ```bash
   proposal-vote --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction_index 1 --action Approve
   ```

   Casts an approval vote for the transaction at index 1 in the specified multisig account.

2. **Rejecting a Transaction:**

   ```bash
   proposal-vote --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction_index 1 --action Reject
   ```

   Casts a rejection vote for the transaction at index 1.

3. **Cancelling a Transaction:**
   ```bash
   proposal-vote --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1 --action Cancel
   ```
   Cancels the transaction at index 1 in the multisig account.

   ```console
   
RPC Cluster URL:   https://api.devnet.solana.com
Program ID:        SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
Your Public Key:       AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5

⚙️ Config Parameters
Multisig Key:       3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb
Transaction Index:       1
Vote Type:       ap

Do you want to proceed? yes

⠤ Sending transaction...                                                                                                                                                                    🔐 SECURITY-CRITICAL ACCOUNT ROLES:
  🛡️  Mutable Signers (Can modify state AND sign):
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5
  🔒 Read-Only Signers (Can view but not modify state):
  ⚠️  Mutable Unsigned (Can modify state but don't sign):
    - 32VejooGNcswcQHeTGhkPMauHyXKeXwJKoWCCvAHKCZG
  👀 Read-Only Unsigned (Can view state but don't sign):
    - ComputeBudget111111111111111111111111111111
    - SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb

🔍 INSPECTING SQUADS INSTRUCTIONS:

🛡️ SQUADS INSTRUCTION #2
  Program ID: SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf
  📛 Instruction: proposalApprove
  🔑 Accounts Involved:
    - 32VejooGNcswcQHeTGhkPMauHyXKeXwJKoWCCvAHKCZG: proposal (MUTABLE UNSIGNED)
    - 3dVd1QQ4nTfCZUoq4jCdx7aforCNkLrmrsb5Y7a7PiTb: multisig (READONLY UNSIGNED)
    - AgZ9okAAA7sHz6ddJnuq6RFHXuEQZt3CgBZsNGHByjq5: member (MUTABLE SIGNER)
  🔓 Decoded Arguments:
{
  "args": {
    "memo": null
  }
}
⠁ Sending transaction...                                                                                                                                                                    Transaction confirmed: 42hDTuzRDPQoSeCXQHdr6QnsUiUN7T85y6zAn8h9XEcArhrdgTsoaDzF3qtkHomPNhU35SL6nWt7HdviNTDSg9fe


✅ Casted ap vote. Signature: 42hDTuzRDPQoSeCXQHdr6QnsUiUN7T85y6zAn8h9XEcArhrdgTsoaDzF3qtkHomPNhU35SL6nWt7HdviNTDSg9fe

   ```
## Vault Transaction Accounts Close

### Description

Closes the proposal and transaction accounts associated with a specific Vault Transaction. The rent will be returned to the multisigs "rent_collector".

### Syntax

```bash
vault-transaction_accounts-close --rpc_url <RPC_URL> --program-id <PROGRAM_ID> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index <TRANSACTION_INDEX> --rent-collector <RENT_COLLECTOR_PUBKEY>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--program-id <PROGRAM_ID>`: (Optional) The ID of the multisig program. Defaults to a standard ID if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-index <TRANSACTION_INDEX>`: The index of the transaction whose accounts are to be closed.
- `--rent-collector <RENT_COLLECTOR_PUBKEY>`: The public key of the account responsible for collecting rent.

### Example Usage

```bash
vault-transaction-accounts-close --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1 --rent-collector <RENT_COLLECTOR_PUBKEY>
```

In this example, the command closes the transaction accounts for the transaction at index 1 in the specified multisig account and collects rent using the provided rent collector public key.

## Vault Transaction Execute

### Description

Executes a transaction once its proposal has reachen threshold.

### Syntax

```bash
vault-transaction-execute --rpc-url <RPC_URL> --program-id <PROGRAM_ID> --keypair <KEYPAIR_PATH> --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index <TRANSACTION_INDEX>
```

### Parameters

- `--rpc-url <RPC_URL>`: (Optional) The URL of the Solana RPC endpoint. Defaults to mainnet if not specified.
- `--program-id <PROGRAM_ID>`: (Optional) The ID of the multisig program. Defaults to a standard ID if not specified.
- `--keypair <KEYPAIR_PATH>`: Path to your keypair file.
- `--multisig-pubkey <MULTISIG_PUBLIC_KEY>`: The public key of the multisig account.
- `--transaction-index <TRANSACTION_INDEX>`: The index of the transaction to be executed.

### Example Usage

```bash
vault-transaction-execute --keypair /path/to/keypair.json --multisig-pubkey <MULTISIG_PUBLIC_KEY> --transaction-index 1
```

This example executes the transaction at index 1 in the specified multisig.
