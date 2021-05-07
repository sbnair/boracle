// #region code
use anchor_lang::prelude::*;

#[program]
pub mod aggregator {
    use super::*;

    #[state]
    pub struct Aggregator {
        authority: Pubkey,
        number_of_nodes: u8,
        node_signers : Vec<Pubkey>,
        node_data: Vec<u64>,
        answer: u64,
    }

    impl Aggregator {
        pub fn new(ctx: Context<Auth>) -> Result<Self> {
            Ok(Self {
                authority: *ctx.accounts.authority.key,
                number_of_nodes: 0,
                node_signers: vec![*ctx.accounts.authority.key; 32],
                node_data: vec![0; 32],
                answer: 0
            })
        }
        pub fn add_node(&mut self, ctx: Context<Auth>, new_node: Pubkey) -> Result<()> {
            if &self.authority != ctx.accounts.authority.key {
                return Err(ErrorCode::Unauthorized.into());
            }

            for i in 0..self.node_signers.len() {
                if self.node_signers[i] == self.authority {
                    self.node_signers[i] = new_node;
                    self.number_of_nodes += 1;
                    return Ok(());
                }
            }
            Err(ErrorCode::NodesFull.into())
        }
        pub fn remove_node(&mut self, ctx: Context<Auth>, old_node: Pubkey) -> Result<()> {
            if &self.authority != ctx.accounts.authority.key {
                return Err(ErrorCode::Unauthorized.into());
            }

            for i in 0..self.node_signers.len() {
                if self.node_signers[i] == old_node {
                    self.node_signers[i] = self.authority;
                    self.number_of_nodes -= 1;
                    return Ok(());
                }
            }
            Err(ErrorCode::NodesDoesNotExist.into())
        }

        pub fn write_data(&mut self, ctx: Context<WriteData>, data: u64) -> Result<()> {

            for i in 0..self.node_signers.len() {
                if self.node_signers[i] == *ctx.accounts.node_signer.key {
                    self.node_data[i] = data;
                    // update answer
                    let mut ready_nodes = vec![];
                    for i in 0..self.node_signers.len() {
                        if self.node_data[i] > 0 {
                            ready_nodes.push(self.node_data[i])
                        }
                    }

                    // update answer if more than two nodes reported data. use their median.
                    if ready_nodes.len() >= 2 {
                        let mid = ready_nodes.len() / 2;
                        ready_nodes.sort();
                        self.answer = ready_nodes[mid];

                        // answer updated. reset all nodes data until they write again.
                        self.node_data = vec![0; 32];
                    }
                    return Ok(());
                }
            }
            Err(ErrorCode::NodesDoesNotExist.into())
        }
    }
}

#[derive(Accounts)]
pub struct Auth<'info> {
    #[account(signer)]
    authority: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WriteData<'info> {
    #[account(signer)]
    pub node_signer: AccountInfo<'info>,
}


#[error]
pub enum ErrorCode {
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
    #[msg("Nodes are full.")]
    NodesFull,
    #[msg("Unable to find node.")]
    NodesDoesNotExist,
}
