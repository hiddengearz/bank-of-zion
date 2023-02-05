use solana_program::program_pack::Pack;

use {
    crate::error::ZionError,
    crate::state::{SwapState,Token, AUTHORITY_PREFIX},
    crate::instructions::{ZionInstruction, Initialize, AdminDeposit, Deposit, Withdraw, Swap},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        pubkey::{Pubkey, PUBKEY_BYTES},
        entrypoint::ProgramResult,
        msg,
        program_option::COption,
        program_error::ProgramError,
        program_memory::sol_memcmp,
        sysvar::{rent::Rent, Sysvar},
        system_program,
    },
    crate::cpi::{
        create_pda_account,
        token_burn,
        token_mint_to,
        token_transfer,
        token_transfer_signed,
    },
    pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed, Price}
};



///struct used for processing instructions
pub struct Processor {}
impl Processor {

    /// Unpacks a spl_token `Account`.
    fn unpack_token_account(
        account_info: &AccountInfo,
    ) -> Result<spl_token::state::Account, ZionError> {
        if !cmp_pubkeys(account_info.owner, &spl_token::id()) {
            Err(ZionError::IncorrectTokenProgramId)
        } else {
            spl_token::state::Account::unpack(&account_info.data.borrow())
                .map_err(|_| ZionError::ExpectedTokenAccount)
        }
    }

    /// Unpacks a spl_token `Mint`.
    fn unpack_mint(
        account_info: &AccountInfo,
    ) -> Result<spl_token::state::Mint, ZionError> {
        if !cmp_pubkeys(account_info.owner, &spl_token::id()) {
            Err(ZionError::IncorrectTokenProgramId)
        } else {
            spl_token::state::Mint::unpack(&account_info.data.borrow())
                .map_err(|_| ZionError::ExpectedMint)
        }
    }


    ///Validate the mint and token acounts authorities/owners are correct
    fn validate_mint_and_token_accounts(
        token_account_owners: &[Pubkey],
        mint_authority: &Pubkey,
        expected_authority: &Pubkey
    ) -> Result<(), ProgramError> {

        if !cmp_pubkeys(mint_authority, expected_authority) {
            return Err(ZionError::InvalidSwapAuthority.into());
        }
        

        for owner in token_account_owners.iter() {
            if !cmp_pubkeys(&owner, &expected_authority) {
                return Err(ZionError::InvalidOwner.into());
            }
        }

        Ok(())
    }

    
    ///Create the swap_authority PDA and compare it to the provided swap_authority
    fn validate_swap_authority_key (
        swap_authority: &AccountInfo,
        swap_authority_bump: u8,
    ) -> Result<(), ProgramError> {
        let authority =
            Pubkey::create_program_address(
                &[
                    AUTHORITY_PREFIX.as_bytes(),
                    &[swap_authority_bump],
                ],
                &crate::id(),
            ).expect("Invalid swap authority");

        if cmp_pubkeys(swap_authority.key, &authority) {
            return Ok(())
        };
        return Err(ZionError::InvalidSwapAuthority.into())
    }

    ///check if any data exists for account
    pub fn assert_uninitialized(account: &AccountInfo) -> ProgramResult {
        if !account.data_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        Ok(())
    }

    /// Processes an [Instruction](enum.ZionInstruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = ZionInstruction::unpack(input)?;

        match instruction {
            ZionInstruction::Initialize(Initialize { swap_state }) => {
                msg!("Instruction: Initialize");
                Self::process_initialize(program_id, swap_state, accounts)
            },
            ZionInstruction::AdminDeposit(AdminDeposit { token_a_deposit, token_b_deposit }) => {
                msg!("Instruction: AdminDeposit");
                Self::process_admin_deposit(program_id, accounts, token_a_deposit, token_b_deposit)
            },
            ZionInstruction::Deposit(Deposit { token_a_deposit, token_b_deposit }) => {
                msg!("Instruction: Deposit");
                Self::process_deposit(program_id, accounts, token_a_deposit, token_b_deposit, )
            },
            ZionInstruction::Withdraw(Withdraw { token_a_withdraw, token_b_withdraw }) => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(program_id, accounts, token_a_withdraw, token_b_withdraw, )
            },
            ZionInstruction::Swap(Swap { amount }) => {
                msg!("Instruction: Swap");
                Self::process_swap(program_id, accounts, amount)
            },
            ZionInstruction::ClosePool() => {
                msg!("Instruction: ClosePool");
                Self::process_close_pool(program_id, accounts)
            }
        }
    }
    
    ///Initialize the swap pool
    pub fn process_initialize(
        _: &Pubkey,
        swap_state: SwapState,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let swap_authority_info = next_account_info(account_info_iter)?;
        let swap_mint_info = next_account_info(account_info_iter)?;
        let swap_state_info = next_account_info(account_info_iter)?;

        let token_a_mint_info = next_account_info(account_info_iter)?;
        let token_a_vault_info = next_account_info(account_info_iter)?;
        let token_a_fee_vault_info = next_account_info(account_info_iter)?;
        let token_a_oracle_info = next_account_info(account_info_iter)?;

        let token_b_mint_info = next_account_info(account_info_iter)?;
        let token_b_vault_info = next_account_info(account_info_iter)?;
        let token_b_fee_vault_info = next_account_info(account_info_iter)?;
        let token_b_oracle_info = next_account_info(account_info_iter)?;

        let token_program_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        

        let token_program_id = *token_program_info.key;
        let rent = Rent::from_account_info(rent_info)?;
        
        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }
        
        //validate system program key
        if !cmp_pubkeys(system_program_info.key, &system_program::id()) {
            return Err(ZionError::InvalidSystemProgramKey.into());
        }

        if !admin_info.is_signer {
            return Err(ZionError::InvalidSigner.into());
        }

        //validate swap state key
        SwapState::validate_swap_state_key(swap_state_info.key)?;
        Self::assert_uninitialized(swap_state_info)?;

        //validate swap authority key
        Self::validate_swap_authority_key(
            swap_authority_info,
            swap_state.swap_authority_bump,
        )?;
 

        //validate swap mint
        let swap_mint = Self::unpack_mint(swap_mint_info)?;
        if swap_mint.mint_authority != COption::Some(swap_state.swap_authority) {
            return Err(ZionError::InvalidAuthority.into());
        }
        if swap_mint.supply != 0 {
            return Err(ZionError::InvalidSupply.into());
        }

        //validate token A mint && accounts
        let token_a_mint = Self::unpack_mint(token_a_mint_info)?;
        let mint_a_authority = token_a_mint.mint_authority.ok_or(ZionError::InvalidAuthority)?;
        let token_a_vault = Self::unpack_token_account(token_a_vault_info)?;
        let token_a_fee_vault = Self::unpack_token_account(token_a_fee_vault_info)?;
         
        Processor::validate_mint_and_token_accounts(
            &[token_a_vault.owner, token_a_fee_vault.owner],
            &mint_a_authority,
            &swap_state.swap_authority
        )?;
        
        
        //validate token B mint & accounts
        let token_b_mint = Self::unpack_mint(token_b_mint_info)?;
        let mint_b_authority = token_b_mint.mint_authority.ok_or(ZionError::InvalidAuthority)?;
        let token_b_vault = Self::unpack_token_account(token_b_vault_info)?;
        let token_b_fee_vault = Self::unpack_token_account(token_b_fee_vault_info)?;
        
        Processor::validate_mint_and_token_accounts(
            &[token_b_vault.owner,
            token_b_fee_vault.owner],
            &mint_b_authority,
            &swap_state.swap_authority
        )?;
        

        if token_a_mint_info.key == token_b_mint_info.key {
            return Err(ZionError::IdenticalMints.into());
        }
        
        //create swap state pda account
        create_pda_account(
            admin_info,
            &rent, 
            SwapState::LEN,
            &crate::id(),
            system_program_info,
            swap_state_info,
            &[SwapState::PREFIX.as_bytes(), &[swap_state.bump]],
        )?;

        //create swap authority pda account
        create_pda_account(
            admin_info,
            &rent, 
            0,
            &crate::id(),
            system_program_info,
            swap_authority_info,
            &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.swap_authority_bump]],
        )?;

        let obj = SwapState {
            admin: *admin_info.key,
            bump: swap_state.bump,
            is_initialized: true,
            swap_authority: swap_state.swap_authority,
            swap_authority_bump: swap_state.swap_authority_bump,
            swap_mint: *swap_mint_info.key,
            token_a: Token {
                mint: token_a_mint_info.key.clone(),
                vault: token_a_vault_info.key.clone(),
                fee_vault: token_a_fee_vault_info.key.clone(),
                oracle:  token_a_oracle_info.key.clone()
            },
            token_b: Token {
                mint: token_b_mint_info.key.clone(),
                vault: token_b_vault_info.key.clone(),
                fee_vault: token_b_fee_vault_info.key.clone(),
                oracle:  token_b_oracle_info.key.clone()
            },
            program_fee: swap_state.program_fee,
            swap_fee: swap_state.swap_fee,
        };
        SwapState::pack(obj, &mut swap_state_info.data.borrow_mut())?;

       
        Ok(())
    }

    ///Admin instruction to deposit tokens priced at the markets value and not the protocols value.
    ///Should be used after initializing the pool to provide initial liquidity or in emergency situations
    pub fn process_admin_deposit(
        _: &Pubkey,
        accounts: &[AccountInfo],
        token_a_deposit: u64,
        token_b_deposit: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let swap_authority_info = next_account_info(account_info_iter)?;
        let swap_mint_info = next_account_info(account_info_iter)?;
        let swap_state_info = next_account_info(account_info_iter)?;
        let admin_swap_wallet = next_account_info(account_info_iter)?;

        let token_a_mint_info = next_account_info(account_info_iter)?;
        let token_a_admin_wallet= next_account_info(account_info_iter)?;
        let token_a_vault_info= next_account_info(account_info_iter)?;
        let token_a_oracle_info = next_account_info(account_info_iter)?;

        let token_b_mint_info = next_account_info(account_info_iter)?;
        let token_b_admin_wallet = next_account_info(account_info_iter)?;
        let token_b_vault_info = next_account_info(account_info_iter)?;
        let token_b_oracle_info = next_account_info(account_info_iter)?;

        let token_program_info = next_account_info(account_info_iter)?;
        let token_program_id = *token_program_info.key;

        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }
        
        //validate swap state key
        SwapState::validate_swap_state_key(swap_state_info.key)?;

        let swap_state_data = swap_state_info.try_borrow_data()?;
        let swap_state = SwapState::unpack_from_slice(&swap_state_data)?;
        
        //validate signer
        if !admin_info.is_signer {
            return Err(ZionError::InvalidSigner.into());
        }

        swap_state.validate_swap_state_authority(swap_authority_info.key)?;
        
        //validate mints
        if token_a_mint_info.key != &swap_state.token_a.mint {
            return Err(ZionError::InvalidMint.into());
        }
        if token_b_mint_info.key != &swap_state.token_b.mint {
            return Err(ZionError::InvalidMint.into());
        }
        if swap_mint_info.key != &swap_state.swap_mint {
            return Err(ZionError::InvalidMint.into());
        }

        //validate oracles
        if token_a_oracle_info.key != &swap_state.token_a.oracle {
            return Err(ZionError::InvalidOracle.into());
        }
        if token_b_oracle_info.key != &swap_state.token_b.oracle {
            return Err(ZionError::InvalidOracle.into());
        }

        //validate vaults
        if token_a_vault_info.key != &swap_state.token_a.vault {
            return Err(ZionError::InvalidVault.into());
        }
        if token_b_vault_info.key != &swap_state.token_b.vault {
            return Err(ZionError::InvalidVault.into());
        }

        //load oracle prices
        let token_a_price_feed: PriceFeed = load_price_feed_from_account_info(&token_a_oracle_info ).unwrap();
        let token_a_price = token_a_price_feed.get_price_unchecked().price.try_into().unwrap();

        let token_b_price_feed: PriceFeed = load_price_feed_from_account_info(&token_b_oracle_info ).unwrap();
        let token_b_price = token_b_price_feed.get_price_unchecked().price.try_into().unwrap();

        //transfer tokens from token_a_admin_wallet to vault
        let token_a_swap_tokens = if token_a_deposit > 0 {
            token_transfer(
                token_program_info, 
                token_a_admin_wallet,
                token_a_vault_info,
                admin_info,
                token_a_deposit,

            )?;
            swap_state.token_a.get_market_value(token_a_deposit, token_a_price).to_imprecise().expect("a valid number") as u64

        } else {
            0
        };

        //transfer tokens from token_b_admin_wallet to vault
        let token_b_swap_tokens = if token_b_deposit > 0 {
            token_transfer(
                token_program_info, 
                token_b_admin_wallet,
                token_b_vault_info,
                admin_info,
                token_b_deposit,

            )?;
            swap_state.token_b.get_market_value(token_b_deposit, token_b_price).to_imprecise().expect("a valid number") as u64
        
        } else {
            0
        };

        //mint swap pool tokens to admin wallet
        token_mint_to(
            token_program_info, 
            swap_mint_info,
            admin_swap_wallet,
            swap_authority_info,
            token_a_swap_tokens + token_b_swap_tokens,
            &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.swap_authority_bump]],

        )?;

        Ok(())
    }

    ///User instruction to deposit tokens priced at the protocols value and not the market value.
    pub fn process_deposit(
        _: &Pubkey,
        accounts: &[AccountInfo],
        token_a_deposit: u64,
        token_b_deposit: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let swap_state_info = next_account_info(account_info_iter)?;
        let swap_authority_info = next_account_info(account_info_iter)?;
        let swap_mint_info = next_account_info(account_info_iter)?;
        let swap_token_user_info = next_account_info(account_info_iter)?;

        let token_a_user_info = next_account_info(account_info_iter)?;
        let token_a_vault_info = next_account_info(account_info_iter)?;
        let token_a_fee_vault_info = next_account_info(account_info_iter)?;
        let token_a_oracle_info = next_account_info(account_info_iter)?;

        let token_b_user_info = next_account_info(account_info_iter)?;
        let token_b_vault_info = next_account_info(account_info_iter)?;
        let token_b_fee_vault_info = next_account_info(account_info_iter)?;
        let token_b_oracle_info = next_account_info(account_info_iter)?;

        let token_program_info = next_account_info(account_info_iter)?;
       
        let token_program_id = *token_program_info.key;

        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }

        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }

        //validate signer
        if !user.is_signer {
            return Err(ZionError::InvalidSigner.into());
        }
        
        //validate swap state key
        SwapState::validate_swap_state_key(swap_state_info.key)?;

        let swap_state_data = swap_state_info.try_borrow_data()?;
        let swap_state = SwapState::unpack_from_slice(&swap_state_data)?;

        swap_state.validate_accounts(
            swap_authority_info.key,
            swap_mint_info.key,
            &swap_state.token_a.mint,
            token_a_vault_info.key,
            token_a_fee_vault_info.key,
            token_a_oracle_info.key,
            &swap_state.token_b.mint,
            token_b_vault_info.key,
            token_b_fee_vault_info.key,
            token_b_oracle_info.key
        )?;

        let token_a_vault = Self::unpack_token_account(token_a_vault_info)?;
        let token_a_fee_vault = Self::unpack_token_account(token_a_fee_vault_info)?;

        let token_b_vault = Self::unpack_token_account(token_b_vault_info)?;
        let token_b_fee_vault = Self::unpack_token_account(token_b_vault_info)?;

        let swap_mint = Self::unpack_mint(swap_mint_info)?;

        //load prices from oracle
        let token_a_price_feed: PriceFeed = load_price_feed_from_account_info(&token_a_oracle_info ).unwrap();
        let token_a_price = token_a_price_feed.get_price_unchecked().price.try_into().unwrap();

        let token_b_price_feed: PriceFeed = load_price_feed_from_account_info(&token_b_oracle_info ).unwrap();
        let token_b_price = token_b_price_feed.get_price_unchecked().price.try_into().unwrap();

        //transfer tokens from user token_a wallet to vault
        let token_a_swap_tokens = if token_a_deposit > 0 {
            token_transfer(
                token_program_info, 
                token_a_user_info,
                token_a_vault_info,
                user,
                token_a_deposit,

            )?;

            swap_state.calculate_swap_tokens(
                token_a_deposit,
                token_a_vault.amount,
                token_a_price,
                token_a_fee_vault.amount,
                token_b_vault.amount,
                token_b_price,
                token_b_fee_vault.amount,
                swap_mint.supply
            )

        } else {
            0
        };

        //transfer tokens from user token_b wallet to vault
        let token_b_swap_tokens = if token_b_deposit > 0 {
            token_transfer(
                token_program_info, 
                token_b_user_info,
                token_b_vault_info,
                user,
                token_b_deposit,

            )?;

            swap_state.calculate_swap_tokens(
                token_b_deposit,
                token_b_vault.amount,
                token_b_price,
                token_b_fee_vault.amount,
                token_a_vault.amount,
                token_a_price,
                token_a_fee_vault.amount,
                swap_mint.supply
            )
        } else {
            0
        };

        //mint swap tokens to user swap wallet
        token_mint_to(
            token_program_info, 
            swap_mint_info,
            swap_token_user_info,
            swap_authority_info,
            token_a_swap_tokens + token_b_swap_tokens,
            &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.swap_authority_bump]],

        )?;
        


        Ok(())
    }

    ///Instruction to withdraw x token_a and y token_b from vault at protocol value
    pub fn process_withdraw(
        _: &Pubkey,
        accounts: &[AccountInfo],
        token_a_withdraw: u64,
        token_b_withdraw: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let swap_state_info = next_account_info(account_info_iter)?;
        let swap_authority_info = next_account_info(account_info_iter)?;
        let swap_mint_info = next_account_info(account_info_iter)?;
        let swap_token_user_info = next_account_info(account_info_iter)?;

        let token_a_user_info = next_account_info(account_info_iter)?;
        let token_a_vault_info = next_account_info(account_info_iter)?;
        let token_a_fee_vault_info = next_account_info(account_info_iter)?;
        let token_a_oracle_info = next_account_info(account_info_iter)?;

        let token_b_user_info = next_account_info(account_info_iter)?;
        let token_b_vault_info = next_account_info(account_info_iter)?;
        let token_b_fee_vault_info = next_account_info(account_info_iter)?;
        let token_b_oracle_info = next_account_info(account_info_iter)?;

        let token_program_info = next_account_info(account_info_iter)?;
       
        let token_program_id = *token_program_info.key;


        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }

        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }

        //validate signer
        if !user.is_signer {
            return Err(ZionError::InvalidSigner.into());
        }
        
        //validate swap state key
        SwapState::validate_swap_state_key(swap_state_info.key)?;

        let swap_state_data = swap_state_info.try_borrow_data()?;
        let swap_state = SwapState::unpack_from_slice(&swap_state_data)?;

        swap_state.validate_accounts(
            swap_authority_info.key,
            swap_mint_info.key,
            &swap_state.token_a.mint,
            token_a_vault_info.key,
            token_a_fee_vault_info.key,
            token_a_oracle_info.key,
            &swap_state.token_b.mint,
            token_b_vault_info.key,
            token_b_fee_vault_info.key,
            token_b_oracle_info.key
        )?;

        let token_a_vault = Self::unpack_token_account(token_a_vault_info)?;
        let token_a_fee_vault = Self::unpack_token_account(token_a_fee_vault_info)?;

        let token_b_vault = Self::unpack_token_account(token_b_vault_info)?;
        let token_b_fee_vault = Self::unpack_token_account(token_b_vault_info)?;

        let swap_mint = Self::unpack_mint(swap_mint_info)?;
        let swap_token_user = Self::unpack_token_account(swap_token_user_info)?;

        let token_a_price_feed: PriceFeed = load_price_feed_from_account_info(&token_a_oracle_info ).unwrap();
        let token_a_price = token_a_price_feed.get_price_unchecked().price.try_into().unwrap();

        let token_b_price_feed: PriceFeed = load_price_feed_from_account_info(&token_b_oracle_info ).unwrap();
        let token_b_price = token_b_price_feed.get_price_unchecked().price.try_into().unwrap();

        //calculate how many swap tokens are needed for token_a_withdraw amount
        let token_a_swap_tokens = if token_a_withdraw > 0 {
            swap_state.calculate_swap_tokens(
                token_a_withdraw,
                token_a_vault.amount,
                token_a_price,
                token_a_fee_vault.amount,
                token_b_vault.amount,
                token_b_price,
                token_b_fee_vault.amount,
                swap_mint.supply
            )

        } else {
            0
        };

        
        //calculate how many swap tokens are needed for token_abwithdraw amount
        let token_b_swap_tokens = if token_b_withdraw > 0 {
            swap_state.calculate_swap_tokens(
                token_b_withdraw,
                token_b_vault.amount,
                token_b_price,
                token_b_fee_vault.amount,
                token_a_vault.amount,
                token_a_price,
                token_a_fee_vault.amount,
                swap_mint.supply
            )
            
        } else {
            0
        };

        if (token_a_swap_tokens + token_b_swap_tokens) < swap_token_user.amount {
            if token_a_withdraw > 0 {
                msg!("Withdrawing {} tokens from pool A",token_a_withdraw);
                token_transfer_signed(
                    token_program_info, 
                    token_a_vault_info,
                    token_a_user_info,
                    swap_authority_info,
                    token_a_withdraw,
                    &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.bump]],
                )?;
            }

            if token_b_withdraw > 0 {
                msg!("Withdrawing {} tokens from pool B",token_b_withdraw);
                token_transfer_signed(
                    token_program_info, 
                    token_b_vault_info,
                    token_b_user_info,
                    swap_authority_info,
                    token_b_withdraw,
                    &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.bump]],
                )?;
            }

            msg!("Burning {} swap tokens", token_a_swap_tokens + token_b_swap_tokens);
            token_burn(
                token_program_info,
                swap_token_user_info,
                swap_mint_info,
                user,
                token_a_swap_tokens + token_b_swap_tokens,
                &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.swap_authority_bump]],
    
            )?;

        } else {
            msg!("{} swap tokens required for withdrawl but only {} available", token_a_swap_tokens + token_b_swap_tokens, swap_token_user.amount);
            return Err(ZionError::InsufficientSwapTokens.into());
        }
        
        Ok(())

    }

    ///Instructions to swap tokens
    pub fn process_swap(
        _: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let swap_state_info = next_account_info(account_info_iter)?;
        let swap_authority_info = next_account_info(account_info_iter)?;

        let source_user_info = next_account_info(account_info_iter)?;
        let source_vault_info = next_account_info(account_info_iter)?;
        let source_fee_vault_info = next_account_info(account_info_iter)?;
        let source_oracle_info = next_account_info(account_info_iter)?;

        let destination_user_info = next_account_info(account_info_iter)?;
        let destination_vault_info = next_account_info(account_info_iter)?;
        let destination_fee_vault_info = next_account_info(account_info_iter)?;
        let destination_oracle_info = next_account_info(account_info_iter)?;

        let token_program_info = next_account_info(account_info_iter)?;
       
        let token_program_id = *token_program_info.key;


        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }

        //validate token program key
        if !cmp_pubkeys(&token_program_id, &spl_token::id()) {
            return Err(ZionError::InvalidTokenProgramKey.into());
        }
         
        //validate signer
        if !user.is_signer {
            return Err(ZionError::InvalidSigner.into());
        }
        
        //validate swap state key
        SwapState::validate_swap_state_key(swap_state_info.key)?;

        let swap_state_data = swap_state_info.try_borrow_data()?;
        let swap_state = SwapState::unpack_from_slice(&swap_state_data)?;

        //validate swap authority key
        swap_state.validate_swap_state_authority(swap_authority_info.key)?;

        let source_vault_data = Self::unpack_token_account(source_vault_info)?;
        let destination_vault_data = Self::unpack_token_account(destination_vault_info)?;
        if destination_vault_data.amount == 0 {
            return Err(ZionError::InvalidSupply.into());
        }

        //load prices from oracle
        let source_price_feed: PriceFeed = load_price_feed_from_account_info(&source_oracle_info ).unwrap();
        let destination_price_feed: PriceFeed = load_price_feed_from_account_info(&destination_oracle_info ).unwrap();

        //validate oracles
        if destination_oracle_info.key == source_oracle_info.key {
            return Err(ZionError::InvalidOracle.into());
        }

        //validate accounts
        let (source_token, source_price, destination_token, destination_price) = if *source_vault_info.key == swap_state.token_a.vault {
            
            //validate vaults
            if source_vault_info.key != &swap_state.token_a.vault {
                return Err(ZionError::InvalidVault.into());
            }
            if destination_vault_info.key != &swap_state.token_b.vault {
                return Err(ZionError::InvalidVault.into());
            }

            //validate fee vaults
            if source_fee_vault_info.key != &swap_state.token_a.fee_vault {
                return Err(ZionError::InvalidVault.into());
            }
            if destination_fee_vault_info.key != &swap_state.token_b.fee_vault {
                return Err(ZionError::InvalidVault.into());
            }
            
            let source_price: Price = source_price_feed.get_price_unchecked();
            let destination_price: Price = destination_price_feed.get_price_unchecked();

            (swap_state.token_a, source_price.price, swap_state.token_b, destination_price.price)

        } else {

            //validate vaults
            if destination_vault_info.key != &swap_state.token_a.vault {
                return Err(ZionError::InvalidVault.into());
            }
            if source_vault_info.key != &swap_state.token_b.vault {
                return Err(ZionError::InvalidVault.into());
            }

            //validate fee vaults
            if destination_fee_vault_info.key != &swap_state.token_a.fee_vault {
                return Err(ZionError::InvalidVault.into());
            }
            if source_fee_vault_info.key != &swap_state.token_b.fee_vault {
                return Err(ZionError::InvalidVault.into());
            }

            let source_price: Price = source_price_feed.get_price_unchecked();
            let destination_price: Price = destination_price_feed.get_price_unchecked();

            (swap_state.token_b, destination_price.price, swap_state.token_a, source_price.price)
        };

        //calculate how mant destination tokens user receives for source_tokens
        let destination_amount = SwapState::calculate_tokens_to_swap(
            source_token,
            source_vault_data.amount,
            source_price.try_into().unwrap(),
            destination_token,
            destination_price.try_into().unwrap(),
            destination_vault_data.amount,
            amount,
        );

        msg!("Swapping {} tokens from source pool", amount);
        token_transfer(
            token_program_info, 
            source_user_info,
            source_vault_info,
            user,
            amount,

        )?;

        msg!("Swapping {} tokens from destination pool", destination_amount);
        token_transfer_signed(
            token_program_info, 
            destination_vault_info,
            destination_user_info,
            swap_authority_info,
            destination_amount,
            &[AUTHORITY_PREFIX.as_bytes(), &[swap_state.bump]],
        )?;

        Ok(())
    }

    ///Instruction to close swap pool
    pub fn process_close_pool(
        _: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let admin_info = next_account_info(account_info_iter)?;
        let swap_state_info = next_account_info(account_info_iter)?;
        let swap_authority_info = next_account_info(account_info_iter)?;

        //validate swap state key
        SwapState::validate_swap_state_key(swap_state_info.key)?;

        let swap_state_data = swap_state_info.try_borrow_data()?;
        let swap_state = SwapState::unpack_from_slice(&swap_state_data)?;

        //validate admin
        if &swap_state.admin != admin_info.key {
            return Err(ZionError::MustBeAdmin.into());
        }

        //validate signer
        if !admin_info.is_signer {
            return Err(ZionError::InvalidSigner.into());
        }

        //validate swap authority key
        swap_state.validate_swap_state_authority(swap_authority_info.key)?;

        let lamports = swap_state_info.lamports();
        let admin_lamports = admin_info.lamports();

        **admin_info.lamports.borrow_mut() = admin_lamports + lamports;
        **swap_state_info.lamports.borrow_mut() = 0;

        Ok(())
    } 

}

///compare two Pubkeys
pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

