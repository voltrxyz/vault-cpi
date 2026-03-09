use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    account_info::AccountInfo,
    instruction::{ AccountMeta, Instruction },
    program::invoke,
};

#[error_code]
pub enum ErrorCodes {
    #[msg("CPI_TO_VOLTR_VAULT_PROGRAM_FAILED")]
    CpiToVoltrVaultFailed,
}

/// Anchor discriminator for the `cancel_request_withdraw_vault` instruction.
/// `sha256("global:cancel_request_withdraw_vault")[0..8]`
fn get_cancel_request_withdraw_vault_discriminator() -> [u8; 8] {
    [231, 54, 14, 6, 223, 124, 127, 238]
}

pub struct CancelRequestWithdrawVaultParams<'info> {
    pub user_transfer_authority: AccountInfo<'info>,
    pub protocol: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_lp_mint: AccountInfo<'info>,
    pub user_lp_ata: AccountInfo<'info>,
    pub request_withdraw_lp_ata: AccountInfo<'info>,
    pub request_withdraw_vault_receipt: AccountInfo<'info>,
    pub lp_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    // Target Voltr Vault program
    pub voltr_vault_program: AccountInfo<'info>,
}

impl<'info> CancelRequestWithdrawVaultParams<'info> {
    pub fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![
            self.user_transfer_authority.clone(),
            self.protocol.clone(),
            self.vault.clone(),
            self.vault_lp_mint.clone(),
            self.user_lp_ata.clone(),
            self.request_withdraw_lp_ata.clone(),
            self.request_withdraw_vault_receipt.clone(),
            self.lp_token_program.clone(),
            self.system_program.clone()
        ]
    }

    pub fn cancel_request_withdraw_vault(&self) -> Result<()> {
        let instruction_data = get_cancel_request_withdraw_vault_discriminator().to_vec();

        let account_metas = vec![
            AccountMeta::new(*self.user_transfer_authority.key, true),
            AccountMeta::new_readonly(*self.protocol.key, false),
            AccountMeta::new(*self.vault.key, false),
            AccountMeta::new(*self.vault_lp_mint.key, false),
            AccountMeta::new(*self.user_lp_ata.key, false),
            AccountMeta::new(*self.request_withdraw_lp_ata.key, false),
            AccountMeta::new(*self.request_withdraw_vault_receipt.key, false),
            AccountMeta::new_readonly(*self.lp_token_program.key, false),
            AccountMeta::new_readonly(*self.system_program.key, false)
        ];

        let instruction = Instruction {
            program_id: *self.voltr_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        invoke(&instruction, &self.to_account_infos()).map_err(|_|
            ErrorCodes::CpiToVoltrVaultFailed.into()
        )
    }
}
