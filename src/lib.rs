use num::integer::Roots;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, address_lookup_table::instruction, entrypoint::{self, entrypoint, ProgramResult}, msg, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::Pubkey
};

use borsh::{BorshSerialize, BorshDeserialize};
use spl_token;
use solana_program::program_pack::Pack; // Import the Pack trait for unpacking accounts
use spl_token::instruction as token_instruction;

// Your LiquidityPool struct (unchanged, but note typo fix: token_a_reserve, token_b_reserve)
#[derive(BorshSerialize, BorshDeserialize)]
struct LiquidityPool {
    authority: Pubkey,
    token_a_mint: Pubkey,
    token_b_mint: Pubkey,
    token_a_vault: Pubkey,
    token_b_vault: Pubkey,
    liquidity_mint: Pubkey,
    liquidity_supply: u64,
    token_a_reserve: u64, // Was token_a_reserve in your code
    token_b_reserve: u64, // Was token_b_reserve in your code
}
#[derive(Debug)]
pub enum LiquidityPoolError{
    InvalidAccount,
    AlreadyInitialized,
    NotInitialized,
    InvalidAmount,
    InsufficientLiquidity,
    ArithmeticOverflow,
    InvalidTokenPair,
    Unauthorized,
}

impl From<LiquidityPoolError> for ProgramError {
    fn from(e: LiquidityPoolError) -> Self {
        msg!("Error: {:?}", e);
        ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
enum PoolInstruction {
    InitializePool,
    AddLiquidity { amount_a: u64, amount_b: u64 },
    RemoveLiquidity { liquidity_amount: u64 },
    Swap { amount_in: u64, a_to_b: bool },
}

entrypoint!(process_instruction);

fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
   let accounts_iter = &mut accounts.iter();

   let pool_state = next_account_info(accounts_iter)?;
   let authority = next_account_info(accounts_iter)?;
   let token_a_mint = next_account_info(accounts_iter)?;
   let token_b_mint = next_account_info(accounts_iter)?;
   let token_a_vault = next_account_info(accounts_iter)?;
   let token_b_vault = next_account_info(accounts_iter);
   let liquidity_supply = next_account_info(accounts_iter)?;
   let token_a_reserve = next_account_info(accounts_iter)?;
   let token_b_reserve = next_account_info(accounts_iter)?;

   let instruction = PoolInstruction::try_from_slice(instruction_data)?;

   match instruction {
    PoolInstruction::InitializePool => initialize_pool(program_id, accounts),
    PoolInstruction::AddLiquidity { amount_a, amount_b } => {
        add_liquidity(program_id, accounts, amount_a, amount_b)
    },
    PoolInstruction::RemoveLiquidity { liquidity_amount } => {
        remove_liquidty(program_id, accounts, liquidity_amount)
    },
    PoolInstruction::Swap { amount_in, a_to_b } => {
        swap(program_id, accounts, amount_in, a_to_b)
    }
    }?;


   fn initialize_pool(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Extract accounts
    let pool_state = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let token_a_mint = next_account_info(accounts_iter)?;
    let token_b_mint = next_account_info(accounts_iter)?;
    let token_a_vault = next_account_info(accounts_iter)?;
    let token_b_vault = next_account_info(accounts_iter)?;
    let liquidity_mint = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // --- Validation ---
    // 1. Pool state account: Must be writable and owned by the program
    if !pool_state.is_writable {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Ensures we can write pool state
    }
    if *pool_state.owner != *program_id && *pool_state.owner != solana_program::system_program::ID {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Ensures pool_state is owned by us
    }

    // 2. Authority: Should not be writable (just a reference, e.g., PDA)
    if authority.is_writable {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Prevents modifying authority
    }

    // 3. Token mints: Must be valid SPL token mints
    if *token_a_mint.owner != *token_program.key || *token_b_mint.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Verifies mints are SPL tokens
    }
    if token_a_mint.key == token_b_mint.key {
        return Err(LiquidityPoolError::InvalidAmount.into()); // Prevents same-token pools (e.g., SOL/SOL)
    }

    // 4. Token vaults: Must be valid SPL token accounts and writable
    if *token_a_vault.owner != *token_program.key || *token_b_vault.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Verifies vaults are SPL token accounts
    }
    if !token_a_vault.is_writable || !token_b_vault.is_writable {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Ensures vaults can receive tokens
    }

    // 5. Token program: Must be the official SPL Token program
    if *token_program.key != spl_token::id() {
        return Err(LiquidityPoolError::InvalidAccount.into()); // Prevents fake token programs
    }

    // --- Check if pool state is uninitialized ---
    if !pool_state.data_is_empty() {
        return Err(LiquidityPoolError::AlreadyInitialized.into()); // Ensures pool is fresh
    }

    // --- Set initial pool state ---
    let pool = LiquidityPool {
        authority: *authority.key,
        token_a_mint: *token_a_mint.key,
        token_b_mint: *token_b_mint.key,
        token_a_vault: *token_a_vault.key,
        token_b_vault: *token_b_vault.key,
        liquidity_mint: *liquidity_mint.key, 
        liquidity_supply: 0,
        token_a_reserve: 0,
        token_b_reserve: 0,
    };
    pool.serialize(&mut *pool_state.data.borrow_mut())?;


    Ok(())
}

fn add_liquidity(program_id: &Pubkey, accounts: &[AccountInfo], amount_a: u64, amount_b: u64) -> ProgramResult{
    let accounts_iter = &mut accounts.iter();

    let pool_state = next_account_info(accounts_iter)?;
    let user_token_a = next_account_info(accounts_iter)?;
    let user_token_b = next_account_info(accounts_iter)?;
    let token_a_vault = next_account_info(accounts_iter)?;
    let token_b_vault = next_account_info(accounts_iter)?;
    let liquidity_mint = next_account_info(accounts_iter)?;
    let user_liquidity = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;

    if !pool_state.is_writable {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if *pool_state.owner != *program_id {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user_token_a.is_writable || *user_token_a.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user_token_b.is_writable || *user_token_b.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !token_a_vault.is_writable || *token_a_vault.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !token_b_vault.is_writable || *token_b_vault.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !liquidity_mint.is_writable || *liquidity_mint.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user_liquidity.is_writable || *user_liquidity.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if *token_program.key != spl_token::id() {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user.is_signer {
        return Err(LiquidityPoolError::Unauthorized.into());
    }

    let mut pool = LiquidityPool::try_from_slice(&pool_state.data.borrow())?;

    if pool.token_a_vault != *token_a_vault.key || pool.token_b_vault != *token_b_vault.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    let user_token_a_mint = spl_token::state::Account::unpack(&user_token_a.data.borrow())?.mint;
    let user_token_b_mint = spl_token::state::Account::unpack(&user_token_b.data.borrow())?.mint;

    if pool.token_a_mint != user_token_a_mint || pool.token_b_mint != user_token_b_mint {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }

    if amount_a == 0 || amount_b == 0 {
        return Err(LiquidityPoolError::InvalidAmount.into())
    }

    let liquidity_to_mint = if pool.liquidity_supply == 0 {
        ((amount_a as u128) * (amount_b as u128)).sqrt() as u128
    } else {
        let liquidity_a = (amount_a as u128 * pool.liquidity_supply as u128) / pool.token_a_reserve as u128;
        let liquidity_b = (amount_b as u128 * pool.liquidity_supply as u128 ) / pool.token_b_reserve as u128;

        liquidity_a.min(liquidity_b) as u128
    };

    if liquidity_to_mint == 0 {
        return Err(LiquidityPoolError::InvalidAmount.into())
    }


    // transfer token a
    let transfer_instruction = token_instruction::transfer(
        token_program.key,
        user_token_a.key,
        token_a_vault.key,
        user.key,
        &[],
        amount_a,
    )?;
    invoke(&transfer_instruction, &[
        user_token_a.clone(),
        token_a_vault.clone(),
        user.clone(),
        token_program.clone(),
    ])?;

    // transfer token b


    invoke(&token_instruction::transfer(token_program.key, user_token_b.key, token_b_vault.key, user.key, &[], amount_b)?, &[
        user_token_b.clone(),
        token_b_vault.clone(),
        user.clone(),
        token_program.clone()
    ])?;

    //mint liquidity token

    invoke(&token_instruction::mint_to(token_program.key, liquidity_mint.key, user_liquidity.key, &pool.authority, &[], liquidity_to_mint as u64,)?,
     &[
        liquidity_mint.clone(),
        user_liquidity.clone(),
        pool_state.clone(),
        token_program.clone(),
     ]
    )?;

// updating pool reserve state

pool.token_a_reserve = pool.token_a_reserve.checked_add(amount_a).ok_or(ProgramError::ArithmeticOverflow)?;
pool.token_b_reserve = pool.token_b_reserve.checked_add(amount_b).ok_or(ProgramError::ArithmeticOverflow)?;
pool.liquidity_supply = pool.liquidity_supply.checked_add(liquidity_to_mint as u64).ok_or(ProgramError::ArithmeticOverflow)?;
pool.serialize(&mut pool_state.data.borrow_mut().as_mut())?;
    


    Ok(())
}

fn remove_liquidty(program_id: &Pubkey, accounts: &[AccountInfo], liquidity_amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let pool_state = next_account_info(account_iter)?;
    let user_liquidity = next_account_info(account_iter)?;
    let token_a_vault = next_account_info(account_iter)?;
    let token_b_vault = next_account_info(account_iter)?;
    let user_token_a = next_account_info(account_iter)?;
    let user_token_b = next_account_info(account_iter)?;
    let liquidity_mint = next_account_info(account_iter)?;
    let user = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;

    if !pool_state.is_writable || *pool_state.owner != *program_id {
        return Err(LiquidityPoolError::InvalidAccount.into())
    }

    if !user_liquidity.is_writable || *user_liquidity.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !token_a_vault.is_writable || *token_a_vault.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !token_b_vault.is_writable || *token_b_vault.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user_token_a.is_writable || *user_token_a.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user_token_b.is_writable || *user_token_b.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !liquidity_mint.is_writable || *liquidity_mint.owner != *token_program.key {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }

    if *token_program.key != spl_token::id() {
        return Err(LiquidityPoolError::InvalidAccount.into());
    }
    if !user.is_signer {
        return Err(LiquidityPoolError::Unauthorized.into());
    }

    let mut pool = LiquidityPool::try_from_slice(&pool_state.data.borrow())?;

    if liquidity_amount == 0 || liquidity_amount > pool.liquidity_supply {
        return  Err(LiquidityPoolError::InvalidAmount.into());
    }

    let  amount_a = (liquidity_amount as u128 * pool.token_a_reserve as u128 / pool.liquidity_supply as u128) as u64;
    let  amount_b = (liquidity_amount as u128 * pool.token_b_reserve as u128 / pool.liquidity_supply as u128) as u64;

    if amount_a == 0 || amount_b == 0 {
        return Err(LiquidityPoolError::InvalidAmount.into());
    }

    // burn liquidity
    invoke(&token_instruction::burn(token_program.key, user_liquidity.key, liquidity_mint.key, user.key, &[], liquidity_amount)?, &[
        user_liquidity.clone(),
        liquidity_mint.clone(),
        user.clone(),
        token_program.clone(),
    ])?;

    invoke_signed(
        &token_instruction::transfer(
            token_program.key,
            token_a_vault.key,
            user_token_a.key,
            &pool.authority,
            &[],
            amount_a,
        )?,
        &[
            token_a_vault.clone(),
            user_token_a.clone(),
            pool_state.clone(),
            token_program.clone(),
        ],
        &[&[/* PDA seeds for authority */]],
    )?;
    
    invoke_signed(
        &token_instruction::transfer(
            token_program.key,
            token_b_vault.key,
            user_token_b.key,
            &pool.authority,
            &[],
            amount_b,
        )?,
        &[
            token_b_vault.clone(),
            user_token_b.clone(),
            pool_state.clone(),
            token_program.clone(),
        ],
        &[&[/* PDA seeds for authority */]],
    )?;

    // Continuing in remove_liquidity
    pool.token_a_reserve = pool.token_a_reserve.checked_sub(amount_a).ok_or(ProgramError::ArithmeticOverflow)?;
    pool.token_b_reserve = pool.token_b_reserve.checked_sub(amount_b).ok_or(ProgramError::ArithmeticOverflow)?;
    pool.liquidity_supply = pool.liquidity_supply.checked_sub(liquidity_amount).ok_or(ProgramError::ArithmeticOverflow)?;
    pool.serialize(&mut *pool_state.data.borrow_mut())?;


    Ok(())
}

fn swap(program_id: &Pubkey, accounts: &[AccountInfo], amount_in: u64, a_to_b: bool) -> ProgramResult {

    let account_iter =&mut accounts.iter();

    let pool_state = next_account_info(account_iter)?;
    let user_input_token = next_account_info(account_iter)?;
    let user_output_token = next_account_info(account_iter)?;
    let input_vault = next_account_info(account_iter)?;
    let output_vault = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let user = next_account_info(account_iter)?;

    if !pool_state.is_writable || *pool_state.owner != *program_id {
        return Err(LiquidityPoolError::InvalidAccount.into())
    } 

    if !user_input_token.is_writable || *user_input_token.owner != *token_program.key{
        return Err(LiquidityPoolError::InvalidAccount.into())
    }

    if !user_output_token.is_writable || *user_output_token.owner != *token_program.key{
        return Err(LiquidityPoolError::InvalidAccount.into())
    }

    if !input_vault.is_writable || *input_vault.owner != *token_program.key{
        return Err(LiquidityPoolError::InvalidAccount.into())
    }

    if !output_vault.is_writable || *output_vault.owner != *token_program.key{
        return Err(LiquidityPoolError::InvalidAccount.into())
    }

    if *token_program.key != spl_token::ID{
        return Err(LiquidityPoolError::InvalidAccount.into());
    }

    if !user.is_signer{
        return Err(LiquidityPoolError::Unauthorized.into())
    }

    // validating the pool state
    let mut pool = LiquidityPool::try_from_slice(*pool_state.data.borrow())?;

    if a_to_b {
        if  pool.token_a_vault != *input_vault.key || pool.token_b_vault != *output_vault.key {
            return Err(LiquidityPoolError::InvalidAccount.into());
        }

        if pool.token_a_mint != *user_input_token.key || pool.token_b_mint != *user_output_token.key{
            return Err(LiquidityPoolError::InvalidAccount.into());
        }
    } else {
        if pool.token_b_vault != *input_vault.key || pool.token_a_vault != *output_vault.key {
            return Err(LiquidityPoolError::InvalidAccount.into());
        } 

        if pool.token_b_mint != *user_input_token.key || pool.token_a_mint != *user_output_token.key{
            return Err(LiquidityPoolError::InvalidAccount.into());
        }
    }

    if amount_in == 0 {
        return Err(LiquidityPoolError::InvalidAmount.into());
    }

    let fee_enumerator = 30;
    let fee_denumerator = 10000;

    let (input_reserve, output_reserve) = if a_to_b {
        (pool.token_a_reserve, pool.token_b_reserve)  
    } else {
        (pool.token_b_reserve, pool.token_a_reserve)
    };

    let amount_in_after_fee = amount_in.checked_mul(fee_denumerator - fee_enumerator)
    .ok_or(ProgramError::ArithmeticOverflow)?
    .checked_div(fee_denumerator)
    .ok_or(ProgramError::ArithmeticOverflow)?;

    let invariant = (input_reserve as u128) * (output_reserve as u128);

    let new_input_reserve = (input_reserve as u128) * (amount_in_after_fee as u128);

    let amount_out= output_reserve.checked_sub((invariant / new_input_reserve)
    .try_into()
    .map_err(|_| ProgramError::ArithmeticOverflow)?, )
    .ok_or(ProgramError::ArithmeticOverflow)?; 

    if amount_out == 0 {
        return Err(LiquidityPoolError::InvalidAmount.into())
    }
    
    // Transfer Input token
    let _ = invoke_signed(&token_instruction::transfer(
        token_program.key, 
        user_input_token.key, 
        input_vault.key, 
        user.key, 
        &[], 
         amount_in)?,
     &[user_input_token.clone(), input_vault.clone(), user.clone(), token_program.clone()],  
     &[&[/* PDA seeds for authority */]],);

    //  transfer output token
    invoke_signed(
        &token_instruction::transfer(
            token_program.key,
            output_vault.key,
            user_output_token.key,
            &pool.authority,
            &[],
            amount_out,
        )?,
        &[
            output_vault.clone(),
            user_output_token.clone(),
            pool_state.clone(),
            token_program.clone(),
        ],
        &[&[/* PDA seeds for authority */]],
    )?;

   if a_to_b {
    pool.token_a_reserve = pool.token_a_reserve.checked_add(amount_in).ok_or(ProgramError::ArithmeticOverflow)?;
    pool.token_b_reserve = pool.token_b_reserve.checked_sub(amount_out).ok_or(ProgramError::ArithmeticOverflow)?;
   } else {
       pool.token_b_reserve = pool.token_b_reserve.checked_add(amount_in).ok_or(ProgramError::ArithmeticOverflow)?;
       pool.token_a_reserve = pool.token_a_reserve.checked_sub(amount_out).ok_or(ProgramError::ArithmeticOverflow)?;
   }
    pool.serialize(&mut *pool_state.data.borrow_mut())?;
    

Ok(())
}
   
Ok(())
}
