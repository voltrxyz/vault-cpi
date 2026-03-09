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

/// Anchor discriminator for the `instant_withdraw_vault` instruction.
/// `sha256("global:instant_withdraw_vault")[0..8]`
fn get_instant_withdraw_vault_discriminator() -> [u8; 8] {
    [221, 56, 115, 168, 128, 220, 235, 245]
}

pub struct InstantWithdrawVaultParams<'info> {
    pub user_transfer_authority: AccountInfo<'info>,
    pub protocol: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_asset_mint: AccountInfo<'info>,
    pub vault_lp_mint: AccountInfo<'info>,
    pub user_lp_ata: AccountInfo<'info>,
    pub vault_asset_idle_ata: AccountInfo<'info>,
    pub vault_asset_idle_auth: AccountInfo<'info>,
    pub user_asset_ata: AccountInfo<'info>,
    pub asset_token_program: AccountInfo<'info>,
    pub lp_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    // Target Voltr Vault program
    pub voltr_vault_program: AccountInfo<'info>,
}

impl<'info> InstantWithdrawVaultParams<'info> {
    pub fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![
            self.user_transfer_authority.clone(),
            self.protocol.clone(),
            self.vault.clone(),
            self.vault_asset_mint.clone(),
            self.vault_lp_mint.clone(),
            self.user_lp_ata.clone(),
            self.vault_asset_idle_ata.clone(),
            self.vault_asset_idle_auth.clone(),
            self.user_asset_ata.clone(),
            self.asset_token_program.clone(),
            self.lp_token_program.clone(),
            self.system_program.clone()
        ]
    }

    pub fn instant_withdraw_vault(
        &self,
        amount: u64,
        is_amount_in_lp: bool,
        is_withdraw_all: bool
    ) -> Result<()> {
        let mut instruction_data = get_instant_withdraw_vault_discriminator().to_vec();
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        instruction_data.push(is_amount_in_lp as u8);
        instruction_data.push(is_withdraw_all as u8);

        let account_metas = vec![
            AccountMeta::new_readonly(*self.user_transfer_authority.key, true),
            AccountMeta::new_readonly(*self.protocol.key, false),
            AccountMeta::new(*self.vault.key, false),
            AccountMeta::new_readonly(*self.vault_asset_mint.key, false),
            AccountMeta::new(*self.vault_lp_mint.key, false),
            AccountMeta::new(*self.user_lp_ata.key, false),
            AccountMeta::new(*self.vault_asset_idle_ata.key, false),
            AccountMeta::new(*self.vault_asset_idle_auth.key, false),
            AccountMeta::new(*self.user_asset_ata.key, false),
            AccountMeta::new_readonly(*self.asset_token_program.key, false),
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
