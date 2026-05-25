use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface, burn, transfer_checked, Burn, TransferChecked},
};

use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint_x: Box<InterfaceAccount<'info, Mint>>,

    pub mint_y: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user,
    )]
    pub user_x: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user,
    )]
    pub user_y: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    pub user_lp: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount: u64, // Amount of LP tokens that the user wants to "burn"
        min_x: u64,  // Minimum amount of token X that the user wants to receive
        min_y: u64,  // Minimum amount of token Y that the user wants to receive
    ) -> Result<()> {
        require!(!self.config.locked, AmmError::PoolLocked);
        require_neq!(amount, 0, AmmError::InvalidAmount);

        let (x, y) =
            if self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
                (min_x, min_y)
            } else {
                let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6,
                )
                .map_err(|_| AmmError::InvalidAmount)?;
                (amounts.x, amounts.y)
            };

        require!(x >= min_x && y >= min_y, AmmError::SlippageExceeded);

        self.burn_lp_tokens(amount)?;
        self.withdraw_tokens(true, x)?;
        self.withdraw_tokens(false, y)
    }

    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint) = match is_x {
            true => (self.vault_x.to_account_info(), self.user_x.to_account_info(), &self.mint_x),
            false => (self.vault_y.to_account_info(), self.user_y.to_account_info(), &self.mint_y),
        };

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                TransferChecked {
                    from,
                    to,
                    authority: self.config.to_account_info(),
                    mint: mint.to_account_info(),
                },
                &[&[
                    b"config",
                    &self.config.seed.to_le_bytes(),
                    &[self.config.config_bump],
                ]],
            ),
            amount,
            mint.decimals,
        )
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        burn(
            CpiContext::new(
                self.token_program.key(),
                Burn {
                    mint: self.mint_lp.to_account_info(),
                    from: self.user_lp.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
        )
    }
}