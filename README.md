## Overview

This documentation covers Cross-Program Invocation (CPI) integration for Voltr Vault, a yield-bearing vault protocol. Users can deposit assets into a vault to receive LP tokens, which represent their share of the vault's total assets. Withdrawing assets is a two-step process designed to ensure vault stability and manage liquidity.

### Deployed Addresses

| Network | Program Address                               | Link                                                                                           |
| ------- | --------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Mainnet | `vVoLTRjQmtFpiYoegx285Ze4gsLJ8ZxgFKVcuvmG1a8` | [voltr-vault](https://explorer.solana.com/address/vVoLTRjQmtFpiYoegx285Ze4gsLJ8ZxgFKVcuvmG1a8) |

## Core CPI Functions

### 1. Deposit Flow

- `deposit_vault` - Deposit assets into the vault and receive LP tokens.

### 2. Withdrawal Flow (Two-Step)

- `request_withdraw_vault` - Initiate a withdrawal request, locking LP tokens into a receipt.
- `withdraw_vault` - Finalize the withdrawal after the waiting period has passed, burning the locked LP tokens to receive the underlying assets.

---

## 1. Deposit Vault CPI Integration

This function allows a user to deposit an underlying asset into the vault and mint a corresponding amount of LP tokens.

### Function Discriminator

```rust
fn get_deposit_vault_discriminator() -> [u8; 8] {
    // discriminator = sha256("global:deposit_vault")[0..8]
    [41, 158, 82, 88, 95, 140, 106, 154]
}
```

### `deposit_vault` CPI Struct

```rust
use anchor_lang::prelude::*;
use anchor_spl::{token, token_interface};

pub struct DepositVaultParams<'info> {
    pub user_transfer_authority: AccountInfo<'info>,
    pub protocol: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_asset_mint: AccountInfo<'info>,
    pub vault_lp_mint: AccountInfo<'info>,
    pub user_asset_ata: AccountInfo<'info>,
    pub vault_asset_idle_ata: AccountInfo<'info>,
    pub vault_asset_idle_auth: AccountInfo<'info>,
    pub user_lp_ata: AccountInfo<'info>,
    pub vault_lp_mint_auth: AccountInfo<'info>,
    pub asset_token_program: AccountInfo<'info>,
    pub lp_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    // Target Voltr Vault program
    pub voltr_vault_program: AccountInfo<'info>,
}
```

### Implementation

The CPI call requires passing all accounts specified in the instruction. The `user_transfer_authority` must sign the transaction. The vault's internal PDAs (`vault_asset_idle_auth`, `vault_lp_mint_auth`) will be signed for by the Voltr Vault program itself during the CPI.

```rust
impl<'info> DepositVaultParams<'info> {
    pub fn deposit_vault(&self, amount: u64) -> Result<()> {
        let mut instruction_data = get_deposit_vault_discriminator().to_vec();
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let account_metas = vec![
            AccountMeta::new_readonly(*self.user_transfer_authority.key, true),
            AccountMeta::new_readonly(*self.protocol.key, false),
            AccountMeta::new(*self.vault.key, false),
            AccountMeta::new_readonly(*self.vault_asset_mint.key, false),
            AccountMeta::new(*self.vault_lp_mint.key, false),
            AccountMeta::new(*self.user_asset_ata.key, false),
            AccountMeta::new(*self.vault_asset_idle_ata.key, false),
            AccountMeta::new_readonly(*self.vault_asset_idle_auth.key, false),
            AccountMeta::new(*self.user_lp_ata.key, false),
            AccountMeta::new_readonly(*self.vault_lp_mint_auth.key, false),
            AccountMeta::new_readonly(*self.asset_token_program.key, false),
            AccountMeta::new_readonly(*self.lp_token_program.key, false),
            AccountMeta::new_readonly(*self.system_program.key, false),
        ];

        let instruction = Instruction {
            program_id: *self.voltr_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        invoke(
            &instruction,
            &[
                self.user_transfer_authority.clone(),
                self.protocol.clone(),
                self.vault.clone(),
                self.vault_asset_mint.clone(),
                self.vault_lp_mint.clone(),
                self.user_asset_ata.clone(),
                self.vault_asset_idle_ata.clone(),
                self.vault_asset_idle_auth.clone(),
                self.user_lp_ata.clone(),
                self.vault_lp_mint_auth.clone(),
                self.asset_token_program.clone(),
                self.lp_token_program.clone(),
                self.system_program.clone(),
            ],
        )?;
        Ok(())
    }
}
```

> Full snippet available [here](./references/deposit_vault.rs)

### `deposit_vault` Account Explanations

| Account                   | Mutability | Signer | Purpose                                             |
| ------------------------- | ---------- | ------ | --------------------------------------------------- |
| `user_transfer_authority` | Immutable  | Yes    | The user depositing assets.                         |
| `protocol`                | Immutable  | No     | The global Voltr protocol state account.            |
| `vault`                   | Mutable    | No     | The target vault state account.                     |
| `vault_asset_mint`        | Immutable  | No     | The mint of the asset being deposited.              |
| `vault_lp_mint`           | Mutable    | No     | The LP mint for the vault, representing shares.     |
| `user_asset_ata`          | Mutable    | No     | The user's ATA for the asset (source).              |
| `vault_asset_idle_ata`    | Mutable    | No     | The vault's idle ATA for the asset (destination).   |
| `vault_asset_idle_auth`   | Immutable  | No     | The PDA authority over the `vault_asset_idle_ata`.  |
| `user_lp_ata`             | Mutable    | No     | The user's ATA for the LP token (destination).      |
| `vault_lp_mint_auth`      | Immutable  | No     | The PDA authority for minting LP tokens.            |
| `asset_token_program`     | Immutable  | No     | The Token Program or Token-2022 Program for assets. |
| `lp_token_program`        | Immutable  | No     | The Token Program for LP tokens.                    |
| `system_program`          | Immutable  | No     | The Solana System Program.                          |

---

## 2. Withdrawal Flow CPI Integration

### Step 1: `request_withdraw_vault`

This function initiates a withdrawal. The user specifies an amount, and their LP tokens are transferred to an escrow receipt account.

#### Function Discriminator

```rust
fn get_request_withdraw_vault_discriminator() -> [u8; 8] {
    // discriminator = sha256("global:request_withdraw_vault")[0..8]
    [147, 67, 155, 26, 32, 163, 32, 193]
}
```

#### `request_withdraw_vault` CPI Struct

```rust
pub struct RequestWithdrawVaultParams<'info> {
    pub payer: AccountInfo<'info>,
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
```

#### Implementation

```rust
impl<'info> RequestWithdrawVaultParams<'info> {
    pub fn request_withdraw_vault(
        &self,
        amount: u64,
        is_amount_in_lp: bool,
        is_withdraw_all: bool
    ) -> Result<()> {
        let mut instruction_data = get_request_withdraw_vault_discriminator().to_vec();
        instruction_data.extend_from_slice(&amount.to_le_bytes());
        instruction_data.push(is_amount_in_lp as u8);
        instruction_data.push(is_withdraw_all as u8);

        let account_metas = vec![
            AccountMeta::new(*self.payer.key, true),
            AccountMeta::new_readonly(*self.user_transfer_authority.key, true),
            AccountMeta::new_readonly(*self.protocol.key, false),
            AccountMeta::new_readonly(*self.vault.key, false),
            AccountMeta::new_readonly(*self.vault_lp_mint.key, false),
            AccountMeta::new(*self.user_lp_ata.key, false),
            AccountMeta::new(*self.request_withdraw_lp_ata.key, false),
            AccountMeta::new(*self.request_withdraw_vault_receipt.key, false),
            AccountMeta::new_readonly(*self.lp_token_program.key, false),
            AccountMeta::new_readonly(*self.system_program.key, false),
        ];

        let instruction = Instruction {
            program_id: *self.voltr_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        invoke(&instruction, &self.to_account_infos())?;
        Ok(())
    }
}
```

> Full snippet available [here](./references/request_withdraw_vault.rs)

#### `request_withdraw_vault` Account Explanations

| Account                          | Mutability | Signer | Purpose                                                         |
| -------------------------------- | ---------- | ------ | --------------------------------------------------------------- |
| `payer`                          | Mutable    | Yes    | The account paying for the receipt's rent.                      |
| `user_transfer_authority`        | Immutable  | Yes    | The user requesting the withdrawal.                             |
| `protocol`                       | Immutable  | No     | The global Voltr protocol state account.                        |
| `vault`                          | Immutable  | No     | The vault from which to withdraw.                               |
| `vault_lp_mint`                  | Immutable  | No     | The LP mint of the vault.                                       |
| `user_lp_ata`                    | Mutable    | No     | The user's LP token account (source).                           |
| `request_withdraw_lp_ata`        | Mutable    | No     | The receipt's ATA to hold escrowed LP tokens.                   |
| `request_withdraw_vault_receipt` | Mutable    | No     | The PDA receipt account to be created, storing request details. |
| `lp_token_program`               | Immutable  | No     | The Token Program for LP tokens.                                |
| `system_program`                 | Immutable  | No     | The Solana System Program.                                      |

---

### Step 2: `withdraw_vault`

After the `withdrawal_waiting_period` defined in the vault has passed, this function can be called to complete the withdrawal.

#### Function Discriminator

```rust
fn get_withdraw_vault_discriminator() -> [u8; 8] {
    // discriminator = sha256("global:withdraw_vault")[0..8]
    [81, 229, 229, 94, 86, 233, 198, 15]
}
```

#### `withdraw_vault` CPI Struct

```rust
pub struct WithdrawVaultParams<'info> {
    pub user_transfer_authority: AccountInfo<'info>,
    pub protocol: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    pub vault_asset_mint: AccountInfo<'info>,
    pub vault_lp_mint: AccountInfo<'info>,
    pub request_withdraw_lp_ata: AccountInfo<'info>,
    pub vault_asset_idle_ata: AccountInfo<'info>,
    pub vault_asset_idle_auth: AccountInfo<'info>,
    pub user_asset_ata: AccountInfo<'info>,
    pub request_withdraw_vault_receipt: AccountInfo<'info>,
    pub asset_token_program: AccountInfo<'info>,
    pub lp_token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    // Target Voltr Vault program
    pub voltr_vault_program: AccountInfo<'info>,
}
```

#### Implementation

```rust
impl<'info> WithdrawVaultParams<'info> {
    pub fn withdraw_vault(&self) -> Result<()> {
        let instruction_data = get_withdraw_vault_discriminator().to_vec();

        let account_metas = vec![
            AccountMeta::new(*self.user_transfer_authority.key, true),
            AccountMeta::new_readonly(*self.protocol.key, false),
            AccountMeta::new(*self.vault.key, false),
            AccountMeta::new_readonly(*self.vault_asset_mint.key, false),
            AccountMeta::new(*self.vault_lp_mint.key, false),
            AccountMeta::new(*self.request_withdraw_lp_ata.key, false),
            AccountMeta::new(*self.vault_asset_idle_ata.key, false),
            AccountMeta::new(*self.vault_asset_idle_auth.key, false),
            AccountMeta::new(*self.user_asset_ata.key, false),
            AccountMeta::new(*self.request_withdraw_vault_receipt.key, false),
            AccountMeta::new_readonly(*self.asset_token_program.key, false),
            AccountMeta::new_readonly(*self.lp_token_program.key, false),
            AccountMeta::new_readonly(*self.system_program.key, false),
        ];

        let instruction = Instruction {
            program_id: *self.voltr_vault_program.key,
            accounts: account_metas,
            data: instruction_data,
        };

        invoke(&instruction, &self.to_account_infos())?;
        Ok(())
    }
}
```

> Full snippet available [here](./references/withdraw_vault.rs)

#### `withdraw_vault` Account Explanations

| Account                          | Mutability | Signer | Purpose                                                             |
| -------------------------------- | ---------- | ------ | ------------------------------------------------------------------- |
| `user_transfer_authority`        | Mutable    | Yes    | The user finalizing the withdrawal.                                 |
| `protocol`                       | Immutable  | No     | The global Voltr protocol state account.                            |
| `vault`                          | Mutable    | No     | The vault state account.                                            |
| `vault_asset_mint`               | Immutable  | No     | The mint of the asset being withdrawn.                              |
| `vault_lp_mint`                  | Mutable    | No     | The vault's LP mint.                                                |
| `request_withdraw_lp_ata`        | Mutable    | No     | The receipt's ATA holding the escrowed LP tokens (source for burn). |
| `vault_asset_idle_ata`           | Mutable    | No     | The vault's idle ATA for the asset (source for transfer).           |
| `vault_asset_idle_auth`          | Mutable    | No     | The PDA authority over the `vault_asset_idle_ata`.                  |
| `user_asset_ata`                 | Mutable    | No     | The user's ATA for the asset (destination).                         |
| `request_withdraw_vault_receipt` | Mutable    | No     | The PDA receipt account, which will be closed after the withdrawal. |
| `asset_token_program`            | Immutable  | No     | The Token Program or Token-2022 Program for assets.                 |
| `lp_token_program`               | Immutable  | No     | The Token Program for LP tokens.                                    |
| `system_program`                 | Immutable  | No     | The Solana System Program.                                          |

---

## Key Implementation Notes

### 1. Account Derivation

Most accounts are standard PDAs derived from seeds defined in the Voltr program. For CPI, you will need to derive these PDAs on your client or in your program to pass them into the instructions. Key PDAs include:

- `protocol`: `["protocol"]`
- `vault_asset_idle_auth`: `["vault_asset_idle_auth", vault_key]`
- `vault_lp_mint_auth`: `["vault_lp_mint_auth", vault_key]`
- `request_withdraw_vault_receipt`: `["request_withdraw_vault_receipt", vault_key, user_key]`

### 2. Signers

- User-initiated actions require the `user_transfer_authority` to be a signer.
- The CPI-calling program does **not** need to provide seeds for the Voltr Vault's internal PDAs. The Voltr program will use `invoke_signed` internally for its own CPIs (e.g., token transfers and burns). Your program simply passes the PDA addresses in the `AccountInfo` slice.

### 3. Error Handling

Your program should be prepared to handle errors from the Voltr Vault program, such as:

- `InvalidAmount`: Input amount is zero or invalid.
- `MaxCapExceeded`: Deposit would exceed the vault's maximum capacity.
- `WithdrawalNotYetAvailable`: Attempting to `withdraw_vault` before the waiting period has passed.
- `OperationNotAllowed`: The protocol has globally disabled the attempted operation.
