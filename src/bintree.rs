/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/


use crate::{
    error::BlockError,
    hashmapaug::Augmentable,
    Serializable, Deserializable,
};
use std::marker::PhantomData;
use ton_types::{
    error, fail, Result,
    BuilderData, Cell, IBitstring, SliceData
};


pub trait BinTreeType<X: Default + Serializable + Deserializable> {
    fn get_data(&self) -> SliceData;
    /// Returns item by key
    fn get(&self, mut key: SliceData) -> Result<Option<X>> {
        let mut cursor = self.get_data();
        while cursor.get_next_bit()? {
            if cursor.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                fail!(BlockError::InvalidData("Fork doesn't have two refs".to_string()))
            }
            match key.get_next_bit_opt() {
                Some(x) => cursor = cursor.reference(x)?.into(),
                _ => return Ok(None)
            }
        }
        if key.is_empty() {
            Ok(Some(X::construct_from(&mut cursor)?))
        } else {
            Ok(None)
        }
    }

    fn find(&self, mut key: SliceData) -> Result<Option<(SliceData, X)>> {
        let mut key_original = key.clone();
        let mut cursor = self.get_data();
        while cursor.get_next_bit()? {
            if cursor.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                fail!(BlockError::InvalidData("Fork doesn't have two refs".to_string()))
            }
            match key.get_next_bit_opt() {
                Some(x) => cursor = cursor.reference(x)?.into(),
                _ => return Ok(None) // key is shorter nothing to return
            }
        }
        key_original.shrink_by_remainder(&key);
        X::construct_from(&mut cursor).map(|x| Some((key_original, x)))
    }
    /// Iterates over all items
    fn iterate<F: FnMut(SliceData, X) -> Result<bool>>(&self, mut p: F) -> Result<bool> {
        iterate_internal(&mut self.get_data(), BuilderData::new(), &mut p)
    }
    /// Iterates over all items by pairs
    fn iterate_pairs<F: FnMut(BuilderData, X, Option<X>) -> Result<bool>>(&self, mut p: F) -> Result<bool> {
        iterate_internal_pairs(&mut self.get_data(), BuilderData::new(), None, &mut p, true)
    }
}

//////////////////////////////////
// helper functions
fn internal_merge<X, F>(data: &SliceData, mut key: SliceData, merger: F) -> Result<Option<BuilderData>>
where F: FnOnce(X, X) -> Result<X>, X: Default + Serializable + Deserializable
{
    if data.remaining_bits() != 1 && data.remaining_references() < 2 {
        return Ok(None)
    } else if let Some(x) = key.get_next_bit_opt() {
        if let Some(reference) = internal_merge(&data.reference(x)?.into(), key, merger)? {
            let mut cell = BuilderData::from_slice(data);
            cell.replace_reference_cell(x, reference.into());
            return Ok(Some(cell))
        }
    } else {
        let mut right_slice = SliceData::from(&data.reference(1)?);
        let mut left_slice = SliceData::from(&data.reference(0)?);
        if right_slice.get_next_bit()? | left_slice.get_next_bit()? {
            return Ok(None)
        }
        let right = X::construct_from(&mut right_slice)?;
        let left = X::construct_from(&mut left_slice)?;
        let merged = merger(left, right)?;
        let mut merged_cell = false.write_to_new_cell()?;
        merged.write_to(&mut merged_cell)?;
        return Ok(Some(merged_cell))
    }
    Ok(None)
}

fn internal_split<X, F>(data: &SliceData, mut key: SliceData, splitter: F) -> Result<Option<BuilderData>>
where F: FnOnce(X) -> Result<(X, X)>, X: Default + Serializable + Deserializable
{
    if data.remaining_bits() == 1 && data.get_bit(0)? { // bt_fork$1 {X:Type} left:^(BinTree X) right:^(BinTree X)
        if data.remaining_references() < 2 {
            return Ok(None)
        }
        if let Some(x) = key.get_next_bit_opt() {
            if let Some(reference) = internal_split(&data.reference(x)?.into(), key, splitter)? {
                let mut cell = BuilderData::from_slice(data);
                cell.replace_reference_cell(x, reference.into());
                return Ok(Some(cell))
            }
        }
    } else if key.is_empty() { // bt_leaf$0 {X:Type} leaf:X
        let mut leaf_slice = data.clone();
        if leaf_slice.get_next_bit()? {
            return Ok(None)
        }
        let (left, right) = splitter(X::construct_from(&mut leaf_slice)?)?;

        let mut left_cell = false.write_to_new_cell()?;
        left.write_to(&mut left_cell)?;

        let mut right_cell = false.write_to_new_cell()?;
        right.write_to(&mut right_cell)?;

        let mut cell = true.write_to_new_cell()?;
        cell.checked_append_reference(left_cell.into_cell()?)?;
        cell.checked_append_reference(right_cell.into_cell()?)?;

        return Ok(Some(cell))
    }
    Ok(None)
}

fn internal_update<X, F>(data: &SliceData, mut key: SliceData, mutator: F) -> Result<Option<BuilderData>>
where F: FnOnce(X) -> Result<X>, X: Default + Serializable + Deserializable
{
    if data.remaining_bits() == 1 && data.get_bit(0)? { // bt_fork$1 {X:Type} left:^(BinTree X) right:^(BinTree X)
        if data.remaining_references() < 2 {
            return Ok(None)
        }
        if let Some(x) = key.get_next_bit_opt() {
            if let Some(reference) = internal_update(&data.reference(x)?.into(), key, mutator)? {
                let mut cell = BuilderData::from_slice(data);
                cell.replace_reference_cell(x, reference.into_cell()?);
                return Ok(Some(cell))
            }
        }
    } else if key.is_empty() { // bt_leaf$0 {X:Type} leaf:X
        let mut leaf_slice = data.clone();
        if leaf_slice.get_next_bit()? {
            return Ok(None)
        }
        let value = mutator(X::construct_from(&mut leaf_slice)?)?;
        let mut cell = false.write_to_new_cell()?;
        value.write_to(&mut cell)?;
        return Ok(Some(cell))
    }
    Ok(None)
}

fn iterate_internal<X, F>(cursor: &mut SliceData, mut key: BuilderData, p: &mut F) -> Result<bool>
where 
    X: Default + Serializable + Deserializable, 
    F: FnMut(SliceData, X) -> Result<bool> {
    
    let result = if cursor.get_next_bit()? {
        let mut left_key = key.clone();
        left_key.append_bit_zero()?;
        key.append_bit_one()?;
        iterate_internal(&mut cursor.checked_drain_reference()?.into(), left_key, p)? &&
        iterate_internal(&mut cursor.checked_drain_reference()?.into(), key, p)?
    } else {
        return p(key.into_cell()?.into(), X::construct_from(cursor)?)
    };
    Ok(result)
}

fn iterate_internal_pairs<X, F>(
    cursor: &mut SliceData,
    mut key: BuilderData,
    sibling: Option<Cell>,
    func: &mut F,
    check_sibling: bool,
) -> Result<bool>
where 
    X: Default + Serializable + Deserializable, 
    F: FnMut(BuilderData, X, Option<X>) -> Result<bool> {
    
    let result = if cursor.get_next_bit()? {
        let mut left_key = key.clone();
        left_key.append_bit_zero()?;
        key.append_bit_one()?;
        let left = cursor.checked_drain_reference()?;
        let right = cursor.checked_drain_reference()?;
        iterate_internal_pairs(&mut left.clone().into(), left_key, Some(right.clone()), func, true)? &&
        iterate_internal_pairs(&mut right.into(), key, Some(left), func, false)?
    } else {
        let left = X::construct_from(cursor)?;
        match sibling {
            Some(cell) => {
                let mut cursor = SliceData::from(cell);
                if cursor.get_next_bit()? {
                    func(key, left, None)?
                } else if check_sibling {
                    func(key, left, Some(X::construct_from(&mut cursor)?))?
                } else {
                    true
                }
            }
            None => func(key, left, None)?
        }
    };
    Ok(result)
}

///
/// Implements a binary tree
/// 
/// TL-B scheme:
/// bt_leaf$0 {X:Type} leaf:X = BinTree X;
/// bt_fork$1 {X:Type} left:^(BinTree X) right:^(BinTree X) = BinTree X;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BinTree<X: Default + Serializable + Deserializable> {
    data: SliceData,
    phantom: PhantomData<X>
}

impl<X: Default + Serializable + Deserializable> BinTreeType<X> for BinTree<X> {
    fn get_data(&self) -> SliceData {
        self.data.clone()
    }
}

impl<X: Default + Serializable + Deserializable> BinTree<X> {
    /// Constructs new instance and put item
    pub fn with_item(value: &X) -> Result<Self> {
        let mut leaf = false.write_to_new_cell()?;
        value.write_to(&mut leaf)?;
        Ok(Self {
            data: leaf.into_cell()?.into(),
            phantom: PhantomData::<X>,
        })
    }

    /// Splits item by calling splitter function, returns false if item was not found
    pub fn split(
        &mut self,
        key: SliceData,
        splitter: impl FnOnce(X) -> Result<(X, X)>
    ) -> Result<bool> {
        if let Some(builder) = internal_split(&self.data, key, splitter)? {
            self.data = builder.into_cell()?.into();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Merge 2 items in fork by calling merger function, returns false if fork was not found
    pub fn merge(
        &mut self,
        key: SliceData,
        merger: impl FnOnce(X, X) -> Result<X>
    ) -> Result<bool> {
        if let Some(builder) = internal_merge(&self.data, key, merger)? {
            self.data = builder.into_cell()?.into();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Change item with given key calling mutator function, returns false if item was not found
    pub fn update(
        &mut self,
        key: SliceData,
        mutator: impl FnOnce(X) -> Result<X>
    ) -> Result<bool> {
        if let Some(builder) = internal_update(&self.data, key, mutator)? {
            self.data = builder.into_cell()?.into();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<X: Default + Serializable + Deserializable> Serializable for BinTree<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_references_and_data(&self.data)?;
        Ok(())
    }
}

impl<X: Default + Serializable + Deserializable> Deserializable for BinTree<X> {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.data = slice.clone();
        if slice.get_next_bit()? {
            slice.shrink_references(2..);
        } else {
            let mut x = X::default();
            x.read_from(slice)?;
        }
        self.data.shrink_by_remainder(slice);
        Ok(())
    }
}

///
/// Implementation of Augmented Binary Tree 
/// 
/// TL-B scheme:
/// bta_leaf$0 {X:Type} {Y:Type} leaf:X extra:Y = BinTreeAug X Y;
/// bta_fork$1 {X:Type} left:^(BinTreeAug X Y) right:^(BinTreeAug X Y) extra:Y = BinTreeAug X Y;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BinTreeAug<X: Default + Serializable + Deserializable, Y: Augmentable> {
    extra: Y,
    data: SliceData,
    phantom: PhantomData<X>,
}

impl<X: Default + Serializable + Deserializable, Y: Augmentable> BinTreeType<X> for BinTreeAug<X, Y> {
    fn get_data(&self) -> SliceData {
        self.data.clone()
    }
}

impl<X: Default + Serializable + Deserializable, Y: Augmentable> BinTreeAug<X, Y> {
    /// Constructs new instance and put item
    pub fn with_item(value: &X, aug: &Y) -> Result<Self> {
        let mut leaf = false.write_to_new_cell()?;
        value.write_to(&mut leaf)?;
        aug.write_to(&mut leaf)?;
        Ok(Self {
            extra: aug.clone(),
            data: leaf.into_cell()?.into(),
            phantom: PhantomData::<X>,
        })
    }
    pub fn set_extra(&mut self, _key: SliceData, _aug: &Y) -> bool {
        unimplemented!()
    }
    /// Returns item augment
    pub fn extra(&self, mut key: SliceData) -> Result<Option<Y>> {
        let mut cursor = self.data.clone();
        while cursor.get_next_bit()? {
            if cursor.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                return Ok(None)
            }
            match key.get_next_bit_opt() {
                Some(x) => cursor = cursor.reference(x)?.into(),
                None => return Ok(None)
            }
        }
        if key.is_empty() {
            X::skip(&mut cursor)?;
            Ok(Some(Y::construct_from(&mut cursor)?))
        } else {
            Ok(None)
        }
    }
    /// Returns root augment
    pub fn root_extra(&self) -> &Y {
        &self.extra
    }
    /// Splits item by key old item will be left
    pub fn split(&mut self, key: SliceData, value: &X, aug: &Y) -> Result<bool> {
        let mut cursor = self.data.clone();
        if Self::internal_split(&mut cursor, key, value, aug)? {
            self.data = cursor;
            self.extra.calc(aug)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    // /// Merges items in fork and put left instead
    // pub fn merge(&mut self, key: SliceData) -> bool {
    //     let mut builder = BuilderData::from_slice(&self.data);
    //     if builder.update_cell(internal_merge, key) {
    //         self.data = builder.into();
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn internal_split(slice: &mut SliceData, mut key: SliceData, value: &X, aug: &Y) -> Result<bool> {
        let original = slice.clone();
        if slice.get_next_bit()? { // bta_fork
            if slice.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                return Ok(false)
            }
            if let Some(x) = key.get_next_bit_opt() {
                let mut cursor = slice.reference(x)?.into();
                if Self::internal_split(&mut cursor, key, value, aug)? {
                    let mut cell = BuilderData::from_slice(&original);
                    cell.replace_reference_cell(x, cursor.into_cell());
                    let mut fork_aug = Y::construct_from(slice)?;
                    fork_aug.calc(aug)?;
                    fork_aug.write_to(&mut cell)?;
                    *slice = cell.into_cell()?.into();
                    return Ok(true)
                }
            }
        } else if key.is_empty() {
            X::skip(slice)?;
            let mut fork_aug = Y::construct_from(slice)?;
            fork_aug.calc(aug)?;
            let mut builder = true.write_to_new_cell()?; // bta_fork
            builder.append_reference_cell(original.into_cell());

            let mut cell = false.write_to_new_cell()?; // bta_leaf
            value.write_to(&mut cell)?;
            aug.write_to(&mut cell)?;
            builder.append_reference_cell(cell.into_cell()?);
            fork_aug.write_to(&mut builder)?;
            *slice = builder.into_cell()?.into();
            return Ok(true)
        }
        Ok(false)
    }
}

impl<X: Default + Serializable + Deserializable, Y: Augmentable> Serializable for BinTreeAug<X, Y> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_references_and_data(&self.data)?;
        Ok(())
    }
}

impl<X: Default + Serializable + Deserializable, Y: Augmentable> Deserializable for BinTreeAug<X, Y> {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.data = slice.clone();
        if slice.get_next_bit()? {
            slice.shrink_references(2..);
        } else {
            X::skip(slice)?;
        }
        self.extra.read_from(slice)?;
        self.data.shrink_by_remainder(slice);
        Ok(())
    }
}
