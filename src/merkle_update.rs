/*
* Copyright (C) 2019-2021 TON Labs. All Rights Reserved.
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
    error::BlockError,
    Serializable, Deserializable, MerkleProof,
};
use std::collections::{HashMap, HashSet};
use ton_types::{
    error, fail, Result,
    BagOfCells,
    UInt256,
    BuilderData, Cell, CellType, IBitstring, LevelMask, SliceData,
};


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
    pub old: Cell, // reference
    pub new: Cell, // reference
}

impl Default for MerkleUpdate {
    fn default() -> MerkleUpdate {
        let old = Cell::default();
        let new = Cell::default();
        MerkleUpdate {
            old_hash: Cell::hash(&old, 0),
            new_hash: Cell::hash(&new, 0),
            old_depth: 0,
            new_depth: 0,
            old,
            new,
        }
    }
}

impl Deserializable for MerkleUpdate {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if CellType::from(cell.get_next_byte()?) != CellType::MerkleUpdate {
            fail!(
                BlockError::InvalidData("invalid Merkle update root's cell type".to_string())
            )
        }
        self.old_hash.read_from(cell)?;
        self.new_hash.read_from(cell)?;
        self.old_depth = cell.get_next_u16()?;
        self.new_depth = cell.get_next_u16()?;
        self.old = cell.checked_drain_reference()?;
        self.new = cell.checked_drain_reference()?;

        if self.old_hash != Cell::hash(&self.old, 0) {
            fail!(
                BlockError::WrongMerkleUpdate(
                    "Stored old hash is not equal calculated one".to_string()
                )
            )
        }
        if self.new_hash != Cell::hash(&self.new, 0) {
            fail!(
                BlockError::WrongMerkleUpdate(
                    "Stored new hash is not equal calculated one".to_string() 
                )
            )
        }
        if self.old_depth != Cell::depth(&self.old, 0) {
            fail!(
                BlockError::WrongMerkleUpdate(
                    "Stored old depth is not equal calculated one".to_string()
                )
            )
        }
        if self.new_depth != Cell::depth(&self.new, 0) {
            fail!(
                BlockError::WrongMerkleUpdate(
                    "Stored new depth is not equal calculated one".to_string() 
                )
             )
        }

        Ok(())
    }
}

impl Serializable for MerkleUpdate {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.set_type(CellType::MerkleUpdate);
        cell.append_u8(u8::from(CellType::MerkleUpdate))?;
        self.old_hash.write_to(cell)?;
        self.new_hash.write_to(cell)?;
        cell.append_u16(self.old_depth)?;
        cell.append_u16(self.new_depth)?;
        cell.append_reference_cell(self.old.clone());
        cell.append_reference_cell(self.new.clone());
        cell.set_level_mask(LevelMask::for_merkle_cell(self.old.level_mask() | self.new.level_mask()));
        Ok(())
    }
}

impl MerkleUpdate {

    /// Creating of a Merkle update
    pub fn create(old: &Cell, new: &Cell) -> Result<MerkleUpdate> {

        if old.repr_hash() == new.repr_hash() {
            // if trees are the same
            let hash = old.repr_hash();
            let pruned_branch_cell = Self::make_pruned_branch_cell(old, 0)?.into_cell()?;
             Ok(MerkleUpdate {
                old_hash: hash.clone(),
                new_hash: hash,
                old_depth: old.repr_depth(),
                new_depth: old.repr_depth(),
                old: pruned_branch_cell.clone(),
                new: pruned_branch_cell,
            })
        } else {
            // trees traversal and update creating;
            let new_bag = BagOfCells::with_root(new);
            let new_cells = new_bag.cells();
            let mut pruned_branches = HashMap::new();

            let old_update_cell = match Self::traverse_old_on_create(old, new_cells, &mut pruned_branches, 0)? {
                Some(old_update_cell) => old_update_cell,
                // Nothing from old tree were pruned, lets prune all tree!
                None => Self::make_pruned_branch_cell(old, 0)?
            };
            let new_update_cell = Self::traverse_new_on_create(new, &pruned_branches)?;

            Ok(MerkleUpdate {
                old_hash: old.repr_hash(),
                new_hash: new.repr_hash(),
                old_depth: old.repr_depth(),
                new_depth: new.repr_depth(),
                old: old_update_cell.into_cell()?,
                new: new_update_cell.into_cell()?,
            })
        }
    }

    pub fn create_fast(old: &Cell, new: &Cell, is_visited_old: impl Fn(&UInt256) -> bool) -> Result<MerkleUpdate> {
        if old.repr_hash() == new.repr_hash() {
            // if trees are the same
            let hash = old.repr_hash();
            let pruned_branch_cell = Self::make_pruned_branch_cell(old, 0)?.into_cell()?;
             Ok(MerkleUpdate {
                old_hash: hash.clone(),
                new_hash: hash,
                old_depth: old.repr_depth(),
                new_depth: old.repr_depth(),
                old: pruned_branch_cell.clone(),
                new: pruned_branch_cell,
            })
        } else {
            let mut pruned_branches = Some(HashSet::new());
            let mut done_cells = HashMap::new();
            let new_update_cell = MerkleProof::create_raw(
                new, &|hash| !is_visited_old(hash), &|_| false, 0, &mut pruned_branches, &mut done_cells)?;
            let pruned_branches = pruned_branches.unwrap();

            let mut used_paths_cells = HashSet::new();
            let mut visited = HashSet::new();
            if Self::collect_used_paths_cells(old, &is_visited_old, &pruned_branches, 
                &mut HashSet::new(), &mut used_paths_cells, &mut visited) {
                used_paths_cells.insert(old.repr_hash());
            }

            let mut done_cells = HashMap::new();
            let old_update_cell = MerkleProof::create_raw(
                old, &|hash| used_paths_cells.contains(hash), &|_| false, 0, &mut None, &mut done_cells)?;

            Ok(MerkleUpdate {
                old_hash: old.repr_hash(),
                new_hash: new.repr_hash(),
                old_depth: old.repr_depth(),
                new_depth: new.repr_depth(),
                old: old_update_cell,
                new: new_update_cell,
            })
        }
    }

    fn collect_used_paths_cells(
        cell: &Cell,
        is_visited_old: &impl Fn(&UInt256) -> bool,
        pruned_branches: &HashSet<UInt256>,
        visited_pruned_branches: &mut HashSet<UInt256>,
        used_paths_cells: &mut HashSet<UInt256>,
        visited: &mut HashSet<UInt256>,
    ) -> bool {
        let repr_hash = cell.repr_hash();

        if visited.contains(&repr_hash) {
            return false;
        }
        visited.insert(repr_hash.clone());

        if used_paths_cells.contains(&repr_hash) {
            return false;
        }

        let is_pruned = if pruned_branches.contains(&repr_hash) {
            if visited_pruned_branches.contains(&repr_hash) {
                return false;
            }
            visited_pruned_branches.insert(repr_hash.clone());
            true
        } else {
            false
        };

        let mut collect = false;
        if is_visited_old(&repr_hash) {
            for r in cell.clone_references() {
                collect |= Self::collect_used_paths_cells(
                    &r,
                    is_visited_old,
                    pruned_branches,
                    visited_pruned_branches,
                    used_paths_cells,
                    visited
                );
            }
            if collect {
                used_paths_cells.insert(repr_hash);
            }
        }
        collect | is_pruned
    }

    /// Applies update to given tree of cells by returning new updated one
    pub fn apply_for(&self, old_root: &Cell) -> Result<Cell> {

        let old_cells = self.check(old_root)?;

        // cells for new bag
        if self.new_hash == self.old_hash {
            Ok(old_root.clone())
        } else {
            let new_root = self.traverse_on_apply(&self.new, &old_cells, &mut HashMap::new(), 0)?;

            // constructed tree's hash have to coinside with self.new_hash
            if new_root.repr_hash() != self.new_hash {
                fail!(BlockError::WrongMerkleUpdate("new bag's hash mismatch".to_string()))
            }

            Ok(new_root)
        }
    }

    /// Check the update corresponds given bag.
    /// The function is called from `apply_for`
    pub fn check(&self, old_root: &Cell) -> Result<HashMap<UInt256, Cell>> {

        // check that hash of `old_tree` is equal old hash from `self`
        if self.old_hash != old_root.repr_hash() {
            fail!(BlockError::WrongMerkleUpdate("old bag's hash mismatch".to_string()))
        }

        // traversal along `self.new` and check all pruned branches.
        // All new tree's pruned branches have to be contained in old one
        let mut known_cells = HashSet::new();
        Self::traverse_old_on_check(&self.old, &mut known_cells, &mut HashSet::new(), 0);
        Self::traverse_new_on_check(&self.new, &known_cells, &mut HashSet::new(), 0)?;

        let mut known_cells_vals = HashMap::new();
        Self::collate_old_cells(old_root, &known_cells, &mut known_cells_vals, &mut HashSet::new(), 0);

        Ok(known_cells_vals)
    }

    /// Recursive traverse merkle update tree while merkle update applying
    /// `cell` ordinary cell from merkle update's new tree;
    /// `old_cells` cells from old bag of cells;
    fn traverse_on_apply(&self,
        update_cell: &Cell,
        old_cells: &HashMap<UInt256, Cell>,
        new_cells: &mut HashMap<UInt256, Cell>,
        merkle_depth: u8
    ) -> Result<Cell> {

        // We will recursively construct new skeleton for new cells 
        // and connect unchanged branches to it

        let mut new_cell = BuilderData::new();
        new_cell.set_type(update_cell.cell_type());

        let child_merkle_depth = if update_cell.is_merkle() { 
            merkle_depth + 1 
        } else { 
            merkle_depth 
        };

        // traverse references
        let mut child_mask = LevelMask::with_mask(0);
        for update_child in update_cell.clone_references().iter() {
            let new_child = match update_child.cell_type() {
                CellType::Ordinary | CellType::MerkleProof | CellType::MerkleUpdate | CellType::LibraryReference => {
                    let new_child_hash = update_child.hash(child_merkle_depth as usize);
                    if let Some(c) = new_cells.get(&new_child_hash) {
                        c.clone()
                    } else {
                        let c = self.traverse_on_apply(update_child, old_cells, new_cells, child_merkle_depth)?;
                        new_cells.insert(new_child_hash, c.clone());
                        c
                    }
                },
                CellType::PrunedBranch => {
                    // if this pruned branch is related to current update
                    let mask = update_child.level_mask().mask();
                    if mask & (1 << child_merkle_depth) != 0 {
                        // connect branch from old bag instead pruned
                        let new_child_hash = Cell::hash(update_child, update_child.level() as usize - 1);
                        old_cells.get(&new_child_hash)
                            .ok_or_else(|| error!("Can't get child with hash {:x}", new_child_hash))?
                            .clone()
                    } else {
                        // else - just copy this cell (like an ordinary)
                        update_child.clone()
                    }
                },
                _ => fail!("Unknown cell type while applying merkle update!")
            };
            child_mask |= new_child.level_mask();
            new_cell.append_reference_cell(new_child);
        }

        new_cell.set_level_mask(if update_cell.is_merkle() {
            LevelMask::for_merkle_cell(child_mask)
        } else {
            child_mask
        });

        // Copy data from update to constructed cell
        new_cell.append_bytestring(&SliceData::load_cell_ref(update_cell)?)?;

        new_cell.into_cell()
    }

    fn traverse_new_on_create(
            new_cell: &Cell, 
            common_pruned: &HashMap<UInt256, Cell>) -> Result<BuilderData> {

        let mut new_update_cell = BuilderData::new();
        new_update_cell.set_type(new_cell.cell_type());
        let mut level_mask = new_cell.level_mask();
        for child in new_cell.clone_references().iter() {
            let update_child =
                if let Some(pruned) = common_pruned.get(&child.repr_hash()) {
                    pruned.clone()
                } else {
                    Self::traverse_new_on_create(child, common_pruned)?.into_cell()?
                };
            level_mask |= update_child.level_mask();
            new_update_cell.append_reference_cell(update_child);
        }

        new_update_cell.set_level_mask(if new_cell.is_merkle() {
            LevelMask::for_merkle_cell(level_mask)
        } else {
            level_mask
        });

        new_update_cell.append_bytestring(&SliceData::load_cell_ref(new_cell)?)?;

        Ok(new_update_cell)
    }

    // If old_cell's child contains in new_cells - it transformed to pruned branch cell,
    //   else - recursion call for the child.
    // If any child is pruned branch (or contains pruned branch among their subtree) 
    //   - all other skipped childs are transformed to pruned branches
    //   else - skip this cell (return None)
    fn traverse_old_on_create(
        old_cell: &Cell,
        new_cells: &HashMap<UInt256, (Cell, u32)>,
        pruned_branches: &mut HashMap<UInt256, Cell>,
        mut merkle_depth: u8,
    ) -> Result<Option<BuilderData>> {

        if old_cell.is_merkle() { 
            merkle_depth += 1;
        }

        let mut childs = vec!(None; old_cell.references_count());
        let mut has_pruned = false;

        for (i, child) in old_cell.clone_references().iter().enumerate() {
            let child_hash = child.repr_hash();
            if let Some((common_cell, _)) = new_cells.get(&child_hash) {

                let pruned_branch_cell = Self::make_pruned_branch_cell(common_cell, merkle_depth)?;
                pruned_branches.insert(child_hash, pruned_branch_cell.clone().into_cell()?);

                childs[i] = Some(pruned_branch_cell);
                has_pruned = true;
            } else {
                childs[i] = Self::traverse_old_on_create(child, new_cells, pruned_branches, merkle_depth)?;
                if childs[i].is_some() {
                    has_pruned = true;
                }
            }
        }

        if has_pruned {

            let mut old_update_cell = BuilderData::new();
            old_update_cell.set_type(old_cell.cell_type());
            let mut child_mask = LevelMask::with_mask(0);
            for (i, child_opt) in childs.into_iter().enumerate() {
                let child = match child_opt {
                    None => {
                        let child = old_cell.reference(i)?;
                        Self::make_pruned_branch_cell(&child, merkle_depth)?
                    }
                    Some(child) => child
                };
                child_mask |= child.level_mask();
                old_update_cell.append_reference_cell(child.into_cell()?);
            }
            old_update_cell.set_level_mask(if old_cell.is_merkle() {
                LevelMask::for_merkle_cell(child_mask)
            } else {
                child_mask
            });

            old_update_cell.append_bytestring(&SliceData::load_cell_ref(old_cell)?)?;
            Ok(Some(old_update_cell))

        } else {
            Ok(None)
        }
    }

    fn add_one_hash(cell: &Cell, depth: u8) -> Result<LevelMask> {
        let mask = cell.level_mask().mask();
        if depth > 2 { 
            fail!(BlockError::InvalidArg("depth".to_string()))
        } else if mask & (1 << depth) != 0 {
            fail!(
                BlockError::InvalidOperation(
                    format!("attempt to add hash with depth {} into mask {:03b}", depth, mask)
                )
            )
        }
        Ok(LevelMask::with_mask(mask | (1 << depth)))
    }

    pub(crate) fn make_pruned_branch_cell(cell: &Cell, merkle_depth: u8) -> Result<BuilderData> {

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

    fn traverse_old_on_check(cell: &Cell, known_cells: &mut HashSet<UInt256>, visited: &mut HashSet<UInt256>, merkle_depth: u8) {
        if visited.insert(cell.repr_hash()) {
            known_cells.insert(cell.hash(merkle_depth as usize));
            if cell.cell_type() != CellType::PrunedBranch {
                let child_merkle_depth = if cell.is_merkle() { merkle_depth + 1 } else { merkle_depth };
                for child in cell.clone_references().iter() {
                    Self::traverse_old_on_check(child, known_cells, visited, child_merkle_depth);
                }
            }
        }
    }

    // Checks all pruned branches from new tree are exist in old tree
    fn traverse_new_on_check(
        cell: &Cell, 
        known_cells: &HashSet<UInt256>, 
        visited: &mut HashSet<UInt256>, 
        merkle_depth: u8
    ) -> Result<()> {
        if visited.insert(cell.repr_hash()) {
            if cell.cell_type() == CellType::PrunedBranch {
                if cell.level() == merkle_depth + 1 &&
                   !known_cells.contains(&cell.hash(merkle_depth as usize))
                {
                    fail!("old and new trees mismatch {:x}", cell.hash(merkle_depth as usize))
                }
            } else {
                let child_merkle_depth = if cell.is_merkle() { merkle_depth + 1 } else { merkle_depth };
                for child in cell.clone_references().iter() {
                    Self::traverse_new_on_check(child, known_cells, visited, child_merkle_depth)?;
                }
            }
        }
        Ok(())
    }

    fn collate_old_cells(
        cell: &Cell, 
        known_cells_hashes: &HashSet<UInt256>, 
        known_cells: &mut HashMap<UInt256, Cell>, 
        visited: &mut HashSet<UInt256>, 
        merkle_depth: u8
    ) {
        if visited.insert(cell.repr_hash()) {
            let hash = cell.hash(merkle_depth as usize);
            if known_cells_hashes.contains(&hash) {
                known_cells.insert(hash, cell.clone());
                let child_merkle_depth = if cell.is_merkle() { merkle_depth + 1 } else { merkle_depth };
                for child in cell.clone_references().iter() {
                    Self::collate_old_cells(child, known_cells_hashes, known_cells, visited, child_merkle_depth);
                }
            }
        }
    }
}
