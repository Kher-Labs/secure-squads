use clap::Args;
use colored::Colorize;
use dialoguer::Confirm;
use indicatif::ProgressBar;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::CompiledInstruction as CompiledInstruction_x;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message;
use solana_sdk::message::{AccountKeys, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::parse_token::parse_token;
use spl_associated_token_account::instruction::create_associated_token_account;
use squads_multisig::anchor_lang::AnchorDeserialize;
use std::io::Cursor;
use std::str::FromStr;
use std::time::Duration;

use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::instruction::transfer;
use squads_multisig::anchor_lang::{AnchorSerialize, InstructionData};
use squads_multisig::client::get_multisig;
use squads_multisig::pda::{get_proposal_pda, get_transaction_pda, get_vault_pda};
use squads_multisig::solana_client::nonblocking::rpc_client::RpcClient;
use squads_multisig::squads_multisig_program::ProposalCreateArgs;
use squads_multisig::squads_multisig_program::VaultTransactionCreateArgs;
use squads_multisig::squads_multisig_program::accounts::ProposalCreate as ProposalCreateAccounts;
use squads_multisig::squads_multisig_program::accounts::VaultTransactionCreate as VaultTransactionCreateAccounts;
use squads_multisig::squads_multisig_program::anchor_lang::ToAccountMetas;
use squads_multisig::squads_multisig_program::instruction::ProposalCreate as ProposalCreateData;
use squads_multisig::squads_multisig_program::instruction::VaultTransactionCreate as VaultTransactionCreateData;
use squads_multisig::squads_multisig_program::{
    CompiledInstruction, MessageAddressTableLookup, TransactionMessage,
};
use squads_multisig::vault_transaction::VaultTransactionMessageExt;

use crate::utils::{
    create_signer_from_path, extract_transaction_message, send_and_confirm_transaction,
    transaction_details,
};

#[derive(Args)]
pub struct InitiateTransfer {
    /// RPC URL
    #[arg(long)]
    rpc_url: Option<String>,

    /// Multisig Program ID
    #[arg(long)]
    program_id: Option<String>,

    /// Token program ID. Defaults to regular SPL.
    #[arg(long)]
    token_program_id: Option<String>,
    //So11111111111111111111111111111111111111112
    /// Token Mint Address.
    #[arg(long)]
    token_mint_address: String,

    #[arg(long)]
    token_amount_u64: u64,

    /// The recipient of the Token(s)
    #[arg(long)]
    recipient: String,

    /// Path to the Program Config Initializer Keypair
    #[arg(long)]
    keypair: String,

    /// The multisig where the transaction has been proposed
    #[arg(long)]
    multisig_pubkey: String,

    #[arg(long)]
    vault_index: u8,

    /// Memo to be included in the transaction
    #[arg(long)]
    memo: Option<String>,

    #[arg(long)]
    priority_fee_lamports: Option<u64>,
}

impl InitiateTransfer {
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            rpc_url,
            program_id,
            token_program_id,
            keypair,
            multisig_pubkey,
            memo,
            vault_index,
            priority_fee_lamports,
            token_amount_u64,
            token_mint_address,
            recipient,
        } = self;

        let program_id =
            program_id.unwrap_or_else(|| "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf".to_string());

        let token_program_id = token_program_id
            .unwrap_or_else(|| "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string());

        let program_id = Pubkey::from_str(&program_id).expect("Invalid program ID");
        let token_program_id: Pubkey =
            Pubkey::from_str(&token_program_id).expect("Invalid program ID");

        let transaction_creator_keypair = create_signer_from_path(keypair).unwrap();

        let transaction_creator = transaction_creator_keypair.pubkey();

        let rpc_url = rpc_url.unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());
        let rpc_url_clone = rpc_url.clone();
        let rpc_client = &RpcClient::new(rpc_url);

        let multisig = Pubkey::from_str(&multisig_pubkey).expect("Invalid multisig address");

        let recipient_pubkey = Pubkey::from_str(&recipient).expect("Invalid recipient address");

        let token_mint = Pubkey::from_str(&token_mint_address).expect("Invalid Token Mint Address");

        let multisig_data = get_multisig(rpc_client, &multisig).await?;

        let transaction_index = multisig_data.transaction_index + 1;

        let transaction_pda = get_transaction_pda(&multisig, transaction_index, Some(&program_id));
        let proposal_pda = get_proposal_pda(&multisig, transaction_index, Some(&program_id));
        println!();
        println!(
            "{}",
            "👀 You're about to create a vault transaction, please review the details:".yellow()
        );
        println!();
        println!("RPC Cluster URL:   {}", rpc_url_clone);
        println!("Program ID:        {}", program_id);
        println!("Your Public Key:       {}", transaction_creator);
        println!();
        println!("⚙️ Config Parameters");
        println!("Multisig Key:       {}", multisig_pubkey);
        println!("Transaction Index:       {}", transaction_index);
        println!("Vault Index:       {}", vault_index);
        println!();

        let proceed = Confirm::new()
            .with_prompt("Do you want to proceed?")
            .default(false)
            .interact()?;
        if !proceed {
            println!("OK, aborting.");
            return Ok(());
        }
        println!();

        let progress = ProgressBar::new_spinner().with_message("Sending transaction...");
        progress.enable_steady_tick(Duration::from_millis(100));

        let blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .expect("Failed to get blockhash");

        let vault_pda = get_vault_pda(&multisig, vault_index, Some(&program_id));
        println!("Vault PDA: {:?}", vault_pda.0);
        let sender_ata = get_associated_token_address_with_program_id(
            &vault_pda.0,
            &token_mint,
            &token_program_id,
        );
        println!("Sender ATA: {:?}", sender_ata);
        // Check if sender ATA exists

        if rpc_client.get_account(&sender_ata).await.is_err() {
            println!("Creating sender ATA...");
            let lamports_to_wrap = 100_000_000;
            let create_sender_ata_ix = create_associated_token_account(
                &transaction_creator,
                &vault_pda.0,
                &token_mint,
                &token_program_id,
            );
            let transfer_sol_ix = solana_sdk::system_instruction::transfer(
                &transaction_creator,
                &sender_ata,
                lamports_to_wrap,
            );

            let sync_native_ix =
                spl_token::instruction::sync_native(&token_program_id, &sender_ata)
                    .expect("Failed to build sync_native instruction");

            let blockhash = rpc_client
                .get_latest_blockhash()
                .await
                .expect("Failed to get blockhash");
            let tx = Transaction::new_signed_with_payer(
                &[create_sender_ata_ix, transfer_sol_ix, sync_native_ix],
                Some(&transaction_creator),
                &[&*transaction_creator_keypair],
                blockhash,
            );

            // Send transaction
            let signature = rpc_client.send_and_confirm_transaction(&tx).await.unwrap();
            println!("Sender ATA created successfully. Signature: {}", signature);
        }

        let recipient_ata = get_associated_token_address_with_program_id(
            &recipient_pubkey,
            &token_mint,
            &token_program_id,
        );
        println!("Recipient ATA: {:?}", recipient_ata);
        if rpc_client.get_account(&recipient_ata).await.is_err() {
            println!("Creating recipient ATA...");
            let create_recipient_ata_ix = create_associated_token_account(
                &transaction_creator,
                &recipient_pubkey,
                &token_mint,
                &token_program_id,
            );
            let blockhash = rpc_client
                .get_latest_blockhash()
                .await
                .expect("Failed to get blockhash");
            let tx = Transaction::new_signed_with_payer(
                &[create_recipient_ata_ix],
                Some(&transaction_creator),
                &[&*transaction_creator_keypair], // avoid boxing
                blockhash,
            );

            // Send transaction
            let signature = rpc_client.send_and_confirm_transaction(&tx).await.unwrap();
            println!(
                "Recipient ATA created successfully. Signature: {}",
                signature
            );
        }
        println!("Token Amount: {:?}", token_amount_u64);
        println!("Authority pubkey: {:?}", &vault_pda.0);

        let transaction_message = TransactionMessage::try_compile(
            &vault_pda.0,
            &[transfer(
                &token_program_id,
                &sender_ata,
                &recipient_ata,
                &vault_pda.0,
                &[&vault_pda.0],
                token_amount_u64,
            )
            .unwrap()],
            &[],
        )
        .unwrap();

        let message = Message::try_compile(
            &transaction_creator,
            &[
                ComputeBudgetInstruction::set_compute_unit_price(
                    priority_fee_lamports.unwrap_or(200_000),
                ),
                Instruction {
                    accounts: VaultTransactionCreateAccounts {
                        creator: transaction_creator,
                        rent_payer: transaction_creator,
                        transaction: transaction_pda.0,
                        multisig,
                        system_program: solana_sdk::system_program::id(),
                    }
                    .to_account_metas(Some(false)),
                    data: VaultTransactionCreateData {
                        args: VaultTransactionCreateArgs {
                            ephemeral_signers: 0,
                            vault_index,
                            memo,
                            transaction_message: transaction_message.try_to_vec().unwrap(),
                        },
                    }
                    .data(),
                    program_id,
                },
                Instruction {
                    accounts: ProposalCreateAccounts {
                        creator: transaction_creator,
                        rent_payer: transaction_creator,
                        proposal: proposal_pda.0,
                        multisig,
                        system_program: solana_sdk::system_program::id(),
                    }
                    .to_account_metas(Some(false)),
                    data: ProposalCreateData {
                        args: ProposalCreateArgs {
                            draft: false,
                            transaction_index,
                        },
                    }
                    .data(),
                    program_id,
                },
            ],
            &[],
            blockhash,
        )
        .unwrap();

        let transaction = VersionedTransaction::try_new(
            VersionedMessage::V0(message),
            &[&*transaction_creator_keypair],
        )
        .expect("Failed to create transaction");
        let result = transaction_details(&transaction.clone());

        match result {
            Ok(json_value) => {
                let json_str = json_value.to_string(); // convert Value to JSON string
                let transaction_message = extract_transaction_message(&json_str);
                match transaction_message {
                    Ok(vec_u8) => {
                        let mut reader = Cursor::new(vec_u8); // Cursor implements `Read`
                        let msg = TransactionMessage::deserialize_reader(&mut reader)?;
                        println!("TransactionMessage:");
                        println!(
                            "  Signers: total={}, writable={}, writable_non_signers={}",
                            msg.num_signers, msg.num_writable_signers, msg.num_writable_non_signers,
                        );

                        // Convert SmallVec to Vec using the From impl
                        let account_keys: Vec<Pubkey> = msg.account_keys.clone().into();
                        let account_keys_x = AccountKeys::new(&account_keys, None);

                        let instructions: Vec<CompiledInstruction> =
                            msg.instructions.clone().into();
                        let compiled_instruction_x = CompiledInstruction_x::new_from_raw_parts(
                            instructions[0].program_id_index,
                            instructions[0].data.clone().into(),
                            instructions[0].account_indexes.clone().into(),
                        );
                        // currenlty the decoder have cuspport for token program only
                        let parsed_token_ix = parse_token(&compiled_instruction_x, &account_keys_x);

                        // now decodde with token program
                        /*
                        println!("  Instructions:");
                        for (i, ix) in instructions.iter().enumerate() {
                            let accounts: Vec<u8> = ix.account_indexes.clone().into();
                            let data: Vec<u8> = ix.data.clone().into();
                            println!("    [{}] Program ID index: {}", i, ix.program_id_index);
                            println!("         Account indexes: {:?}", accounts);
                            println!("         Data : {:#?}", &data);
                        }
                        */

                        let lookups: Vec<MessageAddressTableLookup> =
                            msg.address_table_lookups.clone().into();
                        println!("  Address Table Lookups:");

                        for (i, lookup) in lookups.iter().enumerate() {
                            let writable: Vec<u8> = lookup.writable_indexes.clone().into();
                            let readonly: Vec<u8> = lookup.readonly_indexes.clone().into();
                            println!(
                                "    [{}] Account Key: {}",
                                i,
                                lookup.account_key.to_string()
                            );
                            println!("         Writable Indexes: {:?}", writable);
                            println!("         Readonly Indexes: {:?}", readonly);
                        }
                        match parsed_token_ix {
                            Ok(result) => println!("parse toke ix info {:}", result.info),
                            Err(e) => {
                                eprintln!("Failed to parse token ix: {}", e);
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!("Failed to extract: {}", e);
                    }
                }

                //print trnasfer message
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
        let signature = send_and_confirm_transaction(&transaction, &rpc_client).await?;
        println!(
            "✅ Transaction created successfully. Signature: {}",
            signature.green()
        );
        Ok(())
    }
}
