use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};

declare_id!("EqbbJ4zuquGXzwarPqCiNdFjA2i6rRnrWJte9hRustBk");

#[program]
pub mod network_manager {
    use super::*;

    pub fn create_relayer(ctx: Context<CreateRelayer>, connection_url: String) -> Result<()> {
        let relayer_base = &mut ctx.accounts.relayer_base;
        let service_base = &mut ctx.accounts.service_base;
        let relayer = &mut ctx.accounts.relayer;

        relayer_base.connection_url = connection_url;
        relayer_base.relayer = relayer.key();
        relayer_base.locked_fee = 2_000_000_000; // 2 SOL in lamports

        service_base.service_counter = 0;

        // Transfer 2 SOL as a fee
        let ix = system_instruction::transfer(
            &relayer.key(),
            &relayer_base.to_account_info().key(),
            2_000_000_000, // 2 SOL in lamports
        );
        invoke(
            &ix,
            &[
                relayer.to_account_info(),
                relayer_base.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn update_relayer(ctx: Context<UpdateRelayer>, new_connection_url: String) -> Result<()> {
        let relayer_base = &mut ctx.accounts.relayer_base;
        relayer_base.connection_url = new_connection_url;
        Ok(())
    }

    pub fn create_service(ctx: Context<CreateService>, service_data: ServiceData) -> Result<()> {
        let service_base = &mut ctx.accounts.service_base;
        let service = &mut ctx.accounts.service;

        service.service_url = service_data.service_url;
        service.service_description = service_data.service_description;
        service.service_face_url = service_data.service_face_url;
        service.service_fee = service_data.service_fee;
        service.relayer_share = service_data.relayer_share;

        service_base.service_counter += 1;

        Ok(())
    }

    pub fn update_service(
        ctx: Context<UpdateService>,
        service_count: u64,
        service_data: ServiceData,
    ) -> Result<()> {
        let service = &mut ctx.accounts.service;

        service.service_url = service_data.service_url;
        service.service_description = service_data.service_description;
        service.service_face_url = service_data.service_face_url;
        service.service_fee = service_data.service_fee;
        service.relayer_share = service_data.relayer_share;

        Ok(())
    }

    pub fn close_relayer(ctx: Context<CloseRelayer>) -> Result<()> {
        let relayer_base = &ctx.accounts.relayer_base;
        let relayer = &ctx.accounts.relayer;

        // Transfer the locked fee back to the relayer
        **relayer_base.to_account_info().try_borrow_mut_lamports()? -= relayer_base.locked_fee;
        **relayer.try_borrow_mut_lamports()? += relayer_base.locked_fee;

        // The framework will automatically close the accounts marked with Close

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateRelayer<'info> {
    #[account(
        init,
        payer = relayer,
        space = 8 + 32 + 200 + 8, // Discriminator + pubkey + max url length + locked fee
        seeds = [relayer.key().as_ref(), b"relayerbase"],
        bump
    )]
    pub relayer_base: Account<'info, RelayerBase>,

    #[account(
        init,
        payer = relayer,
        space = 8 + 8, // Discriminator + u64
        seeds = [relayer.key().as_ref(), b"servicebase"],
        bump
    )]
    pub service_base: Account<'info, ServiceBase>,

    #[account(mut)]
    pub relayer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateRelayer<'info> {
    #[account(
        mut,
        seeds = [relayer.key().as_ref(), b"relayerbase"],
        bump,
        has_one = relayer
    )]
    pub relayer_base: Account<'info, RelayerBase>,

    pub relayer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateService<'info> {
    #[account(
        mut,
        seeds = [relayer.key().as_ref(), b"servicebase"],
        bump
    )]
    pub service_base: Account<'info, ServiceBase>,

    #[account(
        init,
        payer = relayer,
        space = 8 + 32 + 200 + 200 + 200 + 8 + 1, // Discriminator + fields
        seeds = [relayer.key().as_ref(), &service_base.service_counter.to_le_bytes(), b"service"],
        bump
    )]
    pub service: Account<'info, Service>,

    #[account(mut)]
    pub relayer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(service_count: u64)]
pub struct UpdateService<'info> {
    #[account(
        seeds = [relayer.key().as_ref(), b"servicebase"],
        bump
    )]
    pub service_base: Account<'info, ServiceBase>,

    #[account(
        mut,
        seeds = [relayer.key().as_ref(), &service_count.to_le_bytes(), b"service"],
        bump
    )]
    pub service: Account<'info, Service>,

    pub relayer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseRelayer<'info> {
    #[account(
        mut,
        close = relayer,
        seeds = [relayer.key().as_ref(), b"relayerbase"],
        bump,
        has_one = relayer
    )]
    pub relayer_base: Account<'info, RelayerBase>,

    #[account(
        mut,
        close = relayer,
        seeds = [relayer.key().as_ref(), b"servicebase"],
        bump
    )]
    pub service_base: Account<'info, ServiceBase>,

    #[account(mut)]
    pub relayer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct RelayerBase {
    pub connection_url: String,
    pub relayer: Pubkey,
    pub locked_fee: u64,
}

#[account]
pub struct ServiceBase {
    pub service_counter: u64,
}

#[account]
pub struct Service {
    pub service_url: String,
    pub service_description: String,
    pub service_face_url: String,
    pub service_fee: u64,
    pub relayer_share: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ServiceData {
    pub service_url: String,
    pub service_description: String,
    pub service_face_url: String,
    pub service_fee: u64,
    pub relayer_share: u8,
}
