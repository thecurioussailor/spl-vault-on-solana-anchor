# 🪙 SPL Token Vault — Anchor Program

**Vault #2 of the Solana Vault Series**

Deposit, withdraw, and close a vault holding any SPL token — USDC, custom tokens, anything with a mint. Each user gets one vault per token mint, enforced by PDA seeds.

---

## What's New vs Vault #1 (SOL Vault)

| | SOL Vault | SPL Token Vault |
|---|---|---|
| **Stores** | Raw lamports | SPL tokens |
| **Vault account type** | System account | Token account |
| **Transfer CPIs** | `system_program::transfer` | `token::transfer` |
| **Token account authority** | — | `vault_state` PDA |
| **Seeds include** | `owner` | `owner` + `mint` |
| **Closing** | Close state account | Close token account + state account |

---

## What This Teaches

| Concept | Where |
|---|---|
| Multi-file Anchor program structure | `src/instructions/`, `state.rs`, `error.rs`, `events.rs` |
| `impl`-on-context pattern | Each instruction file |
| Mint accounts | `initialize.rs` |
| PDA-owned token accounts | `initialize.rs` |
| `token::transfer` CPI (deposit) | `deposit.rs` |
| PDA signing for token transfers | `withdraw.rs`, `close.rs` |
| `token::close_account` CPI | `close.rs` |
| `MintMismatch` guard via `constraint` | `deposit.rs`, `withdraw.rs`, `close.rs` |
| `has_one` authorization | `Withdraw` + `Close` contexts |
| Anchor events | `events.rs` |
| Custom errors | `error.rs` |
| LiteSVM testing | `tests/vault_tests.rs` |

---

## Program Structure

```
programs/spl-vault/src/
├── lib.rs                      ← declare_id! + #[program] router only
├── state.rs                    ← VaultState account struct
├── error.rs                    ← VaultError enum
├── events.rs                   ← on-chain events
└── instructions/
    ├── mod.rs
    ├── initialize.rs           ← Initialize context + impl handler
    ├── deposit.rs              ← Deposit context + impl handler
    ├── withdraw.rs             ← Withdraw context + impl handler
    └── close.rs                ← Close context + impl handler
```

Each instruction file owns both its `#[derive(Accounts)]` struct and its `impl` handler — they always change together.

---

## Architecture

```
owner wallet
    │
    ├── vault_state PDA   [seeds: b"vault_state", owner, mint]
    │       owner:      Pubkey   ← who controls this vault
    │       mint:       Pubkey   ← which token this vault holds
    │       bump:       u8       ← vault_state PDA bump
    │       vault_bump: u8       ← vault_token_account PDA bump
    │
    └── vault_token_account PDA   [seeds: b"vault", owner, mint]
            TokenAccount
            authority = vault_state   ← vault_state PDA controls it
            holds the actual tokens
```

**Why two PDAs?**
A token-holding account must be a `TokenAccount` owned by the Token Program. A state account must be owned by your program. They cannot be the same account — so we use one for each purpose.

**Why is `vault_state` the token account authority (not the token account itself)?**
A PDA can only sign if your program knows its seeds. `vault_state` uses `seeds = [b"vault_state", owner, mint]` — your program reconstructs these to sign CPIs. The vault token account's seeds include a different prefix (`b"vault"`), and since `vault_state` is already on hand in every instruction context, it's the natural signer.

**Why include `mint` in both PDAs' seeds?**
Without it, Alice would have only one vault regardless of which token she's holding. Including the mint means:
- Alice's USDC vault ≠ Alice's BONK vault
- Alice's USDC vault ≠ Bob's USDC vault

---

## Instructions

### `initialize`
Creates `vault_state` and `vault_token_account`. The token account's authority is set to `vault_state`, making it program-controlled. Bumps for both PDAs are stored at init time.

### `deposit(amount: u64)`
Transfers tokens from `owner_token_account → vault_token_account` via Token Program CPI. The owner signs directly. Requires `amount > 0` and a matching mint.

### `withdraw(amount: u64)`
Transfers tokens from `vault_token_account → owner_token_account`. Since `vault_state` is the token account authority, it signs via `CpiContext::new_with_signer` using its own seeds. Requires sufficient vault balance and owner authorization via `has_one`.

### `close`
1. If any tokens remain, transfers them back to the owner (same PDA-signing pattern as withdraw)
2. Calls `token::close_account` on `vault_token_account` — reclaims its rent lamports to the owner
3. Anchor's `close = owner` constraint on `vault_state` closes the state account and reclaims its rent

After this, both PDAs are gone.

---

## Key Patterns

**`vault_state` as token account authority**
```rust
#[account(
    init,
    token::mint = mint,
    token::authority = vault_state,   // vault_state PDA controls the token account
    seeds = [b"vault", owner.key().as_ref(), mint.key().as_ref()],
    bump
)]
pub vault_token_account: Account<'info, TokenAccount>,
```

**PDA signing for token transfer (withdraw / close)**
```rust
let seeds = &[
    b"vault_state",
    owner_key.as_ref(),
    mint_key.as_ref(),
    &[self.vault_state.bump],
];

token::transfer(
    CpiContext::new_with_signer(
        self.token_program.key(),
        Transfer {
            from: self.vault_token_account.to_account_info(),
            to:   self.owner_token_account.to_account_info(),
            authority: self.vault_state.to_account_info(),  // the PDA that signs
        },
        &[&seeds[..]],
    ),
    amount,
)?;
```

**MintMismatch guard**
```rust
#[account(
    mut,
    constraint = owner_token_account.mint == vault_state.mint @ VaultError::MintMismatch,
    constraint = owner_token_account.owner == owner.key() @ VaultError::Unauthorized,
)]
pub owner_token_account: Account<'info, TokenAccount>,
```
Prevents passing in a token account for a different mint than the vault was initialized with.

**`has_one` for authorization**
```rust
#[account(
    has_one = owner @ VaultError::Unauthorized,
)]
pub vault_state: Account<'info, VaultState>,
```
Anchor automatically checks `vault_state.owner == signer.key()`.

**Closing a token account before state**
```rust
token::close_account(CpiContext::new_with_signer(
    self.token_program.key(),
    CloseAccount {
        account:     self.vault_token_account.to_account_info(),
        destination: self.owner.to_account_info(),
        authority:   self.vault_state.to_account_info(),
    },
    &[&seeds[..]],
))?;
// vault_state is closed by Anchor's `close = owner` constraint after the handler returns
```

---

## Testing

Tests use **anchor-litesvm** — in-process Solana runtime, no validator needed, runs in milliseconds.

```
running 9 tests
test test_initialize                  ... ok
test test_initialize_twice_fails      ... ok
test test_deposit                     ... ok
test test_deposit_zero_fails          ... ok
test test_withdraw                    ... ok
test test_withdraw_overdraft_fails    ... ok
test test_withdraw_unauthorized_fails ... ok
test test_close_empty_vault           ... ok
test test_close_drains_tokens_to_owner... ok

test result: ok. 9 passed; 0 failed
```

### Run tests
```bash
cd programs/spl-vault
cargo test
```

---

## Running Locally

```bash
# 1. Install JS dependencies
yarn install

# 2. Build the program
anchor build

# 3. Run LiteSVM tests (no validator needed)
cd programs/spl-vault && cargo test

# 4. Deploy to devnet
solana config set --url devnet
solana airdrop 2
anchor program deploy --provider.cluster devnet
```

---

## Toolchain

| Tool | Version |
|---|---|
| Anchor | 1.0.1 |
| anchor-lang | 1.0.1 |
| anchor-spl | 1.0.1 |
| anchor-litesvm | 0.4.0 |
| Solana CLI | 3.1.13 |
| Rust | 1.89.0 |

---

## Devnet Deployment

**Program ID:** `CPq94NSjtQtEm2rKVoGCkMLCusmeYq9j4EZqbuhsjupQ`

Verify on explorer:
`https://explorer.solana.com/address/CPq94NSjtQtEm2rKVoGCkMLCusmeYq9j4EZqbuhsjupQ?cluster=devnet`

Fetch IDL:
```bash
anchor idl fetch CPq94NSjtQtEm2rKVoGCkMLCusmeYq9j4EZqbuhsjupQ --provider.cluster devnet
```
