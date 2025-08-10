use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use anchor_spl::token_interface::TokenInterface;
use spl_token;
declare_id!("E8jj31VT5EMpWq8mqJVh8rXGUBCet6r31u41SELzSQb9");

#[program]
pub mod my_project {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {

        msg!("Token program passed in: {}", ctx.accounts.token_program.key());
        msg!("Expected token program: {}", spl_token::id());

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.program_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_context, amount)?;

        ctx.accounts.user_state.balance += amount;
        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            amount,
        });
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(ctx.accounts.user_state.balance >= amount, CustomError::InsufficientFunds);

        let cpi_accounts = Transfer {
            from: ctx.accounts.program_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.program_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let authority_bump = ctx.bumps.program_authority;
        let seeds = &[b"authority".as_ref(), &[authority_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_context, amount)?;

        ctx.accounts.user_state.balance -= amount;
        emit!(WithdrawEvent {
            user: ctx.accounts.user.key(),
            amount,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + TokenState::LEN,
        seeds = [b"token_state", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, TokenState>,
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut, seeds = [b"authority"], bump)]
    pub program_authority: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"token_state", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, TokenState>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct TokenState {
    pub balance: u64,
}

impl TokenState {
    pub const LEN: usize = 8;
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum CustomError {
    #[msg("Insufficient funds in your account.")]
    InsufficientFunds,
}

#[derive(Accounts)]
pub struct Initialize {}
