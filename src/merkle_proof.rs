/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::{
    hashmapaug::HashmapAugType,
    merkle_update::MerkleUpdate,
    Serializable, Deserializable, GetRepresentationHash,
    accounts::Account,
    shard::ShardStateUnsplit,
    error::BlockError,
    blocks::{Block, BlockInfo, BlockSeqNoAndShard},
    transactions::Transaction,
    messages::Message,
};
use std::cmp::max;
use ton_types::{
    Cell, CellType, BuilderData, error, fail, IBitstring, LevelMask, SliceData, Result, 
    UsageTree, types::UInt256
};


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleProof {
    pub hash: UInt256,
    pub depth: u16,
    pub proof: Cell,
}

impl Default for MerkleProof {
    fn default() -> MerkleProof {
        MerkleProof {
            hash: UInt256::default(),
            depth: 0,
            proof: Cell::default(),
        }
    }
}

impl Deserializable for MerkleProof {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if CellType::from(cell.get_next_byte()?) != CellType::MerkleProof {
            fail!(
                BlockError::InvalidData("invalid Merkle proof root's cell type".to_string())
            )
        }
        self.hash.read_from(cell)?;
        self.depth = cell.get_next_u16()?;
        self.proof = cell.checked_drain_reference()?.clone();
        if self.hash != Cell::hash(&self.proof, 0) {
            fail!(
                BlockError::WrongMerkleProof(
                    "Stored proof hash is not equal calculated one".to_string()
                )
            )
        }
        if self.depth != Cell::depth(&self.proof, 0) {
            fail!(
                BlockError::WrongMerkleProof(
                    "Stored proof depth is not equal calculated one".to_string() 
                )
            )
        }
        Ok(())
    }
}

impl Serializable for MerkleProof {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.set_type(CellType::MerkleProof);
        cell.append_u8(u8::from(CellType::MerkleProof))?;
        self.hash.write_to(cell)?;
        cell.append_u16(self.depth)?;
        cell.append_reference(BuilderData::from(&self.proof));
        cell.set_level_mask(LevelMask::for_merkle_cell(self.proof.level_mask()));
        Ok(())
    }
}

impl MerkleProof {

    /// Creating of a Merkle proof which includes cells whose hashes contain in `proof_for`.
    pub fn create<F>(root: &Cell, is_include: &F) -> Result<Self>
        where F: Fn(UInt256) -> bool {

        if !is_include(root.repr_hash()) {
            fail!(
                BlockError::InvalidArg(
                    "`bag` doesn't contain any cell to include into proof".to_string()
                )
            )
        }

        let proof = Self::traverse_on_create(root, is_include, 0)?;

        Ok(MerkleProof {
            hash: root.repr_hash(),
            depth: root.repr_depth(),
            proof: proof.into(),
        })
    }

    /// Creating of a Merkle proof which includes cells whose hashes contain in `proof_for`.
    pub fn create_by_usage_tree(root: &Cell, usage_tree: &UsageTree) -> Result<Self> {
        let visited = usage_tree.visited();
        let is_include = |h| {
            visited.contains(&h)
        };
        MerkleProof::create(root, &is_include)
    }

    fn traverse_on_create<F>(cell: &Cell, 
        is_include: &F, merkle_depth: u8)
        -> Result<BuilderData>
        where F: Fn(UInt256) -> bool {

        let child_merkle_depth = if cell.is_merkle() { 
            merkle_depth + 1 
        } else { 
            merkle_depth 
        };

        let mut proof_cell = BuilderData::new();
        proof_cell.set_type(cell.cell_type());
        let mut child_mask = LevelMask::with_mask(0);
        for child in cell.clone_references().iter() {
            let proof_child = if is_include(child.repr_hash()) {
                Self::traverse_on_create(child, is_include, child_merkle_depth)?
            } else {
                MerkleUpdate::make_pruned_branch_cell(child, child_merkle_depth)?
            };
            child_mask |= proof_child.level_mask();
            proof_cell.append_reference(proof_child);
        }
        
        proof_cell.set_level_mask(if cell.is_merkle() {
            LevelMask::for_merkle_cell(child_mask)
        } else {
            child_mask
        });

        let slice = cell.into();
        proof_cell.append_bytestring(&slice).unwrap();


        Ok(proof_cell)
    }
}

// checks if proof contains correct block info
pub fn check_block_info_proof(block: &Block, proof_hash: &UInt256, block_hash: &UInt256) -> Result<BlockInfo> {
    if proof_hash != block_hash {
        fail!(
            BlockError::WrongMerkleProof("Proof hash is not equal given block hash".to_string())
        )
    }
    block.read_info()
}

/// checks if transaction with given id is exist in block.
/// Proof must contain transaction's root cell and block info
pub fn check_transaction_proof(proof: &MerkleProof, tr: &Transaction, block_id: &UInt256) -> Result<()> {

    let block_virt_root = proof.proof.clone().virtualize(1);

    let block: Block = Block::construct_from(&mut block_virt_root.into())
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting block from proof: {}", err)
            )
        )?;

    // check if block id in transaction is corresponds to block in proof
    let block_info = check_block_info_proof(&block, &proof.hash, block_id)?;

    // check if acc is belonged the block's shard
    if !block_info.shard().contains_account(tr.account_id().clone())? {
        fail!(
            BlockError::WrongMerkleProof(
                "Account address in transaction belongs other shardchain".to_string()
            )
        )
    }

    // check if transaction is potencially belonged the block by logical time
    if tr.logical_time() < block_info.start_lt() || tr.logical_time() > block_info.end_lt() {
        fail!(
            BlockError::WrongMerkleProof(
                "Transaction's logical time doesn't belong to \
                 block's logical time interval".to_string()
            )
        )
    }

    // read account block from block and check it

    let block_extra = block.read_extra()
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting block extra from proof: {}", err)
            )
        )?;

    let account_blocks = block_extra.read_account_blocks()
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting account blocks from proof: {}", err)
            )
        )?;

    let account_block = account_blocks.get_serialized(tr.account_id().clone())?
        .ok_or_else(|| BlockError::WrongMerkleProof("No account block in proof".to_string()))?;

    // find transaction
    let tr_parent_slice_opt = account_block.transactions().get_as_slice(&tr.logical_time())
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting transaction from dictionary in proof: {}", err) 
            )
        )?;
    if let Some(mut tr_parent_slice) = tr_parent_slice_opt {
        if let Ok(tr_slice) = tr_parent_slice.checked_drain_reference() {
            // check hash
            if tr_slice.repr_hash() != tr.hash()? {
                fail!(
                    BlockError::WrongMerkleProof("Wrong transaction's hash in proof".to_string())
                )
            }
        }
    } else {
        fail!(BlockError::WrongMerkleProof("No transaction in proof".to_string()))
    }
    Ok(())
}

fn check_transaction_id(given_id: Option<UInt256>, tr_cell: Option<&Cell>) -> Result<()> {
    let existing_id = tr_cell.map(|c| c.repr_hash());
    match (given_id, existing_id) {
        (None, Some(_)) => {
            fail!(
                BlockError::WrongMerkleProof(
                    "Invalid transaction id: None is passed, \
                     but the transaction exists in a block".to_string()
                )
            )
        },
        (Some(_), None) => {
            fail!(
                BlockError::WrongMerkleProof(
                    "Invalid transaction id: it is passed, \
                     but the transaction doesn't exists in a block".to_string()
                )
            )
        },
        (None, None) => Ok(()),
        (Some(id1), Some(id2)) => {
            if id1 != id2 {
                fail!(BlockError::WrongMerkleProof("Invalid transaction id".to_string()))
            }
            Ok(())
        }
    }
}

/// checks if message with given id is exist in block.
/// Proof must contain message's root cell and block info
pub fn check_message_proof(proof: &MerkleProof, msg: &Message, block_id: &UInt256, tr_id: Option<UInt256>) -> Result<()> {

    let block_virt_root = proof.proof.clone().virtualize(1);

    let block: Block = Block::construct_from(&mut block_virt_root.into())
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting block from proof: {}", err)
            )
        )?;

    // check if block id in message is corresponds to block in proof
    check_block_info_proof(&block, &proof.hash, block_id)?;

    // read message from block and check it

    let block_extra = block.read_extra()
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting block extra from proof: {}", err)
            )
        )?;

    let msg_hash = msg.hash()?;
    // attempt to read in msg descr, if fail - read out one
    if let Ok(in_msg_descr) = block_extra.read_in_msg_descr() {
        if let Ok(Some(in_msg)) = in_msg_descr.get(&msg_hash) {
            check_transaction_id(tr_id, in_msg.transaction_cell())?;
            if let Ok(msg_cell) = in_msg.message_cell() {
                if msg_cell.repr_hash() != msg_hash {
                    fail!(
                        BlockError::WrongMerkleProof("Wrong message's hash in proof".to_string())
                    )
                } else {
                    return Ok(())
                }
            } else {
                fail!(
                    BlockError::WrongMerkleProof(
                        "Error extracting message from in message".to_string()
                    )
                )
            }
        }
    }

    let out_msg_descr = block_extra.read_out_msg_descr()
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting out msg descr from proof: {}", err)
            )
        )?;
    if let Ok(Some(out_msg)) = out_msg_descr.get(&msg_hash) {
        if let Ok(real_msg_hash) = out_msg.read_message_hash() {
            check_transaction_id(tr_id, out_msg.transaction_cell())?;
            if real_msg_hash != msg_hash {
                fail!(
                    BlockError::WrongMerkleProof("Wrong message's hash in proof".to_string())
                )
            } else {
                return Ok(())
            }
        } else {
            fail!(
                BlockError::WrongMerkleProof(
                    "Error extracting message from out message".to_string()
                ) 
            )
        }
    } else {
        fail!(BlockError::WrongMerkleProof("No message in proof".to_string()))
    }
}

/// checks if account with given address is exist in shard state.
/// Proof must contain account's root cell
/// Returns info about the block corresponds to shard state the account belongs to.
pub fn check_account_proof(proof: &MerkleProof, acc: &Account) -> Result<BlockSeqNoAndShard> {
    if acc.is_none() {
        fail!(BlockError::InvalidData("Account can't be none".to_string()))
    }

    let ss_virt_root = proof.proof.clone().virtualize(1);
    let ss: ShardStateUnsplit = ShardStateUnsplit::construct_from(&mut ss_virt_root.into())?;

    let accounts = ss.read_accounts()
        .map_err(
            |err| BlockError::WrongMerkleProof(
                format!("Error extracting accounts dict from proof: {}", err)
            )
        )?;

    let shard_acc = accounts.get_serialized(acc.get_addr().unwrap().get_address());
    if let Ok(Some(shard_acc)) = shard_acc {
        let acc_root = shard_acc.account_cell();
        let acc_hash = Cell::hash(&acc_root, (max(acc_root.level(), 1) - 1) as usize);
        if acc.hash()? != acc_hash {
            fail!(BlockError::WrongMerkleProof("Wrong account's hash in proof".to_string()))
        } else {
            return Ok(
                BlockSeqNoAndShard {
                    seq_no: ss.seq_no(),
                    vert_seq_no: ss.vert_seq_no(),
                    shard_id: ss.shard().clone(),
                }
            );
        }
    } else {
        fail!(BlockError::WrongMerkleProof("No account in proof".to_string()))
    }
}
