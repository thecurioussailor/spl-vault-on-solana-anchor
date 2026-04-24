use anchor_litesvm::{AnchorContext, AnchorLiteSVM, Pubkey, Signer, TestHelpers, AssertionHelpers};
use anchor_lang::prelude::Pubkey as APubkey;
use anchor_lang::{AccountDeserialize, declare_program};
use anchor_lang;
use solana_native_token::LAMPORTS_PER_SOL;

declare_program!(spl_vault);

const PROGRAM_BYTES: &[u8] = include_bytes!("../../../target/deploy/spl_vault.so");

fn ap(p: Pubkey) -> APubkey { APubkey::from(p.to_bytes()) }
fn sys() -> APubkey { APubkey::default() }
fn token_program() -> APubkey { anchor_spl::token::ID }

fn program_id() -> Pubkey {
    spl_vault::ID.to_bytes().into()
}

fn setup() -> AnchorContext {
    AnchorLiteSVM::build_with_program(program_id(), PROGRAM_BYTES)
}

fn vault_state_pda(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"vault_state", owner.as_ref(), mint.as_ref()],
        &program_id(),
    ).0
}

fn vault_token_pda(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"vault", owner.as_ref(), mint.as_ref()],
        &program_id(),
    ).0
}

fn get_vault_state(ctx: &AnchorContext, key: &Pubkey) -> spl_vault::accounts::VaultState {
    let acct = ctx.svm.get_account(key).expect("vault_state not found");
    spl_vault::accounts::VaultState::try_deserialize(&mut acct.data.as_ref())
        .expect("deserialize failed")
}

// ── initialize ────────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let mut ctx  = setup();
    let owner    = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp  = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint     = mint_kp.pubkey();
    let vs       = vault_state_pda(&owner.pubkey(), &mint);
    let vault    = vault_token_pda(&owner.pubkey(), &mint);

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner:               ap(owner.pubkey()),
                mint:                ap(mint),
                vault_state:         ap(vs),
                vault_token_account: ap(vault),
                token_program:       token_program(),
                system_program:      sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    let state = get_vault_state(&ctx, &vs);
    assert_eq!(state.owner.to_bytes(), owner.pubkey().to_bytes());
    assert_eq!(state.mint.to_bytes(), mint.to_bytes());
    assert!(state.bump > 0);
    assert!(state.vault_bump > 0);
}

#[test]
fn test_initialize_twice_fails() {
    let mut ctx = setup();
    let owner   = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint    = mint_kp.pubkey();
    let vs      = vault_state_pda(&owner.pubkey(), &mint);
    let vault   = vault_token_pda(&owner.pubkey(), &mint);

    let make_ix = |ctx: &AnchorContext| {
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner:               ap(owner.pubkey()),
                mint:                ap(mint),
                vault_state:         ap(vs),
                vault_token_account: ap(vault),
                token_program:       token_program(),
                system_program:      sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap()
    };

    ctx.execute_instruction(make_ix(&ctx), &[&owner]).unwrap().assert_success();
    ctx.execute_instruction(make_ix(&ctx), &[&owner]).unwrap().assert_failure();
}

// ── deposit ───────────────────────────────────────────────────────────────────

#[test]
fn test_deposit() {
    let mut ctx   = setup();
    let owner     = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp   = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint      = mint_kp.pubkey();
    let owner_ata = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let vs        = vault_state_pda(&owner.pubkey(), &mint);
    let vault     = vault_token_pda(&owner.pubkey(), &mint);

    ctx.svm.mint_to(&mint, &owner_ata, &owner, 1_000_000).unwrap();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner:               ap(owner.pubkey()),
                mint:                ap(mint),
                vault_state:         ap(vs),
                vault_token_account: ap(vault),
                token_program:       token_program(),
                system_program:      sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Deposit {
                owner:               ap(owner.pubkey()),
                vault_state:         ap(vs),
                owner_token_account: ap(owner_ata),
                vault_token_account: ap(vault),
                token_program:       token_program(),
            })
            .args(spl_vault::client::args::Deposit { amount: 500_000 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.svm.assert_token_balance(&vault, 500_000);
}

#[test]
fn test_deposit_zero_fails() {
    let mut ctx   = setup();
    let owner     = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp   = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint      = mint_kp.pubkey();
    let owner_ata = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let vs        = vault_state_pda(&owner.pubkey(), &mint);
    let vault     = vault_token_pda(&owner.pubkey(), &mint);

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner:               ap(owner.pubkey()),
                mint:                ap(mint),
                vault_state:         ap(vs),
                vault_token_account: ap(vault),
                token_program:       token_program(),
                system_program:      sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Deposit {
                owner:               ap(owner.pubkey()),
                vault_state:         ap(vs),
                owner_token_account: ap(owner_ata),
                vault_token_account: ap(vault),
                token_program:       token_program(),
            })
            .args(spl_vault::client::args::Deposit { amount: 0 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_failure();
}

// ── withdraw ──────────────────────────────────────────────────────────────────

#[test]
fn test_withdraw() {
    let mut ctx   = setup();
    let owner     = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp   = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint      = mint_kp.pubkey();
    let owner_ata = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let vs        = vault_state_pda(&owner.pubkey(), &mint);
    let vault     = vault_token_pda(&owner.pubkey(), &mint);

    ctx.svm.mint_to(&mint, &owner_ata, &owner, 1_000_000).unwrap();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner: ap(owner.pubkey()), mint: ap(mint),
                vault_state: ap(vs), vault_token_account: ap(vault),
                token_program: token_program(), system_program: sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Deposit {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Deposit { amount: 1_000_000 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Withdraw {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Withdraw { amount: 400_000 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.svm.assert_token_balance(&vault, 600_000);
    ctx.svm.assert_token_balance(&owner_ata, 400_000);
}

#[test]
fn test_withdraw_overdraft_fails() {
    let mut ctx   = setup();
    let owner     = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp   = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint      = mint_kp.pubkey();
    let owner_ata = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let vs        = vault_state_pda(&owner.pubkey(), &mint);
    let vault     = vault_token_pda(&owner.pubkey(), &mint);

    ctx.svm.mint_to(&mint, &owner_ata, &owner, 500_000).unwrap();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner: ap(owner.pubkey()), mint: ap(mint),
                vault_state: ap(vs), vault_token_account: ap(vault),
                token_program: token_program(), system_program: sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Deposit {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Deposit { amount: 500_000 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Withdraw {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Withdraw { amount: 999_999_999 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_failure();
}

#[test]
fn test_withdraw_unauthorized_fails() {
    let mut ctx      = setup();
    let owner        = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let attacker     = ctx.create_funded_account(5 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp      = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint         = mint_kp.pubkey();
    let owner_ata    = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let attacker_ata = ctx.svm.create_associated_token_account(&mint, &attacker).unwrap();
    let vs           = vault_state_pda(&owner.pubkey(), &mint);
    let vault        = vault_token_pda(&owner.pubkey(), &mint);

    ctx.svm.mint_to(&mint, &owner_ata, &owner, 1_000_000).unwrap();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner: ap(owner.pubkey()), mint: ap(mint),
                vault_state: ap(vs), vault_token_account: ap(vault),
                token_program: token_program(), system_program: sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Deposit {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Deposit { amount: 1_000_000 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Withdraw {
                owner: ap(attacker.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(attacker_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Withdraw { amount: 1_000_000 })
            .instruction().unwrap(),
        &[&attacker],
    ).unwrap().assert_failure();
}

// ── close ─────────────────────────────────────────────────────────────────────

#[test]
fn test_close_empty_vault() {
    let mut ctx   = setup();
    let owner     = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp   = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint      = mint_kp.pubkey();
    let owner_ata = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let vs        = vault_state_pda(&owner.pubkey(), &mint);
    let vault     = vault_token_pda(&owner.pubkey(), &mint);

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner: ap(owner.pubkey()), mint: ap(mint),
                vault_state: ap(vs), vault_token_account: ap(vault),
                token_program: token_program(), system_program: sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Close {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Close {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    assert!(ctx.svm.get_account(&vault).is_none());
    assert!(ctx.svm.get_account(&vs).is_none());
}

#[test]
fn test_close_drains_tokens_to_owner() {
    let mut ctx   = setup();
    let owner     = ctx.create_funded_account(10 * LAMPORTS_PER_SOL).unwrap();
    let mint_kp   = ctx.svm.create_token_mint(&owner, 6).unwrap();
    let mint      = mint_kp.pubkey();
    let owner_ata = ctx.svm.create_associated_token_account(&mint, &owner).unwrap();
    let vs        = vault_state_pda(&owner.pubkey(), &mint);
    let vault     = vault_token_pda(&owner.pubkey(), &mint);

    ctx.svm.mint_to(&mint, &owner_ata, &owner, 1_000_000).unwrap();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Initialize {
                owner: ap(owner.pubkey()), mint: ap(mint),
                vault_state: ap(vs), vault_token_account: ap(vault),
                token_program: token_program(), system_program: sys(),
            })
            .args(spl_vault::client::args::Initialize {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Deposit {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Deposit { amount: 1_000_000 })
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.execute_instruction(
        ctx.program()
            .accounts(spl_vault::client::accounts::Close {
                owner: ap(owner.pubkey()), vault_state: ap(vs),
                owner_token_account: ap(owner_ata), vault_token_account: ap(vault),
                token_program: token_program(),
            })
            .args(spl_vault::client::args::Close {})
            .instruction().unwrap(),
        &[&owner],
    ).unwrap().assert_success();

    ctx.svm.assert_token_balance(&owner_ata, 1_000_000);
    assert!(ctx.svm.get_account(&vault).is_none());
    assert!(ctx.svm.get_account(&vs).is_none());
}
