use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

use anchor_lang::prelude::*;

#[account]
pub struct CounterState {
    pub authority: Pubkey,
    pub count: u64,
}

#[program]
pub mod counter {
    use super::*;

    pub fn initialize(
        ctx: Context<initializeContext>,
    ) -> Result<()> {
        ctx.accounts.state.authority = ctx.accounts.authority.key;
        ctx.accounts.state.count = 0;
        Ok(())
    }

    pub fn increment(
        ctx: Context<incrementContext>,
    ) -> Result<()> {
        require!((ctx.accounts.state.authority == ctx.accounts.authority.key));
        ctx.accounts.state.count = (ctx.accounts.state.count + 1);
        Ok(())
    }

    pub fn decrement(
        ctx: Context<decrementContext>,
    ) -> Result<()> {
        require!((ctx.accounts.state.authority == ctx.accounts.authority.key));
        require!((ctx.accounts.state.count > 0));
        ctx.accounts.state.count = (ctx.accounts.state.count - 1);
        Ok(())
    }

}

#[derive(Accounts)]
pub struct initializeContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 48
    )]
    pub state: Account<'info, CounterState>,
}

#[derive(Accounts)]
pub struct incrementContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub state: Account<'info, CounterState>,
}

#[derive(Accounts)]
pub struct decrementContext<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub state: Account<'info, CounterState>,
}

