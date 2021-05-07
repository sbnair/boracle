use anchor_lang::prelude::*;
use aggregator::aggregator::{Aggregator};
use aggregator::{WriteData};
#[program]
pub mod node {
    use super::*;

    #[state]
    pub struct Node {
        authority: Pubkey,
        nonce: u8,
        node_signer: Pubkey,
        node: Pubkey,
        last_report_epoch: i64
    }

    impl Node {
        pub fn new(ctx: Context<Auth>, nonce: u8) -> Result<Self> {
            Ok(Self {
                authority: *ctx.accounts.authority.key,
                node: *ctx.accounts.node.key,
                node_signer: *ctx.accounts.node_signer.key,
                nonce,
                last_report_epoch: 0
            })
        }

        pub fn write_to_aggregator(&mut self, ctx: Context<AuthWriteToAggregator>, data: u64) -> ProgramResult {
            if &self.authority != ctx.accounts.authority.key {
                return Err(ErrorCode::Unauthorized.into());
            }
            let cpi_program = ctx.accounts.aggregator_program.clone();
            let cpi_accounts = WriteData {
                node_signer: ctx.accounts.node_signer.clone().into(),
            };

            let seeds = &[
                ctx.accounts.node.to_account_info().key.as_ref(),
                &[self.nonce],
            ];
            let signer = &[&seeds[..]];
            let ctx = ctx.accounts.cpi_state.context(cpi_program, cpi_accounts);
            aggregator::cpi::state::write_data(ctx.with_signer(signer), data);
            self.last_report_epoch = anchor_lang::prelude::Clock::get().expect("incorrect").epoch_start_timestamp;
            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct Auth<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    node: AccountInfo<'info>,
    node_signer: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct AuthWriteToAggregator<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
    node: AccountInfo<'info>,
    node_signer: AccountInfo<'info>,
    #[account(mut, state = aggregator_program)]
    cpi_state: CpiState<'info, Aggregator>,
    #[account(executable)]
    aggregator_program: AccountInfo<'info>,
}


#[error]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
