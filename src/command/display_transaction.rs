use clap::Args;
use colored::Colorize;
use serde_json::Value;
use serde_json::to_string_pretty;
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::message::v0::LoadedAddresses;
use solana_sdk::message::v0::MessageAddressTableLookup;
use solana_sdk::message::{AccountKeys, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::parse_accounts::parse_v0_message_accounts;
use solana_transaction_status::parse_instruction::parse;
use squads_multisig::anchor_lang::AnchorDeserialize;
use squads_multisig::anchor_lang::{AccountDeserialize, InstructionData};
use squads_multisig::pda::{
    get_ephemeral_signer_pda, get_multisig_pda, get_proposal_pda, get_transaction_pda,
    get_vault_pda,
};
use squads_multisig::squads_multisig_program::{
    MessageAddressTableLookup as Squads_MessageAddressTableLookup, TransactionMessage,
};
use squads_multisig::state::{MultisigCompiledInstruction, MultisigMessageAddressTableLookup};
use std::borrow::Cow;
use std::collections::HashSet;
use std::io::Cursor;
//use squads_multisig::anchor_lang::{AccountDeserialize, InstructionData};
//use squads_multisig::pda::get_transaction_pda;
use squads_multisig::solana_client::nonblocking::rpc_client::RpcClient;
use squads_multisig::squads_multisig_program::state::VaultTransaction;
use std::str::FromStr;

#[derive(Args)]
pub struct DisplayTransaction {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Multisig Program ID
    #[arg(long)]
    program_id: Option<String>,

    /// Path to the Program Config Initializer Keypair
    #[arg(long)]
    multisig_address: String,

    // index to derive the tx
    #[arg(long)]
    transaction_index: u64,
}

impl DisplayTransaction {
    // vault genrated for (seeds, multisig_address, vault_index)
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            program_id,
            multisig_address,
            transaction_index,
        } = self;

        let program_id =
            program_id.unwrap_or_else(|| "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf".to_string());

        let program_id = Pubkey::from_str(&program_id).expect("Invalid program ID");

        let multisig_address =
            Pubkey::from_str(&multisig_address).expect("Invalid multisig address");

        let transaction_pda =
            get_transaction_pda(&multisig_address, transaction_index, Some(&program_id));

        println!("tx: {:?}", transaction_pda.0);
        // Initialize RPC client
        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
        let rpc_client = RpcClient::new(rpc_url.to_string());
        let transaction_account_data = rpc_client
            .get_account(&transaction_pda.0)
            .await
            .expect("Failed to get transaction account")
            .data;

        let mut transaction_account_data_slice = transaction_account_data.as_slice();

        let deserialized_account_data =
            VaultTransaction::try_deserialize(&mut transaction_account_data_slice).unwrap();

        let transaction_message = deserialized_account_data.message;
        println!(
            "Transaction is proposed by: {}",
            deserialized_account_data.creator.to_string().bright_green()
        );
        let ephemeral_signer_count = deserialized_account_data.ephemeral_signer_bumps.len();
        if ephemeral_signer_count > 0 {
            println!(
                "  Additional Signers: {} ephemeral accounts (used for signing program instructions)",
                ephemeral_signer_count
            );
        } else {
            println!("  Additional Signers: None");
        }
        if !transaction_message.address_table_lookups.is_empty() {
            println!("🔍 Address Table Lookups:");
            for lookup in &transaction_message.address_table_lookups {
                println!(
                    "  Account Key: {}, Writable Indexes: {:?}, Readonly Indexes: {:?}",
                    lookup.account_key, lookup.writable_indexes, lookup.readonly_indexes
                );
            }
        } else {
            println!("🔍 Address Table Lookups: None");
        }
        /*
        pub struct LoadedMessage<'a> {
            pub message: Cow<'a, Message>,
            pub loaded_addresses: Cow<'a, LoadedAddresses>,
            pub is_writable_account_cache: Vec<bool>,
        }
            constuct a load message object for our tx , so that i can use parse_account();
        */
        let multisig_account = rpc_client.get_account(&multisig_address).await.unwrap();
        println!("Multisig Account:  {}", multisig_account.owner);
        let transaction_account_data = rpc_client
            .get_account(&transaction_pda.0)
            .await
            .expect("Failed to get transaction account")
            .data;
        let v0message: Cow<'_, Message> = Cow::Owned(into_v0_message(transaction_message.clone()));

        // let reserved_account_keys = &HashSet::default();

        let vault_pda = get_vault_pda(
            &multisig_address,
            deserialized_account_data.vault_index,
            Some(&program_id),
        );
        // in general remaning accoutn metas
        let (account_metas, address_lookup_table_accounts) = message_to_execute_account_metas(
            &rpc_client,
            transaction_message.clone(),
            deserialized_account_data.ephemeral_signer_bumps,
            &vault_pda.0,
            &transaction_pda.0,
            Some(&program_id),
        )
        .await;

        // Extract loaded addresses
        let loaded_addresses =
            extract_loaded_addresses(&address_lookup_table_accounts, &transaction_message);
        let loaded_message = solana_sdk::message::v0::LoadedMessage::new(
            v0message.into_owned(),
            loaded_addresses,
            //  &reserved_account_keys,
        );
        let parsed_accounts = parse_v0_message_accounts(&loaded_message);

        println!("Parsed Accounts:");
        for (i, account) in parsed_accounts.iter().enumerate() {
            println!("  {}: {:?}", i, account);
        }

        println!("TransactionMessage:");
        println!(
            "  Signers: total={}, writable={}, writable_non_signers={}",
            transaction_message.num_signers,
            transaction_message.num_writable_signers,
            transaction_message.num_writable_non_signers,
        );
        let account_keys: Vec<Pubkey> = transaction_message.account_keys.clone();
        println!("🔒 Account Classification:");
        println!(
            "  Mutable Signers: {}",
            format!(
                "{:?}",
                &account_keys[..transaction_message.num_writable_signers as usize]
            )
            .red()
        );
        println!(
            "  Read-Only Signers: {}",
            format!(
                "{:?}",
                &account_keys[transaction_message.num_writable_signers as usize
                    ..transaction_message.num_signers as usize]
            )
            .yellow()
        );
        println!(
            "  Mutable Non-Signers: {}",
            format!(
                "{:?}",
                &account_keys[transaction_message.num_signers as usize
                    ..(transaction_message.num_signers
                        + transaction_message.num_writable_non_signers)
                        as usize]
            )
            .green()
        );
        println!(
            "  Read-Only Non-Signers: {}",
            format!(
                "{:?}",
                &account_keys[(transaction_message.num_signers
                    + transaction_message.num_writable_non_signers)
                    as usize..]
            )
            .blue()
        );

        let transaction_message_instructions: Vec<CompiledInstruction> = transaction_message
            .instructions
            .iter()
            .map(convert_to_compiled_instruction)
            .collect();
        for (i, instruction) in transaction_message_instructions.iter().enumerate() {
            let parsed_instruction = parse(
                &account_keys[instruction.program_id_index as usize],
                instruction,
                &AccountKeys::new(&account_keys, None),
                None,
            );

            println!("✅ Instruction #{}", i + 1);

            match parsed_instruction {
                Ok(result) => {
                    println!("{}", "✅ Proposed Instruction:".green().bold());
                    println!(
                        "  {} {}",
                        "Program:".bright_blue().bold(),
                        result.program.bright_green()
                    );
                    println!(
                        "  {} {}",
                        "Program ID:".bright_blue().bold(),
                        result.program_id.bright_cyan()
                    );

                    // Pretty-print and colorize the JSON
                    if let Value::Object(parsed_data) = &result.parsed {
                        println!("  {} {}", "Parsed Data:".bright_blue().bold(), "{");
                        for (key, value) in parsed_data {
                            match key.as_str() {
                                "type" => {
                                    println!(
                                        "    {}: {}",
                                        key.bright_yellow().bold(),
                                        value.as_str().unwrap_or("Unknown").bright_green()
                                    );
                                }
                                "info" => {
                                    if let Value::Object(info) = value {
                                        println!("    {}: {}", key.bright_yellow().bold(), "{");
                                        for (info_key, info_value) in info {
                                            println!(
                                                "      {}: {}",
                                                info_key.bright_magenta().bold(),
                                                info_value.to_string().bright_cyan()
                                            );
                                        }
                                        println!("    {} ", "}");
                                    }
                                }
                                _ => {
                                    println!(
                                        "    {}: {}",
                                        key.bright_yellow().bold(),
                                        value.to_string().bright_cyan()
                                    );
                                }
                            }
                        }
                        println!("  {} ", "}");
                    } else {
                        println!(
                            "  {} {}",
                            "Parsed Data:".bright_blue().bold(),
                            result.parsed.to_string().yellow()
                        );
                    }

                    // Stack Height
                    let stack_height = match result.stack_height {
                        Some(height) => height.to_string().bright_magenta(),
                        None => "N/A".to_string().yellow(),
                    };
                    println!(
                        "  {} {}",
                        "Stack Height:".bright_blue().bold(),
                        stack_height
                    );
                }
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        "❌ Failed to parse instruction:".red().bold(),
                        e.to_string().yellow()
                    );
                }
            }
        }
        Ok(())
    }
}
fn convert_to_compiled_instruction(
    multisig_instruction: &MultisigCompiledInstruction,
) -> CompiledInstruction {
    CompiledInstruction {
        program_id_index: multisig_instruction.program_id_index,
        accounts: multisig_instruction.account_indexes.clone(),
        data: multisig_instruction.data.clone(),
    }
}
use solana_sdk::address_lookup_table::AddressLookupTableAccount;
use solana_sdk::hash::Hash;
use solana_sdk::message::MessageHeader;
use solana_sdk::message::v0::Message;
use squads_multisig::state::VaultTransactionMessage;
fn into_v0_message(message: VaultTransactionMessage) -> Message {
    let transaction_message_instructions: Vec<CompiledInstruction> = message
        .instructions
        .iter()
        .map(convert_to_compiled_instruction)
        .collect();
    let transaction_message_address_table_lookups: Vec<MessageAddressTableLookup> = message
        .address_table_lookups
        .iter()
        .map(convert_to_msgaddresstablelookup)
        .collect();
    Message {
        header: MessageHeader {
            num_required_signatures: message.num_signers,
            num_readonly_signed_accounts: message
                .num_signers
                .saturating_sub(message.num_writable_signers),
            num_readonly_unsigned_accounts: message.num_writable_non_signers as u8,
        },
        account_keys: message.account_keys,
        recent_blockhash: Hash::default(), // Placeholder; update if needed
        instructions: transaction_message_instructions,
        address_table_lookups: transaction_message_address_table_lookups,
    }
}
fn convert_to_msgaddresstablelookup(
    multisig_message_address_table_lookup: &MultisigMessageAddressTableLookup,
) -> MessageAddressTableLookup {
    MessageAddressTableLookup {
        account_key: multisig_message_address_table_lookup.account_key,
        writable_indexes: multisig_message_address_table_lookup
            .writable_indexes
            .clone(),
        readonly_indexes: multisig_message_address_table_lookup
            .readonly_indexes
            .clone(),
    }
}
fn extract_loaded_addresses(
    address_lookup_table_accounts: &[AddressLookupTableAccount],
    message: &VaultTransactionMessage,
) -> LoadedAddresses {
    let mut writable = Vec::new();
    let mut readonly = Vec::new();

    for lookup in &message.address_table_lookups {
        let lookup_table_account = address_lookup_table_accounts
            .iter()
            .find(|account| account.key == lookup.account_key)
            .unwrap();

        for &index in &lookup.writable_indexes {
            if let Some(pubkey) = lookup_table_account.addresses.get(index as usize) {
                writable.push(*pubkey);
            }
        }

        for &index in &lookup.readonly_indexes {
            if let Some(pubkey) = lookup_table_account.addresses.get(index as usize) {
                readonly.push(*pubkey);
            }
        }
    }

    LoadedAddresses { writable, readonly }
}
use solana_sdk::instruction::AccountMeta;
pub async fn message_to_execute_account_metas(
    rpc_client: &RpcClient,
    message: VaultTransactionMessage,
    ephemeral_signer_bumps: Vec<u8>,
    vault_pda: &Pubkey,
    transaction_pda: &Pubkey,
    program_id: Option<&Pubkey>,
) -> (Vec<AccountMeta>, Vec<AddressLookupTableAccount>) {
    let mut account_metas = Vec::with_capacity(message.account_keys.len());

    let mut address_lookup_table_accounts: Vec<AddressLookupTableAccount> = Vec::new();

    let ephemeral_signer_pdas: Vec<Pubkey> = (0..ephemeral_signer_bumps.len())
        .map(|additional_signer_index| {
            let (pda, _bump_seed) = get_ephemeral_signer_pda(
                &transaction_pda,
                additional_signer_index as u8,
                program_id,
            );
            pda
        })
        .collect();

    let address_lookup_table_keys = message
        .address_table_lookups
        .iter()
        .map(|lookup| lookup.account_key)
        .collect::<Vec<_>>();

    for key in address_lookup_table_keys {
        let account_data = rpc_client.get_account(&key).await.unwrap().data;
        let lookup_table =
            solana_address_lookup_table_program::state::AddressLookupTable::deserialize(
                &account_data,
            )
            .unwrap();

        let address_lookup_table_account = AddressLookupTableAccount {
            addresses: lookup_table.addresses.to_vec(),
            key,
        };

        address_lookup_table_accounts.push(address_lookup_table_account);
        account_metas.push(AccountMeta::new(key, false));
    }

    for (account_index, account_key) in message.account_keys.iter().enumerate() {
        let is_writable =
            VaultTransactionMessage::is_static_writable_index(&message, account_index);
        let is_signer = VaultTransactionMessage::is_signer_index(&message, account_index)
            && !account_key.eq(&vault_pda)
            && !ephemeral_signer_pdas.contains(account_key);

        account_metas.push(AccountMeta {
            pubkey: *account_key,
            is_signer,
            is_writable,
        });
    }

    for lookup in &message.address_table_lookups {
        let lookup_table_account = address_lookup_table_accounts
            .iter()
            .find(|account| account.key == lookup.account_key)
            .unwrap();

        for &account_index in &lookup.writable_indexes {
            let account_index_usize = account_index as usize;

            let pubkey = lookup_table_account
                .addresses
                .get(account_index_usize)
                .unwrap();

            account_metas.push(AccountMeta::new(*pubkey, false));
        }

        for &account_index in &lookup.readonly_indexes {
            let account_index_usize = account_index as usize;

            let pubkey = lookup_table_account
                .addresses
                .get(account_index_usize)
                .unwrap();

            account_metas.push(AccountMeta::new_readonly(*pubkey, false));
        }
    }

    (account_metas, address_lookup_table_accounts)
}
