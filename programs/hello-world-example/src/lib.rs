use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token, TokenAccount, Transfer, transfer}, associated_token::AssociatedToken};

const ANCHOR_MINT_ADDRESS: &str = "hC2RopBzFGBs1L6WNX3a3FPTpARrnSSGkaiv1HhK3Gk";

declare_id!("9zFgMcx43Aq6RdzA6vGZYKjdpoxQijD7VaYXf456W6vs");

#[program]
pub mod hello_world_example {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.pool.authority = ctx.accounts.authority.key();
        ctx.accounts.pool.user_count = 0u32;
        ctx.accounts.pool.total_staked = 0u64;
        ctx.accounts.pool.bump = ctx.bumps.pool;
        Ok(())
    }

    pub fn create_user(ctx: Context<CreateUser>) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.stake = 0u64;
        user.bump = ctx.bumps.user;

        ctx.accounts.pool.user_count =
         ctx.accounts.pool.user_count + 1;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_anchor_ata.to_account_info(),
                authority: ctx.accounts.user_anchor_ata_authority.to_account_info(),
                to: ctx.accounts.program_anchor_ata.to_account_info(),
            }
        );
        transfer(cpi_ctx, amount)?;

        ctx.accounts.user.stake += amount;
        ctx.accounts.pool.total_staked += amount;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let bump = ctx.accounts.pool.bump;
        let pool_author = ctx.accounts.pool.authority.as_ref();
        let signer: &[&[&[u8]]] = &[&[b"pool", pool_author, &[bump]]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_anchor_ata.to_account_info(),
                to: ctx.accounts.user_anchor_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer
        );
        transfer(cpi_ctx, amount)?;

        ctx.accounts.user.stake -= amount;
        ctx.accounts.pool.total_staked -= amount;

        Ok(())
    }
}

pub const POOL_STORAGE_TOTAL_BYTES: usize = 32 + 4 + 8 + 1;
#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub user_count: u32,
    pub total_staked: u64,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + POOL_STORAGE_TOTAL_BYTES,
        seeds = [b"pool", authority.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = authority,
        associated_token::mint=anchor_mint,
        associated_token::authority=pool,
    )]
    vault: Account<'info, TokenAccount>,
    #[account(
        address = ANCHOR_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub anchor_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    // SPL Token Program
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

pub const USER_STORAGE_TOTAL_BYTES: usize = 1 + 8;
#[account]
pub struct User {
    bump: u8,
    stake: u64,
}

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(
        init, 
        payer = authority,
        space = 8 + USER_STORAGE_TOTAL_BYTES,
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    // Used to update total staked amount
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    // Used to update staked amount by user
    #[account(
        mut,
        seeds = [b"user", user_anchor_ata_authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    // Require for the deriving associated token accounts
    #[account(
        address = ANCHOR_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub anchor_mint: Account<'info, Mint>,
    // Associated Token Account for User which holds $ANCHOR.
    #[account(mut)]
    pub user_anchor_ata: Account<'info, TokenAccount>,
    // The authority allowed to mutate user anchor's associated token account
    pub user_anchor_ata_authority: Signer<'info>,
    // Used to receive $ANCHOR from users
    #[account(mut)]
    pub program_anchor_ata: Account<'info, TokenAccount>,
    // SPL Token Program
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    // Used to update total staked amount
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    // Used to update staked amount by user
    #[account(
        mut,
        seeds = [b"user", user_anchor_ata_authority.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,
    // Require for the deriving associated token accounts
    #[account(
        address = ANCHOR_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub anchor_mint: Account<'info, Mint>,
    // Associated Token Account for User which holds $ANCHOR.
    #[account(mut)]
    pub user_anchor_ata: Account<'info, TokenAccount>,
    // The authority allowed to mutate user anchor's associated token account
    pub user_anchor_ata_authority: Signer<'info>,
    // Used to receive $ANCHOR from users
    #[account(mut)]
    pub program_anchor_ata: Account<'info, TokenAccount>,
    // SPL Token Program
    pub token_program: Program<'info, Token>,
}
