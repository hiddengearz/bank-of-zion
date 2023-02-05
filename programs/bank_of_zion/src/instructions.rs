use {
    crate::{
        error::ZionError,
        state::SwapState
        
    },
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvar,
        program_pack::Pack,
    },
    std::{
        mem::size_of,
    },
    arrayref::{array_ref, array_refs}
};


///Enum listing all of the instructions for the program
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum ZionInstruction {
    ///Initialize the swap pool
    Initialize (Initialize),

    ///Deposit initial liquidity for pools
    AdminDeposit(AdminDeposit),

    ///Deposit liquidity into pools
    Deposit(Deposit),

    ///Withdraw liquidity from pools
    Withdraw(Withdraw),

    ///Swap tokens
    Swap(Swap),

    ///Close pool
    ClosePool()
}

/// Initialize instruction data
#[repr(C)]
#[derive(Clone,Debug, PartialEq)]
pub struct Initialize {
    /// all swap fees
    pub swap_state: SwapState,
}
///Admin to deposit initial liquidity
#[repr(C)]
#[derive(Clone,Debug, PartialEq)]
pub struct AdminDeposit {
    /// tokens for pool a
    pub token_a_deposit: u64,

    /// tokens for pool b
    pub token_b_deposit: u64,
}
impl AdminDeposit { 
    ///length of AdminDeposit struct
    pub const LEN: usize = 16;
}

///Users deposit liquidity
#[repr(C)]
#[derive(Clone,Debug, PartialEq)]
pub struct Deposit {
    /// tokens for pool a
    pub token_a_deposit: u64,

    /// tokens for pool b
    pub token_b_deposit: u64,
}
impl Deposit { 
    ///length of Deposit struct
    pub const LEN: usize = 16;
}

///Users deposit liquidity
#[repr(C)]
#[derive(Clone,Debug, PartialEq)]
pub struct Withdraw {
    /// tokens for pool a
    pub token_a_withdraw: u64,

    /// tokens for pool b
    pub token_b_withdraw: u64,
}
impl Withdraw { 
    ///length of Deposit struct
    pub const LEN: usize = 16;
}

///Swap a token from one pool to the other
#[repr(C)]
#[derive(Clone,Debug, PartialEq)]
pub struct Swap {
    /// tokens for pool a
    pub amount: u64,
}
impl Swap { 
    ///length of Deposit struct
    pub const LEN: usize = 8;
}

impl ZionInstruction {
    /// Unpacks a byte buffer into a [ZionInstruction](enum.ZionInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use ZionError::InvalidInstruction;
        
        if input.len() == 0 {
            Ok(ZionInstruction::ClosePool())
        } else {
            let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
            
            Ok(match tag {
                0 => {
                    let swap_state = SwapState::unpack_from_slice(rest)?;
                    Self::Initialize (
                        Initialize { swap_state}
                    )
                },
                1 => {
                    let input = array_ref![rest, 0, AdminDeposit::LEN];
                    
                    let (
                        token_a_deposit,
                        token_b_deposit,
                    ) = array_refs![input, 8, 8];

                    Self::AdminDeposit (
                        AdminDeposit { 
                            token_a_deposit: u64::from_le_bytes(*token_a_deposit),
                            token_b_deposit: u64::from_le_bytes(*token_b_deposit)
                        }

                    )
                },
                2 => {
                    let data = array_ref![rest, 0, Deposit::LEN];
                    
                    let (
                        token_a_deposit,
                        token_b_deposit,
                    ) = array_refs![data, 8, 8];

                    Self::Deposit (
                        Deposit { 
                            token_a_deposit: u64::from_le_bytes(*token_a_deposit),
                            token_b_deposit: u64::from_le_bytes(*token_b_deposit)
                        }

                    )
                },
                3 => {
                    let data = array_ref![rest, 0, Withdraw::LEN];
                    
                    let (
                        token_a_withdraw,
                        token_b_withdraw,
                    ) = array_refs![data, 8, 8];

                    Self::Withdraw (
                        Withdraw { 
                            token_a_withdraw: u64::from_le_bytes(*token_a_withdraw),
                            token_b_withdraw: u64::from_le_bytes(*token_b_withdraw)
                        }

                    )
                },
                4 => {
                    let data = array_ref![rest, 0, Swap::LEN]; 

                    Self::Swap (
                        Swap { 
                            amount: u64::from_le_bytes(*data),
                        }

                    )
                },
                _ => return Err(ZionError::InvalidInstruction.into()),

            })
        }
    }

    /// Packs a [ZionInstruction](enum.ZionInstruction.html) into a byte buffer.    
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match &*self {
            Self::Initialize(Initialize { swap_state }) => {
                buf.push(0);
                let mut state_slice = [0u8; SwapState::LEN];
                Pack::pack_into_slice(swap_state, &mut state_slice[..]);
                buf.extend_from_slice(&state_slice);
            },
            Self::AdminDeposit( AdminDeposit {token_a_deposit, token_b_deposit}) => {
                buf.push(1);
                buf.extend_from_slice(&token_a_deposit.to_le_bytes());
                buf.extend_from_slice(&token_b_deposit.to_le_bytes());
            },
            Self::Deposit( Deposit {token_a_deposit, token_b_deposit}) => {
                buf.push(2);
                buf.extend_from_slice(&token_a_deposit.to_le_bytes());
                buf.extend_from_slice(&token_b_deposit.to_le_bytes());
            },
            Self::Withdraw( Withdraw {token_a_withdraw, token_b_withdraw}) => {
                buf.push(3);
                buf.extend_from_slice(&token_a_withdraw.to_le_bytes());
                buf.extend_from_slice(&token_b_withdraw.to_le_bytes());
            },
            Self::Swap( Swap {amount}) => {
                buf.push(4);
                buf.extend_from_slice(&amount.to_le_bytes());
            },
            Self::ClosePool() => {}
        }
        buf
    }
}

/// Creates an 'initialize' instruction.
pub fn initialize(
    swap_state: SwapState,
    swap_state_pubkey: &Pubkey
) -> Instruction {
    
    let accounts = vec![
        AccountMeta::new(swap_state.admin, true),
        AccountMeta::new(swap_state.swap_authority, false),
        AccountMeta::new_readonly(swap_state.swap_mint, false),
        AccountMeta::new(*swap_state_pubkey, false),

        AccountMeta::new_readonly(swap_state.token_a.mint, false),
        AccountMeta::new_readonly(swap_state.token_a.vault, false),
        AccountMeta::new_readonly(swap_state.token_a.fee_vault, false),
        AccountMeta::new_readonly(swap_state.token_a.oracle, false),

        AccountMeta::new_readonly(swap_state.token_b.mint, false),
        AccountMeta::new_readonly(swap_state.token_b.vault, false),
        AccountMeta::new_readonly(swap_state.token_b.fee_vault, false),
        AccountMeta::new_readonly(swap_state.token_b.oracle, false),
        
        AccountMeta::new_readonly(spl_token::ID, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
    ];

    let init_data = ZionInstruction::Initialize(Initialize { swap_state });
    let data = init_data.pack();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

/// Creates an 'admin_deposit' instruction.
pub fn admin_deposit(
    admin_pubkey: &Pubkey,
    swap_authority_pubkey: &Pubkey,
    swap_mint_pubkey: &Pubkey,
    swap_state_pubkey: &Pubkey,
    admin_swap_wallet_pubkey: &Pubkey,
    token_a_mint_pubkey: &Pubkey,
    token_a_admin_pubkey: &Pubkey,
    token_a_vault_pubkey: &Pubkey,
    token_a_oracle_pubkey: &Pubkey,
    token_b_mint_pubkey: &Pubkey,
    token_b_admin_pubkey: &Pubkey,
    token_b_vault_pubkey: &Pubkey,
    token_b_oracle_pubkey: &Pubkey,
    token_a_deposit: u64,
    token_b_deposit: u64,
) -> Instruction {
    
    let accounts = vec![
        AccountMeta::new_readonly(*admin_pubkey, true),
        AccountMeta::new_readonly(*swap_authority_pubkey, false),
        AccountMeta::new(*swap_mint_pubkey, false),
        AccountMeta::new_readonly(*swap_state_pubkey, false),
        AccountMeta::new(*admin_swap_wallet_pubkey, false),

        AccountMeta::new(*token_a_mint_pubkey, false),
        AccountMeta::new(*token_a_admin_pubkey, false),
        AccountMeta::new(*token_a_vault_pubkey, false),
        AccountMeta::new_readonly(*token_a_oracle_pubkey, false),

        AccountMeta::new(*token_b_mint_pubkey, false),
        AccountMeta::new(*token_b_admin_pubkey, false),
        AccountMeta::new(*token_b_vault_pubkey, false),
        AccountMeta::new_readonly(*token_b_oracle_pubkey, false),

        AccountMeta::new_readonly(spl_token::ID, false),
    ];

    let init_data = ZionInstruction::AdminDeposit(AdminDeposit { token_a_deposit, token_b_deposit });
    let data = init_data.pack();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

/// Creates an 'deposit' instruction.
pub fn deposit(
    user_pubkey: &Pubkey,
    swap_state_pubkey: &Pubkey,
    swap_authority_pubkey: &Pubkey,
    swap_mint_pubkey: &Pubkey,
    user_swap_wallet_pubkey: &Pubkey,

    token_a_user_pubkey: &Pubkey,
    token_a_vault_pubkey: &Pubkey,
    token_a_fee_vault: &Pubkey,
    token_a_oracle_pubkey: &Pubkey,

    token_b_user_pubkey: &Pubkey,
    token_b_vault_pubkey: &Pubkey,
    token_b_fee_vault: &Pubkey,
    token_b_oracle_pubkey: &Pubkey,

    token_a_deposit: u64,
    token_b_deposit: u64,
) -> Instruction {
    
    let accounts = vec![
        AccountMeta::new_readonly(*user_pubkey, true),
        AccountMeta::new_readonly(*swap_state_pubkey, false),
        AccountMeta::new_readonly(*swap_authority_pubkey, false),
        AccountMeta::new(*swap_mint_pubkey, false),
        AccountMeta::new(*user_swap_wallet_pubkey, false),

        AccountMeta::new(*token_a_user_pubkey, false),
        AccountMeta::new(*token_a_vault_pubkey, false),
        AccountMeta::new(*token_a_fee_vault, false),
        AccountMeta::new_readonly(*token_a_oracle_pubkey, false),

        AccountMeta::new(*token_b_user_pubkey, false),
        AccountMeta::new(*token_b_vault_pubkey, false),
        AccountMeta::new(*token_b_fee_vault, false),
        AccountMeta::new_readonly(*token_b_oracle_pubkey, false),

        AccountMeta::new_readonly(spl_token::ID, false),
    ];

    let init_data = ZionInstruction::Deposit(Deposit { token_a_deposit, token_b_deposit });
    let data = init_data.pack();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}


/// Creates an 'withdraw' instruction.
pub fn withdraw(
    user_pubkey: &Pubkey,
    swap_state_pubkey: &Pubkey,
    swap_authority_pubkey: &Pubkey,
    swap_mint_pubkey: &Pubkey,
    user_swap_wallet_pubkey: &Pubkey,

    token_a_user_pubkey: &Pubkey,
    token_a_vault_pubkey: &Pubkey,
    token_a_fee_vault: &Pubkey,
    token_a_oracle_pubkey: &Pubkey,

    token_b_user_pubkey: &Pubkey,
    token_b_vault_pubkey: &Pubkey,
    token_b_fee_vault: &Pubkey,
    token_b_oracle_pubkey: &Pubkey,

    token_a_withdraw: u64,
    token_b_withdraw: u64,
) -> Instruction {
    
    let accounts = vec![
        AccountMeta::new_readonly(*user_pubkey, true),
        AccountMeta::new_readonly(*swap_state_pubkey, false),
        AccountMeta::new_readonly(*swap_authority_pubkey, false),
        AccountMeta::new(*swap_mint_pubkey, false),
        AccountMeta::new(*user_swap_wallet_pubkey, false),

        AccountMeta::new(*token_a_user_pubkey, false),
        AccountMeta::new(*token_a_vault_pubkey, false),
        AccountMeta::new(*token_a_fee_vault, false),
        AccountMeta::new_readonly(*token_a_oracle_pubkey, false),

        AccountMeta::new(*token_b_user_pubkey, false),
        AccountMeta::new(*token_b_vault_pubkey, false),
        AccountMeta::new(*token_b_fee_vault, false),
        AccountMeta::new_readonly(*token_b_oracle_pubkey, false),

        AccountMeta::new_readonly(spl_token::ID, false),
    ];

    let init_data = ZionInstruction::Withdraw(Withdraw { token_a_withdraw, token_b_withdraw });
    let data = init_data.pack();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

/// Creates an 'swap' instruction.
pub fn swap(
    user_pubkey: &Pubkey,
    swap_state_pubkey: &Pubkey,
    swap_authority_pubkey: &Pubkey,
    
    source_user_pubkey: &Pubkey,
    source_vault_pubkey: &Pubkey,
    source_fee_vault: &Pubkey,
    source_oracle_pubkey: &Pubkey,

    destination_user_pubkey: &Pubkey,
    destination_vault_pubkey: &Pubkey,
    destination_fee_vault: &Pubkey,
    destination_oracle_pubkey: &Pubkey,

    amount: u64,
) -> Instruction {
    
    let accounts = vec![
        AccountMeta::new_readonly(*user_pubkey, true),
        AccountMeta::new_readonly(*swap_state_pubkey, false),
        AccountMeta::new_readonly(*swap_authority_pubkey, false),
        
        AccountMeta::new(*source_user_pubkey, false),
        AccountMeta::new(*source_vault_pubkey, false),
        AccountMeta::new(*source_fee_vault, false),
        AccountMeta::new_readonly(*source_oracle_pubkey, false),

        AccountMeta::new(*destination_user_pubkey, false),
        AccountMeta::new(*destination_vault_pubkey, false),
        AccountMeta::new(*destination_fee_vault, false),
        AccountMeta::new_readonly(*destination_oracle_pubkey, false),

        AccountMeta::new_readonly(spl_token::ID, false),
    ];

    let init_data = ZionInstruction::Swap(Swap { amount });
    let data = init_data.pack();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}


/// Creates an 'close_pool' instruction.
pub fn close_pool(
    admin_pubkey: &Pubkey,
    swap_state_pubkey: &Pubkey,
    swap_authority_pubkey: &Pubkey,
) -> Instruction {
    
    let accounts = vec![
        AccountMeta::new(*admin_pubkey, true),
        AccountMeta::new(*swap_state_pubkey, false),
        AccountMeta::new_readonly(*swap_authority_pubkey, false),
    ];

    let init_data = ZionInstruction::ClosePool();
    let data = init_data.pack();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}