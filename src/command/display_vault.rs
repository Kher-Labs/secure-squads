use solana_sdk::pubkey::Pubkey;
use squads_multisig::pda::get_vault_pda;
use squads_multisig::solana_client::nonblocking::rpc_client::RpcClient;
use std::str::FromStr;

use clap::Args;

#[derive(Args)]
pub struct DisplayVault {
    /// Multisig Program ID
    #[arg(long)]
    program_id: Option<String>,

    /// Path to the Program Config Initializer Keypair
    #[arg(long)]
    multisig_address: String,

    // index to derive the vault, default 0
    #[arg(long)]
    vault_index: Option<u8>,
}

impl DisplayVault {
    // vault genrated for (seeds, multisig_address, vault_index)
    pub async fn execute(self) -> eyre::Result<()> {
        let Self {
            program_id,
            multisig_address,
            vault_index,
        } = self;

        let program_id =
            program_id.unwrap_or_else(|| "SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf".to_string());

        let program_id = Pubkey::from_str(&program_id).expect("Invalid program ID");

        let multisig_address =
            Pubkey::from_str(&multisig_address).expect("Invalid multisig address");

        let vault_index = vault_index.unwrap_or(0);

        let vault_address = get_vault_pda(&multisig_address, vault_index, Some(&program_id));

        println!("Vault: {:?}", vault_address.0);
        // Initialize RPC client
        let rpc_url = "https://api.devnet.solana.com "; // Replace with your desired cluster
        let rpc_client = RpcClient::new(rpc_url.to_string());

        match rpc_client.get_balance(&vault_address.0).await {
            Ok(balance) => println!("Vault SOL Balance: {} lamports", balance),
            Err(_) => println!("Vault does not exist or has no SOL balance."),
        }

        Ok(())
    }
}
