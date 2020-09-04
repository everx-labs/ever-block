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
        self.old = cell.checked_drain_reference()?.clone();
        self.new = cell.checked_drain_reference()?.clone();

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
        cell.append_reference(BuilderData::from(&self.old));
        cell.append_reference(BuilderData::from(&self.new));
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

    pub fn create_fast(old: &Cell, new: &Cell, is_visited_old: impl Fn(&UInt256) -> bool) -> Result<MerkleUpdate> {
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
            // * for old tree - build merkle proof using usage tree.
            //   Need to collect all pruned branches from old tree's proof while buildung.
            // * for new tree - build merkle proof and prune branches which were pruned in old tree's proof
            // * if new tree contains subtree which included into old-tree but was not visited
            //   (not included into old-usage-tree) - this subtree will be duplicated
            //   in a merkle update's new tree. But update will be built much faster 
            //   than using full traverse.

            let mut pruned_branches = Some(HashMap::new());
            let old_update_cell = MerkleProof::create_raw(old, &is_visited_old, 0, &mut pruned_branches)?;
            
            let new_update_cell = Self::traverse_new_on_create(new, &pruned_branches.unwrap());
            
            Ok(MerkleUpdate {
                old_hash: old.repr_hash(),
                new_hash: new.repr_hash(),
                old_depth: old.repr_depth(),
                new_depth: new.repr_depth(),
                old: old_update_cell.into(),
                new: new_update_cell.into(),
            })
        }
    }

    /// Applies update to given tree of cells by returning new updated one
    pub fn apply_for(&self, old_root: &Cell) -> Result<Cell> {

        let old_cells = self.check(old_root)?;

        // cells for new bag
        if self.new_hash == self.old_hash {
            Ok(old_root.clone())
        } else {
            let new_root: Cell =
                self.traverse_on_apply(&self.new, &old_cells, 0).into();

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

        // traversal along `self.new` and check all pruned branches,
        // all new tree's pruned branches have to be contained in old one
        let mut known_cells = HashSet::new();
        Self::traverse_old_on_check(&self.old, &mut known_cells, &mut HashSet::new(), 0);
        if !Self::traverse_new_on_check(&self.new, &known_cells, &mut HashSet::new(), 0) {
            fail!(
                BlockError::WrongMerkleUpdate("old and new trees mismatch".to_string())
            )
        }

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
        merkle_depth: u8
    ) -> BuilderData {

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
                CellType::Ordinary | CellType::MerkleProof | CellType::MerkleUpdate => {
                    self.traverse_on_apply(update_child, old_cells, child_merkle_depth)
                },
                CellType::PrunedBranch => {
                    // if this pruned branch is related to current update
                    let mask = update_child.level_mask().mask();
                    if mask & (1 << child_merkle_depth) != 0 {
                        // connect branch from old bag instead pruned
                        let new_child_hash = Cell::hash(&update_child, update_child.level() as usize - 1);
                        BuilderData::from(old_cells.get(&new_child_hash).unwrap())
                    } else {
                        // else - just copy this cell (like an ordinary)
                        BuilderData::from(update_child)
                    }
                },
                CellType::LibraryReference => {
                    unimplemented!() // TODO
                },
                _ => panic!("Unknown cell type!")
            };
            child_mask |= new_child.level_mask();
            new_cell.append_reference(new_child);
        }

        new_cell.set_level_mask(if update_cell.is_merkle() {
            LevelMask::for_merkle_cell(child_mask)
        } else {
            child_mask
        });

        // Copy data from update to constructed cell
        new_cell.append_bytestring(&SliceData::from(update_cell)).unwrap();

        new_cell
    }

    fn traverse_new_on_create(
            new_cell: &Cell, 
            common_pruned: &HashMap<UInt256, Cell>) -> BuilderData {

        let mut new_update_cell = BuilderData::new();
        let mut child_mask = LevelMask::with_mask(0);
        for child in new_cell.clone_references().iter() {
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
        old_cell: &Cell,
        new_cells: &HashMap<UInt256, Cell>,
        pruned_branches: &mut HashMap<UInt256, Cell>)
        -> Result<Option<BuilderData>> {

        let mut childs = vec!(None; old_cell.references_count());
        let mut has_pruned = false;

        for (i, child) in old_cell.clone_references().iter().enumerate() {
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

    pub(crate) fn make_pruned_branch_cell(cell: &Cell, merkle_depth: u8) 
        -> Result<BuilderData> {

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
    fn traverse_new_on_check(cell: &Cell, known_cells: &HashSet<UInt256>, visited: &mut HashSet<UInt256>, merkle_depth: u8) -> bool{
        if visited.insert(cell.repr_hash()) {
            if cell.cell_type() == CellType::PrunedBranch {
                if cell.level() == merkle_depth + 1 &&
                    !known_cells.contains(&cell.hash(merkle_depth as usize)) {
                    return false;
                }
            } else {
                let child_merkle_depth = if cell.is_merkle() { merkle_depth + 1 } else { merkle_depth };
                for child in cell.clone_references().iter() {
                    if !Self::traverse_new_on_check(child, known_cells, visited, child_merkle_depth) {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn collate_old_cells(cell: &Cell, known_cells_hashes: &HashSet<UInt256>, known_cells: &mut HashMap<UInt256, Cell>, visited: &mut HashSet<UInt256>, merkle_depth: u8) {
        if visited.insert(cell.repr_hash()) {
            let hash = cell.hash(merkle_depth as usize);
            if known_cells_hashes.contains(&hash) {
                known_cells.insert(hash, cell.clone());
                let child_merkle_depth = if cell.is_merkle() { merkle_depth + 1 } else { merkle_depth };
                for child in cell.clone_references().iter() {
                    Self::collate_old_cells(&child, known_cells_hashes, known_cells, visited, child_merkle_depth);
                }
            }
        }
    }
}