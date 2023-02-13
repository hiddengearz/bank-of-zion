#![allow(deprecated)]
// #region code
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("GS9ftm9H95koKobmomiJeThUY1zPwJpvikQJT6jiXFgB");

#[program]
pub mod basic_4 {
    use super::*;

    #[state]
    pub struct Hello {
        pub name: String,
    }

    impl Hello {

        pub fn hello(&mut self, ctx: Context<Auth>,) -> anchor_lang::Result<()> {
            msg!("hello {}, let's pretend this is the pyth program", ctx.accounts.authority.key());
            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct Auth<'info> {
    authority: Signer<'info>,
}
// #endregion code
