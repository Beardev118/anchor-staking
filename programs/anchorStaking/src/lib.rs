use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("71RRavF9hhoqRePEMEXYJPJoYeG6HSp1YqorBJMY7cLw");

#[program]
pub mod anchor_staking {
    // REPLACE ADDRESS of stake mint by running solana address -k .keys/stake_mint.json
    pub const STAKE_MINT_ADDRESS: &str = "342rhowqN3Up7uhm559bZMLs2NkaUGb3kjPG7SmnBoLV";
    // REPLACE ADDRESS of beef mint by running solana address -k .keys/beef_mint.json
    pub const BEEF_MINT_ADDRESS: &str = "3xmco6A5tPCXPUtRmVPNTbEtCfj3hmizqPmTF1Xz1xif";

    use super::*;

    pub fn stake(
        ctx: Context<Stake>,
        stake_mint_authority_bump: u8,
        program_beef_bag_bump: u8,
        beef_amount: u64,
    ) -> Result<()> {
        // ************
        // MINT STUFF PART
        // ************

        let stake_amount = beef_amount; // ???? FOR NOW!!!

        // We know that:
        //                                  findPDA(programId + seed)
        // stakeMintPDA, stakeMintPDABump = findPDA(programId + stakeMint.address)

        // -> So signer can be found using:
        // findPDA(programId + seed)              = X + bump
        // findPDA(programId + stakeMintAddress)  = X + bump
        let stake_mint_address = ctx.accounts.stake_mint.key();
        let seeds = &[stake_mint_address.as_ref(), &[stake_mint_authority_bump]];
        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.user_stake_token_bag.to_account_info(),
                authority: ctx.accounts.stake_mint_authority.to_account_info(),
            },
            &signer,
        );
        token::mint_to(cpi_ctx, stake_amount)?;

        // **********
        // Ask SPL Token Program to transfer $beef from the user
        // **********

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer{
                from: ctx.accounts.user_beef_token_bag.to_account_info(),
                authority: ctx.accounts.user_beef_token_bag_authority.to_account_info(),
                to: ctx.accounts.program_beef_token_bag.to_account_info(),
            }
        );

        token::transfer(cpi_ctx, beef_amount)?;

        Ok(())
    }

    pub fn create_beef_token_bag(ctx: Context<CreateBeefTokenBag>) -> Result<()> {
        Ok(())
    }

    pub fn unstake(ctx: Context<UnStake>, program_beef_bag_bump: u8, stake_amount: u64) -> Result<()> {
        // *********************
        // 1. Ask SPL Token Program to burn User's $stake
        // *********************

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.user_stake_token_bag.to_account_info(),
                authority: ctx.accounts.user_stake_token_bag_authority.to_account_info()
            },
        );

        token::burn(cpi_ctx, stake_amount)?;

        // ***********************
        // 2. Ask SPL Token Program to transfer back $beef to the user
        // ***********************

        // see why we did this in 'fn stake()'
        let beef_mint_address = ctx.accounts.beef_mint.key();
        let seeds = &[beef_mint_address.as_ref(), &[program_beef_bag_bump]];
        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.program_beef_token_bag.to_account_info(),
                authority: ctx.accounts.program_beef_token_bag.to_account_info(),
                to: ctx.accounts.user_beef_token_bag.to_account_info()
            },
            &signer
        );

        let beef_amount = stake_amount;
        token::transfer(cpi_ctx, beef_amount)?;
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(stake_mint_authority_bump: u8, program_beef_bag_bump: u8)]
pub struct Stake<'info> {
    // SPL Token Program
    pub token_program: Program<'info, Token>,

    // ***********
    // MINT
    // ***********

    // Address of the stake mint 🏭🥩
    #[account(
    mut,
    address = STAKE_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub stake_mint: Account<'info, Mint>,

    // The authority allowed to mutate the above ⬆️
    // And Print Stake Tokens
    /// CHECK: only used as a signing PDA
    #[account(
    seeds = [ stake_mint.key().as_ref() ],
    bump = stake_mint_authority_bump,
    )]
    pub stake_mint_authority: UncheckedAccount<'info>,

    // Associated Token Account 💰 for User to receive 🥩
    #[account(mut)]
    pub user_stake_token_bag: Account<'info, TokenAccount>,

    // ***************
    // TRANSFER $beef  FROM USERS
    // ***************

    // Associated Token Account for User which holds $beef
    #[account(mut)]
    pub user_beef_token_bag: Account<'info, TokenAccount>,

    // The authority allowed to mutate the above
    pub user_beef_token_bag_authority: Signer<'info>,

    // Used to receive $beef from Users
    #[account(
        mut, 
        seeds = [ beef_mint.key().as_ref()],
        bump = program_beef_bag_bump
    )]
    pub program_beef_token_bag: Account<'info, TokenAccount>,

    // Require for the PDA above
    #[account(
        address = BEEF_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub beef_mint: Box<Account<'info, Mint>>


}

#[derive(Accounts)]
pub struct CreateBeefTokenBag<'info> {
    // 1. PDA (so pubkey)
    // for the soon-to-be created beef token bag for our program
    #[account(
        init,
        payer = payer,

        // We use the token mint as a seed for the mapping
        // -> think "HashMap[seeds+bump] = pda"
        seeds = [ BEEF_MINT_ADDRESS.parse::<Pubkey>().unwrap().as_ref()],
        bump,

        // Token Program wants to know what kind of token this token bag is for 
        token::mint = beef_mint,

        // It's a PDA so the authority is itself!
        token::authority = program_beef_token_bag
    )]
    pub program_beef_token_bag: Account<'info, TokenAccount>,

    // 2. The mint $beef because it's needed from above .. token::mint =  ...
    #[account(address = BEEF_MINT_ADDRESS.parse::<Pubkey>().unwrap())]
    pub beef_mint: Account<'info, Mint>,

    // 3. The rent payer
    #[account(mut)]
    pub payer: Signer<'info>,

    // 4. Needed from Anchor for the creation of an Associated Token Account
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(program_beef_bag_bump: u8)]
pub struct UnStake<'info> {
    // SPL token program
    pub token_program: Program<'info, Token>,

    // ***********
    // Burning user's $stake
    // ***********

    // see 'token::Burn.mint
    #[account(
        mut,
        address = STAKE_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub stake_mint: Account<'info, Mint>,

    // see 'token::Burn.to
    #[account(mut)]
    pub user_stake_token_bag: Account<'info, TokenAccount>,

    // The authority allowed to mutate the above
    pub user_stake_token_bag_authority: Signer<'info>,

    // ****************
    // TRANSFER $beef TO USERS
    // ****************

    // see 'token::Transfer.from'
    #[account(
        mut, 
        seeds = [ beef_mint.key().as_ref()],
        bump = program_beef_bag_bump
    )]
    pub program_beef_token_bag: Account<'info, TokenAccount>,

    // see 'token::Transfer.to'
    #[account(mut)]
    pub user_beef_token_bag: Account<'info, TokenAccount>,

    // Require for the PDA above
    #[account(
        address = BEEF_MINT_ADDRESS.parse::<Pubkey>().unwrap()
    )]
    pub beef_mint: Box<Account<'info, Mint>>,
}