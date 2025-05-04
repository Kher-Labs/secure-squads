use crate::command::display_transaction::DisplayTransaction;
use crate::command::display_vault::DisplayVault;
use crate::command::initiate_program_upgrade::InitiateProgramUpgrade;
use crate::command::initiate_transfer::InitiateTransfer;
use crate::command::multisig_create::MultisigCreate;
use crate::command::proposal_vote::ProposalVote;
use crate::command::vault_transaction_accounts_close::VaultTransactionAccountsClose;
use crate::command::vault_transaction_execute::VaultTransactionExecute;

use clap::Subcommand;

pub mod display_transaction;
pub mod display_vault;
pub mod initiate_program_upgrade;
pub mod initiate_transfer;
pub mod multisig_create;
pub mod proposal_vote;
pub mod vault_transaction_accounts_close;
pub mod vault_transaction_execute;

#[derive(Subcommand)]
pub enum Command {
    MultisigCreate(MultisigCreate),
    DisplayVault(DisplayVault),
    InitiateTransfer(InitiateTransfer),
    ProposalVote(ProposalVote),
    VaultTransactionExecute(VaultTransactionExecute),
    VaultTransactionAccountsClose(VaultTransactionAccountsClose),
    DisplayTransaction(DisplayTransaction),
    InitiateProgramUpgrade(InitiateProgramUpgrade),
}
