#![allow(deprecated)]
// #region code
use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

declare_id!("CwrqeMj2U8tFr1Rhkgwc84tpAsqbt9pTt2a4taoTADPr");

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
