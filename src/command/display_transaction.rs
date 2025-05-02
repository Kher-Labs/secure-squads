use clap::Args;
use colored::Colorize;
use serde_json::Value;
use serde_json::to_string_pretty;
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::message::{AccountKeys, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::parse_instruction::parse;
use squads_multisig::anchor_lang::AnchorDeserialize;
use squads_multisig::anchor_lang::{AccountDeserialize, InstructionData};
use squads_multisig::squads_multisig_program::{MessageAddressTableLookup, TransactionMessage};
use squads_multisig::state::{MultisigCompiledInstruction, MultisigMessageAddressTableLookup};
use std::io::Cursor;
//use squads_multisig::anchor_lang::{AccountDeserialize, InstructionData};
use squads_multisig::pda::get_transaction_pda;
use squads_multisig::solana_client::nonblocking::rpc_client::RpcClient;
use squads_multisig::squads_multisig_program::state::VaultTransaction;
use std::str::FromStr;

#[derive(Args)]
pub struct DisplayTransaction {
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
        let rpc_url = "https://api.devnet.solana.com "; // Replace with your desired cluster
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

//use squads_multisig::squads_multisig_program::utils::SmallVec;
/*
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
*/
