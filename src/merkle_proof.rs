/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use std::cmp::max;
use std::sync::Arc;
use ton_types::{SliceData, CellData, CellType, BuilderData, IBitstring, LevelMask};
use ton_types::cells_serialization::BagOfCells;
use UInt256;
use std::collections::{HashMap};
use super::{BlockErrorKind, MerkleUpdate, BlockResult, Serializable, Deserializable,
    Block, BlockExtra, BlockInfo, Transaction, GetRepresentationHash, AccountBlock, BlockError, 
    Message, InMsg, OutMsg, Account, ShardStateUnsplit};


#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleProof {
    pub hash: UInt256,
    pub depth: u16,
    pub proof: Arc<CellData>,
}

impl Default for MerkleProof {
    fn default() -> MerkleProof {
        MerkleProof {
            hash: UInt256::default(),
            depth: 0,
            proof: Arc::new(CellData::default()),
        }
    }
}

impl Deserializable for MerkleProof {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        if CellType::from(cell.get_next_byte()?) != CellType::MerkleProof {
            bail!(BlockErrorKind::InvalidData("invalid Merkle proof root's cell type".into()))
        }
        self.hash.read_from(cell)?;
        self.depth = cell.get_next_u16()?;
        self.proof = cell.checked_drain_reference()?.clone();

        if self.hash != CellData::hash(&self.proof, 0) {
            bail!(BlockErrorKind::WrongMerkleProof("Stored proof hash is not equal calculated one".into()));
        }
        if self.depth != CellData::depth(&self.proof, 0) {
            bail!(BlockErrorKind::WrongMerkleProof("Stored proof depth is not equal calculated one".into()));
        }

        Ok(())
    }
}

impl Serializable for MerkleProof {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
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
    /// Other words, create a proof for given cells.
    /// Remark: it include only given cells, not their referents. Cells from given to root 
    ///         is included too.
    pub fn create<F>(root: &Arc<CellData>, is_include: &F)
        -> BlockResult<Self>
        where F: Fn(UInt256) -> bool {

        let bag = BagOfCells::with_root(root);
        let proof = Self::traverse_on_create(bag.cells(), root, is_include, 0)?
            .ok_or(BlockErrorKind::InvalidArg(
                "`bag` doesn't contain any cell to include into proof".into()))?;

        Ok(MerkleProof {
            hash: root.repr_hash(),
            depth: root.repr_depth(),
            proof: proof.into(),
        })
    }

    fn traverse_on_create<F>(cells: &HashMap<UInt256, Arc<CellData>>, cell: &Arc<CellData>, 
        is_include: &F, merkle_depth: u8)
        -> BlockResult<Option<BuilderData>>
        where F: Fn(UInt256) -> bool {

        let mut childs = Vec::with_capacity(cell.references_used());
        let mut has_childs = false;

        let child_merkle_depth = if cell.is_merkle() { merkle_depth + 1 } else { merkle_depth };
        for child in cell.references().iter() {
            let child_proof_cell = Self::traverse_on_create(cells, child, is_include, child_merkle_depth)?;
            if child_proof_cell.is_some() {
                has_childs = true;
            }
            childs.push(child_proof_cell);
        }

        if has_childs || is_include(cell.repr_hash()) {
            let mut proof_cell = BuilderData::new();
            proof_cell.set_type(cell.cell_type());
            let mut child_mask = LevelMask::with_mask(0);
            let mut i = 0;
            for child_opt in childs {
                let child = if child_opt.is_some() {
                    child_opt.unwrap()
                } else {
                    let child = &cell.reference(i).unwrap();
                    MerkleUpdate::make_pruned_branch_cell(child, child_merkle_depth)?
                };
                child_mask |= child.level_mask();
                proof_cell.append_reference(child);
                i += 1;
            }
            proof_cell.set_level_mask(if cell.is_merkle() {
                    LevelMask::for_merkle_cell(child_mask)
                } else {
                    child_mask
                });

            let slice = cell.into();
            proof_cell.append_bytestring(&slice).unwrap();
            Ok(Some(proof_cell))

        } else {
            Ok(None)
        }
    }
}

// checks if proof contains correct block info
pub fn check_block_info_proof(proof: &MerkleProof, block_hash: UInt256) -> BlockResult<BlockInfo> {
    if proof.hash != block_hash {
        bail!(BlockErrorKind::WrongMerkleProof("Proof hash is not equal given block hash".into()));
    }
    Ok(Block::read_info_from(&mut SliceData::from(&proof.proof))?)
}

/// checks if transaction with given id is exist in block.
/// Proof must contain transaction (TODO only tr's root cell) and block info
pub fn check_transaction_proof(proof: &MerkleProof, tr: &Transaction) -> BlockResult<()> {

    // check if block id in transaction is corresponds to block in proof
    let block_info = match &tr.block_id {
        Some(block_id) => check_block_info_proof(proof, block_id.clone())?,
        None => bail!(BlockErrorKind::InvalidData("Transaction must contain a block id".into()))
    };

    // check if acc is belonged the block's shard
    if !block_info.shard.contains_account(tr.account_id().clone())? {
        bail!(BlockErrorKind::WrongMerkleProof(
            "Account address in transaction belongs other shardchain".into()));
    }

    // check if transaction is potencially belonged the block by logical time
    if tr.logical_time() < block_info.start_lt || tr.logical_time() > block_info.end_lt {
        bail!(BlockErrorKind::WrongMerkleProof(
            "Transaction's logical time isn't belongs block's logical time interval".into()));
    }

    // read block extra
    let mut block_extra_slice = Block::read_extra_slice_from(&mut SliceData::from(&proof.proof))
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
                format!("Error extracting block extra from proof: {}", e))))?;

    // read account blocks root
    let account_blocks = BlockExtra::read_account_blocks_from(&mut block_extra_slice)
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
            format!("Error extracting account blocks from proof: {}", e))))?;

    // read transactions dict from account block
    let mut account_block_slice = account_blocks.get_as_slice(tr.account_id())?
        .ok_or(BlockError::from(BlockErrorKind::WrongMerkleProof(
                "No account block in proof".into())))?;
    let tr_dict = AccountBlock::read_transactions_from(&mut account_block_slice)
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
            format!("Error extracting transactions dictionary from account blocks in proof: {}", e))))?;

    // find transaction
    // TODO: read only transactions's root cell
    let tr1 = tr_dict.get(&tr.logical_time())
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
            format!("Error extracting transaction from dictionary in proof: {}", e))))?;
    if let Some(tr1) = tr1 {
        // check hash
        if tr1.0.hash()? != tr.hash()? {
            bail!(BlockErrorKind::WrongMerkleProof(
                "Wrong transaction's hash in proof".into()));
        }
    } else {
        bail!(BlockErrorKind::WrongMerkleProof(
            "No transaction in proof".into()));
    }
    Ok(())
}

/// checks if message with given id is exist in block.
/// Proof must contain message (TODO only message's root cell) and block info
pub fn check_message_proof(proof: &MerkleProof, msg: &Message) -> BlockResult<()> {

    // check if block id in message is corresponds to block in proof
    let _block_info = match &msg.block_id {
        Some(block_id) => check_block_info_proof(proof, block_id.clone())?,
        None => bail!(BlockErrorKind::InvalidData("Message must contain a block id".into()))
    };

    // read block extra
    let mut block_extra_slice = Block::read_extra_slice_from(&mut SliceData::from(&proof.proof))
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
                format!("Error extracting block extra from proof: {}", e))))?;

    let msg_hash = msg.hash()?;
    // attempt to read in msg descr, if fail - read out one
    if let Ok(in_msg_descr) = BlockExtra::read_in_msg_descr_from(&mut block_extra_slice.clone()) {
        let in_msg_slice = in_msg_descr.get_as_slice(&msg_hash);
        if let Ok(Some(mut in_msg_slice)) = in_msg_slice {
            if let Ok(msg1) = InMsg::read_message_from(&mut in_msg_slice) {
                if msg1.hash()? != msg_hash {
                    bail!(BlockErrorKind::WrongMerkleProof(
                        "Wrong message's hash in proof".into()));
                } else {
                    return Ok(());
                }
            }
        }
    }

    let out_msg_descr = BlockExtra::read_out_msg_descr_from(&mut block_extra_slice)
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
            format!("Error extracting out msg descr from proof: {}", e))))?;
    let out_msg_slice = out_msg_descr.get_as_slice(&msg_hash);
    if let Ok(Some(mut out_msg_slice)) = out_msg_slice {
        if let Ok(msg1) = OutMsg::read_message_from(&mut out_msg_slice) {
            if msg1.hash()? != msg_hash {
                bail!(BlockErrorKind::WrongMerkleProof(
                    "Wrong message's hash in proof".into()));
            } else {
                return Ok(());
            }
        } else {
            bail!(BlockErrorKind::WrongMerkleProof(
                "Error extracting message from out message".into()));
        }
    } else {
        bail!(BlockErrorKind::WrongMerkleProof(
            "No message in proof".into()));
    }
}

/// checks if account with given address is exist in shard state.
/// Proof must contain account's root cell
pub fn check_account_proof(proof: &MerkleProof, acc: &Account) -> BlockResult<()> {
    if acc.is_none() {
        bail!(BlockErrorKind::InvalidData("Account can't be none".into()));
    }

    let ss: ShardStateUnsplit = ShardStateUnsplit::construct_from(&mut SliceData::from(&proof.proof))?;

    let accounts = ss.read_accounts()
        .map_err(|e| BlockError::from(BlockErrorKind::WrongMerkleProof(
            format!("Error extracting accounts dict from proof: {}", e))))?;

    let shard_acc = accounts.get(&acc.get_addr().unwrap().get_address());
    if let Ok(Some(shard_acc)) = shard_acc {
        let acc_root = shard_acc.account_cell();
        let acc_hash = CellData::hash(&acc_root, (max(acc_root.level(), 1) - 1) as usize);
        if acc.hash()? != acc_hash {
            bail!(BlockErrorKind::WrongMerkleProof(
                "Wrong account's hash in proof".into()));
        } else {
            return Ok(());
        }
    } else {
        bail!(BlockErrorKind::WrongMerkleProof(
            "No account in proof".into()));
    }
}