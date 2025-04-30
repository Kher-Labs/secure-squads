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
![Screenshot From 2025-04-30 23-54-54](https://github.com/user-attachments/assets/9f2c3f50-aab3-49ea-8cd1-5d25d2cd0a68)

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
