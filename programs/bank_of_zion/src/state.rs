use crate::error::ZionError;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{
    program_error::ProgramError,
    program_pack::{ Pack, Sealed},
    pubkey::Pubkey,
    pubkey::PUBKEY_BYTES,
    program_memory::sol_memcmp,
    
};
use spl_math::precise_number::PreciseNumber;

///Prefix used in generating the PDA for the swap authority
pub const AUTHORITY_PREFIX: &str = "swap_authority";

/// Program states.
#[repr(C)]
#[derive(Debug, Default, PartialEq)]
pub struct SwapState {

    ///admin of the Swap Pool
    pub admin: Pubkey,
    ///bump of the SwapState pda
    pub bump: u8,
    /// is the pool initialized
    pub is_initialized: bool,
    ///PDA that owns/controls the vaults and mints
    pub swap_authority: Pubkey,
    ///bump of the swap authority pda
    pub swap_authority_bump: u8,
    ///Mint for the swap tokens
    pub swap_mint: Pubkey,

    ///First token in the swap pool
    pub token_a: Token,
    ///Second token in the swap pool
    pub token_b: Token,
    ///basis point fee applied to transactions that are given to the admin
    pub program_fee: u64, //wip, next version

    ///basis point fee applied to transactios that are given to the user
    pub swap_fee: u64 //wip, next version
}
impl Sealed for SwapState {}
impl Pack for SwapState {
    const LEN: usize = 371;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 371];
        let (
            admin,
            bump,
            is_initialized,
            swap_authority,
            swap_authority_bump,
            swap_mint,
            token_a,
            token_b,
            program_fee,
            swap_fee,
        ) = mut_array_refs![output, 32, 1, 1, 32, 1, 32, 128, 128, 8, 8];
        admin.copy_from_slice(self.admin.as_ref());
        *bump = self.bump.to_le_bytes();
        is_initialized[0] = self.is_initialized as u8;
        swap_authority.copy_from_slice(self.swap_authority.as_ref());
        *swap_authority_bump = self.swap_authority_bump.to_le_bytes();
        swap_mint.copy_from_slice(self.swap_mint.as_ref());
        self.token_a.pack_into_slice(&mut token_a[..]);
        self.token_b.pack_into_slice(&mut token_b[..]);
        *program_fee = self.program_fee.to_le_bytes();
        *swap_fee = self.swap_fee.to_le_bytes();
    }
    
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, 371];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            admin,
            bump,
            is_initialized,
            swap_authority,
            swap_authority_bump,
            swap_mint,
            token_a,
            token_b,
            program_fee,
            swap_fee,
        ) = array_refs![input, 32, 1, 1, 32, 1, 32, 128, 128, 8, 8];
        Ok(Self {
            admin: Pubkey::new_from_array(*admin),
            bump: u8::from_be_bytes(*bump),
            is_initialized: match is_initialized {
                [0] => false,
                [1] => true,
                _ => return Err(ProgramError::InvalidAccountData),
            },
            swap_authority: Pubkey::new_from_array(*swap_authority),
            swap_authority_bump: u8::from_be_bytes(*swap_authority_bump),
            swap_mint: Pubkey::new_from_array(*swap_mint),
            token_a: Token::unpack_from_slice(token_a)?,
            token_b: Token::unpack_from_slice(token_b)?,
            program_fee: u64::from_le_bytes(*program_fee),
            swap_fee: u64::from_le_bytes(*swap_fee),
        })
    }
}

impl Clone for SwapState {
    fn clone(&self) -> Self {
        let mut packed_self = [0u8; Self::LEN];
        Self::pack_into_slice(self, &mut packed_self);
        Self::unpack_from_slice(&packed_self).unwrap()
    }
}

impl SwapState {
    
    ///Prefix for generating the PDA for the swap state
    pub const PREFIX: &'static str = "swap_state";

    ///tmp var, delete it
    pub const DECIMALS: u8 = 3;

    ///validate the swap state pubkey and owner
    pub fn validate(&self, swap_state_key: &Pubkey, swap_state_owner: &Pubkey) -> Result<(), ProgramError> {
        
        SwapState::validate_swap_state_key(swap_state_key)?;
        self.validate_swap_state_owner(swap_state_owner)?;

        if self.is_initialized {
            return Err(ZionError::PoolAlreadyInitialized.into());
        };

        Ok(())
    }

    ///validate the owner of the swap state
    pub fn validate_swap_state_owner (
        &self,
        swap_state_owner: &Pubkey,
    ) -> Result<(), ProgramError> {
        if !cmp_pubkeys(swap_state_owner, &crate::id()) {
            return Err(ZionError::SwapStateWrongOwner.into());
        };

        return Ok(())
    }
    
    ///validate the pubkey of the swap state
    pub fn validate_swap_state_key (
       swap_state_key: &Pubkey,
    ) -> Result<(), ProgramError> {
        let (key, _) = Pubkey::find_program_address(
            &[ SwapState::PREFIX.as_bytes()], &crate::id()
        );

        if !cmp_pubkeys(swap_state_key, &key)
        {
            return Err(ZionError::InvalidSwapState.into())
        }
        return Ok(())
    }

    ///validate all of the accounts associated with the swap state
    pub fn validate_accounts (
        &self,
        swap_authority: &Pubkey,
        swap_mint: &Pubkey,
        token_a_mint: &Pubkey,
        token_a_vault: &Pubkey,
        token_a_fee_vault: &Pubkey,
        token_a_oracle: &Pubkey,
        token_b_mint: &Pubkey,
        token_b_vault: &Pubkey,
        token_b_fee_vault: &Pubkey,
        token_b_oracle: &Pubkey,

    ) -> Result<(), ProgramError> {
        self.validate_swap_state_authority(swap_authority)?;
        self.validate_swap_mint(swap_mint)?;
        self.token_a.validate_accounts(token_a_mint, token_a_vault, token_a_fee_vault, token_a_oracle)?;
        self.token_b.validate_accounts(token_b_mint, token_b_vault, token_b_fee_vault, token_b_oracle)?;


        return Ok(())
    }

    ///validate the swap authority against the swap_state.authority
    pub fn validate_swap_state_authority (
        &self,
        swap_authority: &Pubkey
    ) -> Result<(), ProgramError> {
        if !cmp_pubkeys(&self.swap_authority, swap_authority) {
            return Err(ZionError::InvalidSwapAuthority.into());
        };

        return Ok(())
    }
    
    ///validate the swap mint against the swap_state.swap_mint
    pub fn validate_swap_mint (
        &self,
        swap_mint: &Pubkey
    ) -> Result<(), ProgramError> {
        if !cmp_pubkeys(&self.swap_mint, swap_mint) {
            return Err(ZionError::InvalidSwapMint.into());
        };

        return Ok(())
    }

    ///calculate the prermium the protocol will pay to balance  the pool
    pub fn get_price_premium(
        vault_a_value: PreciseNumber,
        vault_b_value: PreciseNumber,
    ) -> PreciseNumber {
        let one = PreciseNumber::new(1 as u128).expect("one");
        let zero = PreciseNumber::new(0 as u128).expect("zero");

        //can't be zero or the math breaks, for now min is 1
        let tmp_vault_a_value = if vault_a_value.less_than_or_equal(&zero) {
            one.clone()
        } else {
            vault_a_value.clone()
        };

        //can't be zero or the math breaks, for now min is 1
        let tmp_vault_b_value = if vault_b_value.less_than_or_equal(&zero) {
            one.clone()
        } else {
            vault_b_value.clone()
        };

        tmp_vault_b_value.checked_div(&tmp_vault_a_value).expect("a valid number")
    }
    
    ///calculate how much token_a and token_b to be deposited aswell as how many swap tokens received
    pub fn calculate_swap_tokens (
        &self,
        tokens_deposit: u64,
        vault_a_supply: u64,
        token_a_market_price: u64,
        fee_vault_a_supply: u64,
        vault_b_supply: u64,
        token_b_market_price: u64,
        fee_vault_b_supply: u64,
        swap_supply: u64,
    ) -> u64 {

        let zero = PreciseNumber::new(0 as u128).expect("zero");
        let one = PreciseNumber::new(1 as u128).expect("one");
        let swap_supply = PreciseNumber::new(swap_supply as u128).expect("swap_supply");

        //total value of tokens in vault a
        let vault_a_value = self.token_a.get_market_value(vault_a_supply, token_a_market_price);
        
        //total value of tokens in vault b
        let vault_b_value = self.token_b.get_market_value(vault_b_supply, token_b_market_price);

        let price_premium = Self::get_price_premium(vault_a_value.clone(), vault_b_value.clone());

        let tokens_deposit = PreciseNumber::new(tokens_deposit as u128).expect("a valid number");
        
        //value of tokens user is depositing
        let tokens_deposit_value = tokens_deposit.checked_mul(&price_premium).expect("a valid number");
        
        let fee_vault_a_value = self.token_a.get_market_value(fee_vault_a_supply, token_a_market_price);
        let fee_vault_b_value = self.token_b.get_market_value(fee_vault_b_supply, token_b_market_price);
        
        //total value of recoverable funds in the protocol
        let mut total_protocol_value = 
            vault_a_value.checked_add(&vault_b_value).expect("a valid number")
                .checked_add(&fee_vault_a_value).expect("a valid number")
                    .checked_add(&fee_vault_b_value).expect("a valid number");

        //can't be zero or the math breaks, for now min is 1
        if total_protocol_value.less_than_or_equal(&zero) {
            total_protocol_value = one.clone();
        }
        
        //percentage value of users deposit to total value of funds in the protocol
        let percent = tokens_deposit_value.checked_div(&total_protocol_value).expect("a valid number");
        
        //miltiply % of user value contributed to total protocol value against total swap tokens to get how many tokens the user should receive
        let swap_tokens_from_deposit = swap_supply.checked_mul(&percent).expect("a valid number");
        
        return swap_tokens_from_deposit.to_imprecise().expect("a valid number") as u64

    }
    
    ///calculate how many destination tokens a user receives when swapping source tokens
    pub fn calculate_tokens_to_swap (
        source: Token,
        source_supply: u64,
        source_market_price: u64,
        destination: Token,
        destination_market_price: u64,
        destination_supply: u64,
        token_amount: u64,

    ) -> u64 {
        let token_amount = PreciseNumber::new(token_amount as u128).expect("a valid number");
       
        //total value of tokens in vault a
        let source_value = source.get_market_value(source_supply, source_market_price);
        
        //total value of tokens in vault b
        let destination_value = destination.get_market_value(destination_supply, destination_market_price);
        
        let price_premium = Self::get_price_premium(source_value, destination_value);
        
        let source_value = Token::get_protocol_price(source_market_price, price_premium)
            .checked_mul(&token_amount).expect("a valid number");
        
        let tokens_receive = source_value
            .checked_div(&PreciseNumber::new(destination_market_price as u128).expect("a valid number"))
            .expect("a valid number")
            .floor().expect("a valid number");
        
        tokens_receive.to_imprecise().expect("a valid number") as u64
        

    }

}

/// Program states.
#[repr(C)]
#[derive(Debug, Default, PartialEq)]
pub struct Token {
    ///Mint pubkey for the token
    pub mint: Pubkey, //32
    ///pubkey for the vault associated with this token
    pub vault: Pubkey, //64
    ///Token account where fee's are stored for this token
    pub fee_vault: Pubkey, //96
    ///basis point fee applied to transactions that
    pub oracle: Pubkey //128
}

impl Sealed for Token {}
impl Pack for Token {
    const LEN: usize = 128;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 128];
        let (
            mint,
            vault,
            fee_vault,
            oracle,
        ) = mut_array_refs![output, 32, 32, 32, 32];
        mint.copy_from_slice(self.mint.as_ref());
        vault.copy_from_slice(self.vault.as_ref());
        fee_vault.copy_from_slice(self.fee_vault.as_ref());
        oracle.copy_from_slice(self.oracle.as_ref());
    }

    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, 128];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            mint,
            vault,
            fee_vault,
            oracle,
        ) = array_refs![input, 32, 32, 32, 32];
        Ok(Self {
            mint: Pubkey::new_from_array(*mint),
            vault: Pubkey::new_from_array(*vault),
            fee_vault: Pubkey::new_from_array(*fee_vault),
            oracle: Pubkey::new_from_array(*oracle),
        })
    }
}
impl Clone for Token {
    fn clone(&self) -> Self {
        let mut packed_self = [0u8; Self::LEN];
        Self::pack_into_slice(self, &mut packed_self);
        Self::unpack_from_slice(&packed_self).unwrap()
    }
}

impl Token {

    ///validate the token accounts against the Token struct
    pub fn validate_accounts (
        &self,
        mint: &Pubkey,
        vault: &Pubkey,
        fee_vault: &Pubkey,
        oracle: &Pubkey,
    ) -> Result<(), ProgramError> {
        if !cmp_pubkeys(&self.mint, mint) {
            return Err(ZionError::InvalidMint.into());
        };
        if !cmp_pubkeys(&self.vault, vault) {
            return Err(ZionError::InvalidVault.into());
        };
        if !cmp_pubkeys(&self.fee_vault, fee_vault) {
            return Err(ZionError::InvalidFeeVault.into());
        };
        if !cmp_pubkeys(&self.oracle, oracle) {
            return Err(ZionError::InvalidOracle.into());
        };

        return Ok(())
    }

    ///calculate the value of the tokens
    fn calculate_market_value (
        price: PreciseNumber,
        supply: u64,
        //what about decimals?
    ) -> PreciseNumber {
        let supply = PreciseNumber::new(supply as u128).expect("valid number");
        let value = price.checked_mul(&supply).expect("valid number");

        //maybe remove this, change formula so its never 0
        //if value.less_than_or_equal(&zero) {
        //    return PreciseNumber::new(1 as u128).expect("zero");
        //}
        return value
    }
    
    ///retrieve the value of the tokens
    pub fn get_market_value (
        &self,
        amount: u64,
        market_price: u64
    ) -> PreciseNumber {
        let market_price = PreciseNumber::new(market_price as u128).expect("market_price");
        Token::calculate_market_value(market_price, amount)
    }

    ///get the local price of the token
    pub fn get_protocol_price (
        price: u64,
        premium: PreciseNumber

    ) -> PreciseNumber {
        let price = PreciseNumber::new(price as u128).expect("price");
        price.checked_mul(&premium).expect("a valid number")
    }

}

///compare two Pubkeys
pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

#[cfg(test)]
mod tests {
    use super::Token;
    use super::SwapState;
    use solana_program:: { 
        pubkey::Pubkey,
    };

    ///assume both tokens have the same price
    #[test]
    fn test_calc_swap_token() {
        let token_a = Token {
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            fee_vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
        };
        let token_a_price = 1;

        let token_b = Token {
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            fee_vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
        };
        let token_b_price = 1;

        let swap_state = SwapState{
            admin: Pubkey::new_unique(),
            bump: 0,
            is_initialized: true,
            swap_authority: Pubkey::new_unique(),
            swap_authority_bump: 0,
            swap_mint:Pubkey::new_unique(),
            token_a,
            token_b,
            program_fee: 100,
            swap_fee: 100,
        };

        let user_a_deposit_token_a = 10000000;
        let user_b_deposit_token_b = 10000000;
        let user_c_deposit_token_a = 10000000;

        let mut vault_a_supply: u64 = 100000000;
        let mut vault_b_supply: u64 = 100000000;
        let mut fee_vault_a: u64 = 0;
        let fee_vault_b: u64 = 0;
        let mut swap_supply: u64 = 200000000;

        let user_a_swap_tokens = swap_state.calculate_swap_tokens(
            user_a_deposit_token_a,
            vault_a_supply,
            token_a_price,
            fee_vault_a,
            vault_b_supply,
            token_b_price,
            fee_vault_b,
            swap_supply
        );

        //provides 10% of total protocol value, receives 10% of swap tokens
        assert!(user_a_swap_tokens==10000000); //aprox 5% of swap tokens 10000000/200000000
        swap_supply += user_a_swap_tokens; //210000000

        vault_a_supply += user_a_deposit_token_a; //110000000

        let user_b_swap_tokens = swap_state.calculate_swap_tokens(
            user_b_deposit_token_b,
            vault_b_supply,
            token_b_price,
            fee_vault_b,
            vault_a_supply,
            token_b_price,
            fee_vault_a,
            swap_supply
        );

        //token b is now in demand thus value of token b has increased
        //user b receives more swap tokens than user a due to this
        assert!(user_b_swap_tokens==11000000); // aprox 5.2 of swap tokens, 11000000/210000000
        
        swap_supply += user_b_swap_tokens; //221000000
        vault_b_supply += user_b_deposit_token_b; //110000000

        //a bunch of swaps are dome and there are now 10000000 tokens in the fee vault
        fee_vault_a += 100000000; 

        let user_c_swap_tokens = swap_state.calculate_swap_tokens(
            user_c_deposit_token_a,
            vault_a_supply,
            token_a_price,
            fee_vault_a,
            vault_b_supply,
            token_b_price,
            fee_vault_b,
            swap_supply
        );
        
        //user c deposit of 10000000 tokens now only accounts for 3.1% of total protocol value due to the tokens in the fee vault
        assert!(user_c_swap_tokens==6906250); //aprox  3.1% of swap tokens 6906250/221000000, 

    }

    ///assume both tokens have the same price
    #[test]
    fn test_calc_tokens_to_swap() {
        
        let token_a = Token {
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            fee_vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
        };
        let token_a_price = 1;

        let token_b = Token {
            mint: Pubkey::new_unique(),
            vault: Pubkey::new_unique(),
            fee_vault: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
        };
        let token_b_price = 1;

        let swap_state = SwapState{
            admin: Pubkey::new_unique(),
            bump: 0,
            is_initialized: true,
            swap_authority: Pubkey::new_unique(),
            swap_authority_bump: 0,
            swap_mint:Pubkey::new_unique(),
            token_a,
            token_b,
            program_fee: 100,
            swap_fee: 100,
        };

        let source_tokens:u64 = 1000000;
        let destination_tokens = SwapState::calculate_tokens_to_swap(
            swap_state.token_a.clone(),
            10000000,
            token_a_price,
            swap_state.token_b.clone(),
            token_b_price,
            10000000,
            source_tokens
        );
        assert!(destination_tokens==1000000);

        let destination_tokens = SwapState::calculate_tokens_to_swap(
            swap_state.token_a.clone(),
            10000000,
            token_a_price,
            swap_state.token_b.clone(),
            token_b_price, //this should cause the source token to be half the price, due to -50% premium
            5000000,
            1000000
        );

        //local market price for source token is -50% so you should get 50% back in destination tokens
        assert!(destination_tokens==source_tokens/2); 

    }
    

}