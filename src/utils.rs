use crate::squads_decoder::{ParseableInstruction, map_instruction};
use clap_v3::ArgMatches;
use colored::Colorize;
use eyre::eyre;
use serde_json::{Map, Value};
use solana_clap_v3_utils::keypair::signer_from_path;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signer::Signer, transaction::VersionedTransaction};
use squads_multisig::solana_client::nonblocking::rpc_client::RpcClient;
use squads_multisig::solana_client::{
    client_error::ClientErrorKind,
    rpc_request::{RpcError, RpcResponseErrorData},
    rpc_response::RpcSimulateTransactionResult,
};
use std::str::FromStr;

pub fn extract_transaction_message(json_str: &str) -> Result<Vec<u8>, String> {
    // Parse the JSON string into a serde_json::Value
    let parsed_json: Value = serde_json::from_str(json_str).map_err(|e| e.to_string())?;

    // Access the "args" object
    let args = parsed_json
        .get("args")
        .and_then(Value::as_object)
        .ok_or_else(|| "Missing 'args' field in JSON".to_string())?;

    // Access the "transactionMessage" array
    let transaction_message = args
        .get("transactionMessage")
        .and_then(Value::as_array)
        .ok_or_else(|| "Missing or invalid 'transactionMessage' field in JSON".to_string())?;

    // Convert the array of integers to Vec<u8>
    transaction_message
        .iter()
        .map(|v| {
            v.as_u64()
                .ok_or_else(|| "Non-integer value in 'transactionMessage'".to_string())
                .and_then(|n| u8::try_from(n).map_err(|_| "Value out of u8 range".to_string()))
        })
        .collect()
}

pub fn create_signer_from_path(
    keypair_path: String,
) -> Result<Box<dyn Signer>, Box<dyn std::error::Error>> {
    let mut wallet_manager = None;
    let matches = ArgMatches::default();

    signer_from_path(&matches, &keypair_path, "Keypair", &mut wallet_manager)
}

pub async fn send_and_confirm_transaction(
    transaction: &VersionedTransaction,
    rpc_client: &RpcClient,
) -> eyre::Result<String> {
    // Try to send and confirm the transaction
    match rpc_client.send_and_confirm_transaction(transaction).await {
        Ok(signature) => {
            println!(
                "Transaction confirmed: {}\n\n",
                signature.to_string().green()
            );
            Ok(signature.to_string())
        }
        Err(err) => {
            if let ClientErrorKind::RpcError(RpcError::RpcResponseError {
                data:
                    RpcResponseErrorData::SendTransactionPreflightFailure(
                        RpcSimulateTransactionResult {
                            logs: Some(logs), ..
                        },
                    ),
                ..
            }) = &err.kind
            {
                println!("Simulation logs:\n\n{}\n", logs.join("\n").yellow());
            }

            Err(eyre!("Transaction failed: {}", err.to_string().red()))
        }
    }
}
pub fn transaction_details(transaction: &VersionedTransaction) -> eyre::Result<Value> {
    let squads_program_id = Pubkey::from_str("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf")?;
    let message = &transaction.message;
    let mut result = Value::Null;

    // Classify and print account roles with security context
    let accounts: Vec<String> = message
        .static_account_keys()
        .iter()
        .map(|key| key.to_string())
        .collect();

    let header = message.header();
    let (mutable_signers, readonly_signers, mutable_unsigned, readonly_unsigned) =
        classify_accounts(
            accounts,
            header.num_required_signatures as usize,
            header.num_readonly_signed_accounts as usize,
            header.num_readonly_unsigned_accounts as usize,
        )?;

    println!("🔐 SECURITY-CRITICAL ACCOUNT ROLES:");
    println!("  🛡️  Mutable Signers (Can modify state AND sign):");
    for account in &mutable_signers {
        println!("    - {}", account.red());
    }
    println!("  🔒 Read-Only Signers (Can view but not modify state):");
    for account in &readonly_signers {
        println!("    - {}", account.yellow());
    }
    println!("  ⚠️  Mutable Unsigned (Can modify state but don't sign):");
    for account in &mutable_unsigned {
        println!("    - {}", account.bright_red());
    }
    println!("  👀 Read-Only Unsigned (Can view state but don't sign):");
    for account in &readonly_unsigned {
        println!("    - {}", account.green());
    }

    // Load IDL for Squads program
    let idl_path = std::path::Path::new("./idl.json");
    if !idl_path.exists() {
        return Err(eyre::eyre!(
            "SECURITY WARNING: Missing IDL file at {:?}",
            idl_path
        ));
    }
    let idl = solana_idl::try_extract_classic_idl(std::fs::read_to_string(idl_path)?.as_str())?;

    // Process all instructions targeting Squads program
    println!("\n🔍 INSPECTING SQUADS INSTRUCTIONS:");
    for (ix_index, ix) in message.instructions().iter().enumerate() {
        let program_pubkey = message.static_account_keys()[ix.program_id_index as usize];

        if program_pubkey != squads_program_id {
            continue; // Skip non-Squads instructions
        }

        println!("\n🛡️ SQUADS INSTRUCTION #{}", ix_index + 1);
        println!("  Program ID: {}", program_pubkey.to_string().blue());

        // Extract accounts with security context

        let account_pubkeys: Vec<Pubkey> = ix
            .accounts
            .iter()
            .map(|&index| &message.static_account_keys()[index as usize])
            .cloned()
            .collect();
        let instruction = MyInstruction {
            program_id_key: squads_program_id,
            account_keys: account_pubkeys.clone(),
            instruction_data: ix.data.clone(),
        };

        // Map and print instruction details with security focus
        match map_instruction(
            &instruction,
            Some(&idl),
            &chainparser::borsh::BorshDeserializer,
        ) {
            Ok(ix_map_result) => {
                println!(
                    "  📛 Instruction: {}",
                    ix_map_result
                        .instruction_name
                        .unwrap_or("UNKNOWN".to_string())
                        .red()
                        .bold()
                );

                println!("  🔑 Accounts Involved:");
                for (pubkey, label) in ix_map_result.accounts {
                    let role = if mutable_signers.contains(&pubkey.to_string()) {
                        "MUTABLE SIGNER".red()
                    } else if readonly_signers.contains(&pubkey.to_string()) {
                        "READONLY SIGNER".yellow()
                    } else if mutable_unsigned.contains(&pubkey.to_string()) {
                        "MUTABLE UNSIGNED".bright_red()
                    } else {
                        "READONLY UNSIGNED".green()
                    };
                    println!("    - {}: {} ({})", pubkey, label, role);
                }

                //  let redacted_args = redact_sensitive_data(&ix_map_result.decoded_args);
                println!("  🔓 Decoded Arguments:");
                println!(
                    "{}",
                    serde_json::to_string_pretty(&ix_map_result.decoded_args)
                        .unwrap_or_else(|_| "    [SECURITY WARNING: Failed to decode]".to_string())
                        .bright_black()
                );

                // Store first Squads instruction result
                if result.is_null() {
                    result = ix_map_result.decoded_args;
                }
            }
            Err(e) => {
                eprintln!(
                    "❌ SECURITY ALERT: Failed to decode Squads instruction #{}: {}",
                    ix_index + 1,
                    e
                );
            }
        }
    }

    Ok(result)
}

fn classify_accounts(
    accounts: Vec<String>,
    num_required_signatures: usize,
    num_readonly_signed_accounts: usize,
    num_readonly_unsigned_accounts: usize,
) -> eyre::Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)> {
    if accounts.len() < num_required_signatures {
        return Err(eyre::eyre!(
            "SECURITY ERROR: Not enough accounts to classify - possible malformed transaction"
        ));
    }

    let mutable_signers =
        accounts[..num_required_signatures - num_readonly_signed_accounts].to_vec();
    let readonly_signers = accounts
        [num_required_signatures - num_readonly_signed_accounts..num_required_signatures]
        .to_vec();
    let mutable_unsigned =
        accounts[num_required_signatures..accounts.len() - num_readonly_unsigned_accounts].to_vec();
    let readonly_unsigned = accounts[accounts.len() - num_readonly_unsigned_accounts..].to_vec();

    Ok((
        mutable_signers,
        readonly_signers,
        mutable_unsigned,
        readonly_unsigned,
    ))
}
pub fn redact_sensitive_data(decoded_args: &Value) -> Value {
    match decoded_args {
        // If it's an object, iterate over its key-value pairs
        Value::Object(map) => {
            let mut redacted_map = Map::new();
            for (key, value) in map {
                if key == "configAuthority" {
                    // Redact sensitive field
                    redacted_map.insert(key.clone(), Value::String("[REDACTED]".to_string()));
                } else {
                    // Recursively process nested values
                    redacted_map.insert(key.clone(), redact_sensitive_data(value));
                }
            }
            Value::Object(redacted_map)
        }
        // If it's an array, recursively process each element
        Value::Array(vec) => Value::Array(vec.iter().map(|v| redact_sensitive_data(v)).collect()),
        // For other types (e.g., strings, numbers, booleans), return as-is
        _ => decoded_args.clone(),
    }
}

pub struct MyInstruction {
    pub program_id_key: Pubkey,
    pub account_keys: Vec<Pubkey>,
    pub instruction_data: Vec<u8>,
}

impl ParseableInstruction for MyInstruction {
    fn program_id(&self) -> &Pubkey {
        &self.program_id_key
    }

    fn accounts(&self) -> Vec<Pubkey> {
        self.account_keys.clone()
    }

    fn data(&self) -> &[u8] {
        &self.instruction_data
    }
}
