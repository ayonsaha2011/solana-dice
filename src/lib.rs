use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("Dice111111111111111111111111111111111111111");

#[program]
pub mod dice {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, reward_percent: u8) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.admin = ctx.accounts.admin.key();
        state.reward_percent = reward_percent;
        state.is_paused = false;
        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet>, guess: u8, amount: u64) -> Result<()> {
        require!(guess >= 1 && guess <= 6, DiceError::InvalidGuess);
        let state = &mut ctx.accounts.state;
        require!(!state.is_paused, DiceError::ContractPaused);
        require!(amount > 0, DiceError::InvalidBetAmount);

        // Transfer bet lamports from user to vault
        transfer(CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ), amount)?;

        let random_number = random(ctx.accounts.user.key(), Clock::get()?.unix_timestamp);

        if random_number == guess {
            let reward = amount
                .checked_mul(state.reward_percent as u64)
                .ok_or(DiceError::MathOverflow)?
                / 100;
            **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= reward;
            **ctx.accounts.user.to_account_info().try_borrow_mut_lamports()? += reward;
        }
        Ok(())
    }

    pub fn update_reward(ctx: Context<AdminOnly>, reward_percent: u8) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.reward_percent = reward_percent;
        Ok(())
    }

    pub fn set_pause(ctx: Context<AdminOnly>, paused: bool) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.is_paused = paused;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

fn random(key: Pubkey, timestamp: i64) -> u8 {
    use solana_program::hash::hashv;
    let hash = hashv(&[key.as_ref(), &timestamp.to_le_bytes()]);
    (hash.to_bytes()[0] % 6) + 1
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + State::SIZE, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    #[account(init, payer = admin, seeds = [b"vault"], bump, space = 8)]
    /// CHECK: vault account to hold SOL
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut, seeds = [b"state"], bump)]
    pub state: Account<'info, State>,
    #[account(mut, seeds = [b"vault"], bump)]
    /// CHECK: vault account to hold SOL
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminOnly<'info> {
    #[account(mut, seeds = [b"state"], bump, has_one = admin)]
    pub state: Account<'info, State>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(seeds = [b"state"], bump, has_one = admin)]
    pub state: Account<'info, State>,
    #[account(mut, seeds = [b"vault"], bump)]
    /// CHECK: vault account to hold SOL
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[account]
pub struct State {
    pub admin: Pubkey,
    pub reward_percent: u8,
    pub is_paused: bool,
}

impl State {
    const SIZE: usize = 32 + 1 + 1;
}

#[error_code]
pub enum DiceError {
    #[msg("Invalid guess")] InvalidGuess,
    #[msg("Contract is paused")] ContractPaused,
    #[msg("Invalid bet amount")] InvalidBetAmount,
    #[msg("Math overflow")] MathOverflow,
}

