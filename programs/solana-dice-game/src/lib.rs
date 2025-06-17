use anchor_lang::prelude::*;
use anchor_lang::solana_program::{system_instruction, system_program, sysvar::clock};

declare_id!("BkiKwAoQEogZtundzzn5buhLfntYdoiFABur66buDXMM"); // Replace after deployment

#[program]
pub mod solana_dice_game {
    use super::*;

    #[event]
    pub struct BetResult {
        pub player: Pubkey,
        pub bet_amount: u64,
        pub chosen_number: u8,
        pub random_number: u8,
        pub win_amount: u64,
        pub won: bool,
        pub timestamp: i64,
    }

    #[derive(Accounts)]
    pub struct InitializeGame<'info> {
        #[account(init, payer = admin, space = 8 + GameConfig::LEN)]
        pub game_config: Account<'info, GameConfig>,
        #[account(mut)]
        pub admin: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    pub fn initialize_game(ctx: Context<InitializeGame>, reward_percentage: u64) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = *ctx.accounts.admin.key;
        game_config.reward_percentage = reward_percentage;
        game_config.is_paused = false;
        game_config.min_bet = 1000000; // 0.01 SOL
        game_config.max_bet = 100000000; // 1 SOL
        Ok(())
    }

    #[derive(Accounts)]
    pub struct PlaceBet<'info> {
        #[account(mut)]
        pub game_config: Account<'info, GameConfig>,
        #[account(mut)]
        pub player: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    pub fn place_bet(ctx: Context<PlaceBet>, chosen_number: u8, bet_amount: u64) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;

        // Validate game state
        require!(!game_config.is_paused, ErrorCode::GamePaused);
        require!(
            bet_amount >= game_config.min_bet && bet_amount <= game_config.max_bet,
            ErrorCode::InvalidBetAmount
        );
        require!(
            chosen_number >= 1 && chosen_number <= 6,
            ErrorCode::InvalidNumber
        );

        // Generate random number using clock (for demo)
        let clock = Clock::get()?;
        let random_number = ((clock.unix_timestamp % 6) + 1) as u8;

        // Calculate potential win
        let potential_win = bet_amount
            .checked_mul(game_config.reward_percentage)
            .and_then(|v| v.checked_div(100))
            .ok_or(ErrorCode::MathOverflow)?;

        // Transfer SOL from player to program
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.game_config.key(),
            bet_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.game_config.to_account_info(),
            ],
        )?;

        // Check if player won
        if chosen_number == random_number {
            // Player won - transfer winnings
            **ctx
                .accounts
                .game_config
                .to_account_info()
                .try_borrow_mut_lamports()? -= potential_win;
            **ctx
                .accounts
                .player
                .to_account_info()
                .try_borrow_mut_lamports()? += potential_win;

            emit!(BetResult {
                player: *ctx.accounts.player.key,
                bet_amount,
                chosen_number,
                random_number,
                win_amount: potential_win,
                won: true,
                timestamp: clock.unix_timestamp,
            });
        } else {
            emit!(BetResult {
                player: *ctx.accounts.player.key,
                bet_amount,
                chosen_number,
                random_number,
                win_amount: 0,
                won: false,
                timestamp: clock.unix_timestamp,
            });
        }

        Ok(())
    }

    #[derive(Accounts)]
    pub struct UpdateConfig<'info> {
        #[account(mut, has_one = admin @ ErrorCode::Unauthorized)]
        pub game_config: Account<'info, GameConfig>,
        pub admin: Signer<'info>,
    }

    pub fn update_reward_percentage(ctx: Context<UpdateConfig>, new_percentage: u64) -> Result<()> {
        ctx.accounts.game_config.reward_percentage = new_percentage;
        Ok(())
    }

    pub fn pause_game(ctx: Context<UpdateConfig>) -> Result<()> {
        ctx.accounts.game_config.is_paused = true;
        Ok(())
    }

    pub fn unpause_game(ctx: Context<UpdateConfig>) -> Result<()> {
        ctx.accounts.game_config.is_paused = false;
        Ok(())
    }

    pub fn withdraw_funds(ctx: Context<UpdateConfig>, amount: u64) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        let game_config_lamports = game_config.to_account_info().lamports();
        require!(amount <= game_config_lamports, ErrorCode::InsufficientFunds);

        **game_config.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx
            .accounts
            .admin
            .to_account_info()
            .try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

#[account]
pub struct GameConfig {
    pub admin: Pubkey,
    pub reward_percentage: u64,
    pub is_paused: bool,
    pub min_bet: u64,
    pub max_bet: u64,
}

impl GameConfig {
    pub const LEN: usize = 32 + 8 + 1 + 8 + 8;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Game is currently paused")]
    GamePaused,
    #[msg("Invalid bet amount")]
    InvalidBetAmount,
    #[msg("Chosen number must be between 1 and 6")]
    InvalidNumber,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}
