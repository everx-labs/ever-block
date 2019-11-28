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

use ton_types::{SliceData, CellData, CellType, BuilderData, IBitstring, LevelMask};
use UInt256;
use ton_types::cells_serialization::{BagOfCells};
use std::collections::HashMap;
use super::*;


/*
!merkle_update {X:Type} old_hash:uint256 new_hash:uint256
old:^X new:^X = MERKLE_UPDATE X;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleUpdate {
    pub old_hash: UInt256,
    pub new_hash: UInt256,
    pub old_depth: u16,
    pub new_depth: u16,
    pub old: Arc<CellData>, // reference
    pub new: Arc<CellData>, // reference
}

impl Default for MerkleUpdate {
    fn default() -> MerkleUpdate {
        let old = Arc::new(CellData::default());
        let new = Arc::new(CellData::default());
        MerkleUpdate {
            old_hash: CellData::hash(&old, 0),
            new_hash: CellData::hash(&new, 0),
            old_depth: 0,
            new_depth: 0,
            old,
            new,
        }
    }
}

impl Deserializable for MerkleUpdate {
    fn read_from(&mut self, cell: &mut SliceData) -> BlockResult<()> {
        if CellType::from(cell.get_next_byte()?) != CellType::MerkleUpdate {
            bail!(BlockErrorKind::InvalidData("invalid Merkle update root's cell type".into()))
        }
        self.old_hash.read_from(cell)?;
        self.new_hash.read_from(cell)?;
        self.old_depth = cell.get_next_u16()?;
        self.new_depth = cell.get_next_u16()?;
        self.old = cell.checked_drain_reference()?.clone();
        self.new = cell.checked_drain_reference()?.clone();

        if self.old_hash != CellData::hash(&self.old, 0) {
            bail!(BlockErrorKind::WrongMerkleUpdate("Stored old hash is not equal calculated one".into()));
        }
        if self.new_hash != CellData::hash(&self.new, 0) {
            bail!(BlockErrorKind::WrongMerkleUpdate("Stored new hash is not equal calculated one".into()));
        }
        if self.old_depth != CellData::depth(&self.old, 0) {
            bail!(BlockErrorKind::WrongMerkleUpdate("Stored old depth is not equal calculated one".into()));
        }
        if self.new_depth != CellData::depth(&self.new, 0) {
            bail!(BlockErrorKind::WrongMerkleUpdate("Stored new depth is not equal calculated one".into()));
        }

        Ok(())
    }
}

impl Serializable for MerkleUpdate {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.set_type(CellType::MerkleUpdate);
        cell.append_u8(u8::from(CellType::MerkleUpdate))?;
        self.old_hash.write_to(cell)?;
        self.new_hash.write_to(cell)?;
        cell.append_u16(self.old_depth)?;
        cell.append_u16(self.new_depth)?;
        cell.append_reference(BuilderData::from(&self.old));
        cell.append_reference(BuilderData::from(&self.new));
        cell.set_level_mask(LevelMask::for_merkle_cell(self.old.level_mask() | self.new.level_mask()));
        Ok(())
    }
}

impl MerkleUpdate {

    /// Creating of a Merkle update
    pub fn create(old: &Arc<CellData>, new: &Arc<CellData>) -> BlockResult<MerkleUpdate> {

        if old.repr_hash() == new.repr_hash() {
            // if trees are the same
            let hash = old.repr_hash();
            let pruned_branch_cell = Self::make_pruned_branch_cell(old, 0)?;
             Ok(MerkleUpdate {
                old_hash: hash.clone(),
                new_hash: hash,
                old_depth: old.repr_depth(),
                new_depth: old.repr_depth(),
                old: pruned_branch_cell.clone().into(),
                new: pruned_branch_cell.into(),
            })
        } else {
            // trees traversal and update creating;
            let new_bag = BagOfCells::with_root(new);
            let new_cells = new_bag.cells();
            let mut pruned_branches = HashMap::new();

            let mut old_update_cell = 
                Self::traverse_old_on_create(old, new_cells, &mut pruned_branches)?;
            if old_update_cell.is_none() {
                // Nothing from old tree were pruned, lets prune all tree!
                old_update_cell = Some(Self::make_pruned_branch_cell(old, 0)?);
            }
            let new_update_cell = Self::traverse_new_on_create(new, &pruned_branches);

            Ok(MerkleUpdate {
                old_hash: old.repr_hash(),
                new_hash: new.repr_hash(),
                old_depth: old.repr_depth(),
                new_depth: new.repr_depth(),
                old: old_update_cell.unwrap().into(),
                new: new_update_cell.into(),
            })
        }
    }

    /// Applies update to given tree of cells by returning new updated one
    pub fn apply_for(&self, old_root: &Arc<CellData>) -> BlockResult<Arc<CellData>> {
        let old_bag = BagOfCells::with_root(old_root);
        let old_cells = old_bag.withdraw_cells();
        self._apply_for(old_root, old_cells)
    }

    pub fn apply_many_for<I>(mut updates: I, old_root: &Arc<CellData>) -> BlockResult<Arc<CellData>> 
        where I: Iterator<Item = MerkleUpdate> {

        if let Some(first_update) = updates.next() {
            let old_bag = BagOfCells::with_root(old_root);
            let old_cells = old_bag.withdraw_cells();
            let mut new_root = first_update._apply_for(old_root, old_cells)?;

            for update in updates {
                let old_bag = BagOfCells::with_root(&new_root);
                let old_cells = old_bag.withdraw_cells();
                new_root = update._apply_for(&new_root, old_cells)?;
            }

            Ok(new_root)
        } else {
            bail!(BlockErrorKind::InvalidArg("updates".into()))
        }
    }

    fn _apply_for(&self, old_root: &Arc<CellData>, old_cells: HashMap<UInt256, Arc<CellData>>)
        -> BlockResult<Arc<CellData>> {

        self.check(&old_root.repr_hash())?;

        // cells for new bag
        if self.new_hash == self.old_hash {
            Ok(Arc::clone(old_root))
        } else {
            let new_root: Arc<CellData> =
                self.traverse_on_apply(&self.new, &old_cells).into();

            // constructed tree's hash have to coinside with self.new_hash
            assert_eq!(new_root.repr_hash(), self.new_hash);

            Ok(new_root)
        }
    }

    /// Check the update corresponds given bag.
    /// The function is called from `apply_for`
    pub fn check(&self, old_repr_hash: &UInt256) -> BlockResult<()> {

        // check that hash of `old_tree` is equal old hash from `self`
        if &self.old_hash != old_repr_hash {
            bail!(BlockErrorKind::WrongMerkleUpdate("old bag's hash mismatch".into()));
        }

        // traversal along `self.new` and check all pruned branches,
        // all new tree's pruned brunches have to be contained in old one
        let old_bag = BagOfCells::with_root(&self.old);
        let old_cells = old_bag.withdraw_cells();
        if !Self::traverse_on_check(&old_cells, &self.new) {
            bail!(BlockErrorKind::WrongMerkleUpdate("old and new trees mismatch".into()));
        }

        Ok(())
    }

    /// Recursive traverse merkle update tree while merkle update applying
    /// `cell` ordinary cell from merkle update's new tree;
    /// `old_cells` cells from old bag of cells;
    fn traverse_on_apply(&self,
        update_cell: &Arc<CellData>,
        old_cells: &HashMap<UInt256, Arc<CellData>>) -> BuilderData {

        // We will recursively construct new skeleton for new cells 
        // and connect unchanged branches to it

        let mut new_cell = BuilderData::new();

        // traverse references
        for update_child in update_cell.references().iter() {
            let new_child = match update_child.cell_type() {
                CellType::Ordinary => {
                    self.traverse_on_apply(update_child, old_cells)
                },
                CellType::PrunedBranch => {
                    // connect branch from old bag instead pruned.
                    let new_child_hash = CellData::hash(&update_child, update_child.level() as usize - 1);
                    BuilderData::from(old_cells.get(&new_child_hash).unwrap())
                },
                CellType::LibraryReference | CellType::MerkleProof | CellType::MerkleUpdate => {
                    unimplemented!() // TODO
                },
                _ => panic!("Unknown cell type!")
            };            
            new_cell.append_reference(new_child);
        }

        // Copy data from update to constructed cell
        new_cell.append_bytestring(&SliceData::from(update_cell)).unwrap();

        new_cell
    }

    fn traverse_new_on_create(
            new_cell: &Arc<CellData>, 
            common_pruned: &HashMap<UInt256, Arc<CellData>>) -> BuilderData {

        let mut new_update_cell = BuilderData::new();
        let mut child_mask = LevelMask::with_mask(0);
        for child in new_cell.references().iter() {
            let update_child =
                if let Some(pruned) = common_pruned.get(&child.repr_hash()) {
                    BuilderData::from(pruned)
                } else {
                    Self::traverse_new_on_create(child, common_pruned)
                };
            child_mask |= child.level_mask();
            new_update_cell.append_reference(update_child);
        }
        new_update_cell.set_level_mask(child_mask);

        new_update_cell.append_bytestring(&SliceData::from(new_cell)).unwrap();

        new_update_cell
    }

    // If old_cell's child contains in new_cells - it transformed to pruned branch cell,
    //   else - recursion call for the child.
    // If any child is pruned branch (or contains pruned branch among their subtree) 
    //   - all other skipped childs are transformed to pruned branches
    //   else - skip this cell (return None)
    fn traverse_old_on_create(
        old_cell: &Arc<CellData>,
        new_cells: &HashMap<UInt256, Arc<CellData>>,
        pruned_branches: &mut HashMap<UInt256, Arc<CellData>>)
        -> BlockResult<Option<BuilderData>> {

        let mut childs = vec!(None; old_cell.references_used());
        let mut has_pruned = false;

        for (i, child) in old_cell.references().iter().enumerate() {
            let child_hash = child.repr_hash();
            if let Some(common_cell) = new_cells.get(&child_hash) {

                let pruned_branch_cell = Self::make_pruned_branch_cell(common_cell, 0)?;
                pruned_branches.insert(child_hash.clone(), (&pruned_branch_cell).into());

                childs[i] = Some(pruned_branch_cell);
                has_pruned = true;
            } else {
                childs[i] = Self::traverse_old_on_create(child, new_cells, pruned_branches)?;
                if childs[i].is_some() {
                    has_pruned = true;
                }
            }
        }

        if has_pruned {

            let mut old_update_cell = BuilderData::new();
            let mut child_mask = LevelMask::with_mask(0);
            let mut i = 0;
            for child_opt in childs {
                let child = if child_opt.is_some() {
                    child_opt.unwrap()
                } else {
                    let child = &old_cell.reference(i).unwrap();
                    Self::make_pruned_branch_cell(child, 0)?
                };
                child_mask |= child.level_mask();
                old_update_cell.append_reference(child);
                i += 1;
            }
            old_update_cell.set_level_mask(child_mask);

            old_update_cell.append_bytestring(&SliceData::from(old_cell)).unwrap();
            Ok(Some(old_update_cell))

        } else {
            Ok(None)
        }
    }

    fn add_one_hash(cell: &Arc<CellData>, depth: u8) -> BlockResult<LevelMask> {
        let mask = cell.level_mask().mask();
        if depth > 2 { 
            bail!(BlockErrorKind::InvalidArg("depth".into()))
        } else if mask & (1 << depth) != 0 {
            bail!(BlockErrorKind::InvalidOperation(format!(
                "attempt to add hash with depth {} into mask {:03b}", depth, mask)))
        }
        Ok(LevelMask::with_mask(mask | (1 << depth)))
    }

    pub(crate) fn make_pruned_branch_cell(cell: &Arc<CellData>, merkle_depth: u8) 
        -> BlockResult<BuilderData> {

        let mut result = BuilderData::new();
        let level_mask = Self::add_one_hash(cell, merkle_depth)?;
        result.set_type(CellType::PrunedBranch);
        result.set_level_mask(level_mask);
        result.append_u8(u8::from(CellType::PrunedBranch))?;
        result.append_u8(level_mask.mask())?;
        for hash in cell.hashes() {
            result.append_raw(hash.as_slice(), hash.as_slice().len() * 8)?;
        }
        for depth in cell.depths() {
            result.append_u16(depth)?;
        }
        Ok(result)
    }

    // Checks all pruned brunches from new tree are exist in old tree
    fn traverse_on_check(old_cells: &HashMap<UInt256, Arc<CellData>>, new_cell: &Arc<CellData>) -> bool {
        for child in new_cell.references().iter() {
            if child.cell_type() == CellType::PrunedBranch {
                if !old_cells.contains_key(&child.repr_hash()) {
                    return false;
                }
            } else {
                if !Self::traverse_on_check(old_cells, child) {
                    return false;
                }
            }
        }
        true
    }
}