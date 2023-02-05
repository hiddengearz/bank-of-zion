
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::{ invoke_signed, invoke}, pubkey::Pubkey,
    rent::Rent, sysvar::Sysvar, system_instruction
};

///CPI to system program to create an account
pub fn create_account<'a>(
    program_id: &Pubkey,
    system_program: &AccountInfo<'a>,
    fee_payer: &AccountInfo<'a>,
    account_to_create: &AccountInfo<'a>,
    signer_seeds: &[&[&[u8]]],
    space: usize,
) -> ProgramResult {

    invoke_signed(
        &system_instruction::create_account(
            fee_payer.key,
            account_to_create.key,
            Rent::get()?.minimum_balance(space),
            space as u64,
            program_id,
        ),
        &[

            fee_payer.clone(),
            account_to_create.clone(),
            system_program.clone(),
        ],
        signer_seeds,
    )
}

///CPI to system program to create an account on a program-derived address
pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    rent: &Rent,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> ProgramResult {
    if new_pda_account.lamports() > 0 {
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(new_pda_account.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[
                    payer.clone(),
                    new_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(new_pda_account.key, owner),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )
    } else {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )
    }
}

///CPI to spl_token program to issue a spl_token `Burn` instruction.
pub fn token_burn<'a>(
    token_program: &AccountInfo<'a>,
    wallet: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &spl_token::instruction::burn(
            token_program.key,
            wallet.key,
            mint.key,
            authority.key,
            &[authority.key],
            amount,
        )?,
        &[mint.clone(), wallet.clone(), authority.clone(), token_program.clone()],
        &[signer_seeds],
    )
}

///CPI to spl_token program to issue a spl_token `Mint_To` instruction.
pub fn token_mint_to<'a>(
    token_program: &AccountInfo<'a>,
    mint: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &spl_token::instruction::mint_to(
            token_program.key,
            mint.key,
            destination.key,
            authority.key,
            &[authority.key],
            amount,
        )?,
        &[mint.clone(), destination.clone(), authority.clone(), token_program.clone()],
        &[signer_seeds],
    )
}
///CPI to spl_token program to issue a spl_token `Transfer` instruction.
pub fn token_transfer<'a>(
    token_program: &AccountInfo<'a>,
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[authority.key],
            amount,
        )?,
        &[source.clone(), destination.clone(), authority.clone(), token_program.clone()],
    )
}

///CPI to spl_token program to issue a spl_token `Transfer` instruction.
pub fn token_transfer_signed<'a>(
    token_program: &AccountInfo<'a>,
    source: &AccountInfo<'a>,
    destination: &AccountInfo<'a>,
    authority: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> ProgramResult {
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[authority.key],
            amount,
        )?,
        &[source.clone(), destination.clone(), authority.clone(), token_program.clone()],
        &[signer_seeds],
    )
}

