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


use super::*;
use ton_types::SliceData;
use std::marker::PhantomData;
use self::hashmapaug::Augmentable;


pub trait BinTreeType<X: Default + Serializable + Deserializable> {
    fn get_data(&self) -> SliceData;
    /// Returns item by key
    fn get(&self, mut key: SliceData) -> BlockResult<Option<X>> {
        let mut cursor = self.get_data();
        while cursor.get_next_bit()? {
            if cursor.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                bail!(BlockErrorKind::InvalidData("Fork doesn't have two refs".into()))
            }
            match key.get_next_bit_int() {
                Ok(x) => cursor = cursor.reference(x).expect("There must be at least two links").into(),
                _ => return Ok(None)
            }
        }
        if key.is_empty() {
            X::construct_from(&mut cursor).map(|x| Some(x))
        } else {
            Ok(None)
        }
    }
    /// Iterates over all items
    fn iterate<F: FnMut(SliceData, X) -> BlockResult<bool>>(&self, p: &mut F) -> BlockResult<bool> {
        iterate_internal(BuilderData::new(), &mut self.get_data(), p)
    }
}

//////////////////////////////////
// helper functions
fn internal_merge(
    data: &mut Vec<u8>, bits: &mut usize, children: &mut Vec<Arc<CellData>>, mut key: SliceData
) -> bool {
    if *bits != 1 || children.len() < 2 {
        false
    } else if let Ok(x) = key.get_next_bit_int() {
        let mut child = BuilderData::from(&children.remove(x));
        let result = child.update_cell(internal_merge, key);
        children.insert(x, child.into());
        result
    } else {
        let mut child = BuilderData::from(&children.remove(0));
        child.cell_data(data, bits, children);
        true
    }
}

fn internal_split<X: Default + Serializable + Deserializable>(
    data: &mut Vec<u8>, bits: &mut usize, children: &mut Vec<Arc<CellData>>, (mut key, value): (SliceData, &X)
) -> BlockResult<bool> {
    if *bits == 1 && data.as_slice() == [0x80] { // bt_fork$1 {X:Type} left:^(BinTree X) right:^(BinTree X)
        if children.len() < 2 {
            return Ok(false)
        }
        if let Ok(x) = key.get_next_bit_int() {
            let mut child = BuilderData::from(&children.remove(x));
            let result = child.update_cell(internal_split, (key, value));
            children.insert(x, child.into());
            return result
        }
    } else if key.is_empty() { // bt_leaf$0 {X:Type} leaf:X
        let leaf = BuilderData::with_raw_and_refs(std::mem::replace(data, vec![0x80]), *bits, children.drain(..))?;
        *bits = 1;
        children.push(leaf.into()); // existing always left
        let mut cell = BuilderData::with_raw(vec![0], 1)?;
        value.write_to(&mut cell)?;
        children.push(cell.into()); // new value right
        return Ok(true)
    }
    Ok(false)
}

fn append_key_bit(mut key: BuilderData, bit: bool) -> Result<BuilderData, ExceptionCode> {
    key.append_bit_bool(bit)?;
    Ok(key)
}

fn iterate_internal<X, F>(key: BuilderData, slice: &mut SliceData, p: &mut F) -> BlockResult<bool>
where X: Default + Serializable + Deserializable, F: FnMut(SliceData, X) -> BlockResult<bool> {
    let result = if slice.get_next_bit()? {
        iterate_internal(append_key_bit(key.clone(), false)?, &mut slice.checked_drain_reference()?.into(), p)? &&
        iterate_internal(append_key_bit(key, true)?, &mut slice.checked_drain_reference()?.into(), p)?
    } else {
        return p(key.into(), X::construct_from(slice)?)
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
    pub fn with_item(value: &X) -> Self {
        let mut leaf = BuilderData::with_raw(vec![0x00], 1).unwrap();
        value.write_to(&mut leaf).expect("should be ok");
        Self {
            data: leaf.into(),
            phantom: PhantomData::<X>,
        }
    }
    /// Splits item by key old item will be left
    pub fn split(&mut self, key: SliceData, value: &X) -> BlockResult<bool> {
        let mut builder = BuilderData::from_slice(&self.data);
        if builder.update_cell(internal_split, (key, value))? {
            self.data = builder.into();
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Merges items in fork and put left instead
    pub fn merge(&mut self, key: SliceData) -> bool {
        let mut builder = BuilderData::from_slice(&self.data);
        if builder.update_cell(internal_merge, key) {
            self.data = builder.into();
            true
        } else {
            false
        }
    }
}

impl<X: Default + Serializable + Deserializable> Serializable for BinTree<X> {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.checked_append_references_and_data(&self.data)?;
        Ok(())
    }
}

impl<X: Default + Serializable + Deserializable> Deserializable for BinTree<X> {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
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
    pub fn with_item(value: &X, aug: &Y) -> Self {
        let mut leaf = BuilderData::with_raw(vec![0x00], 1).unwrap();
        value.write_to(&mut leaf).expect("should be ok");
        aug.write_to(&mut leaf).expect("should be ok");
        Self {
            extra: aug.clone(),
            data: leaf.into(),
            phantom: PhantomData::<X>,
        }
    }
    pub fn set_extra(&mut self, _key: SliceData, _aug: &Y) -> bool {
        unimplemented!()
    }
    /// Returns item augment
    pub fn extra(&self, mut key: SliceData) -> BlockResult<Option<Y>> {
        let mut cursor = self.data.clone();
        while cursor.get_next_bit()? {
            if cursor.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                return Ok(None)
            }
            match key.get_next_bit_int() {
                Ok(x) => cursor = cursor.reference(x).expect("There must be at least two links").into(),
                Err(_) => return Ok(None)
            }
        }
        if key.is_empty() {
            X::skip::<X>(&mut cursor)?;
            Y::construct_from(&mut cursor).map(|extra| Some(extra))
        } else {
            Ok(None)
        }
    }
    /// Returns root augment
    pub fn root_extra(&self) -> &Y {
        &self.extra
    }
    /// Splits item by key old item will be left
    pub fn split(&mut self, key: SliceData, value: &X, aug: &Y) -> BlockResult<bool> {
        let mut cursor = self.data.clone();
        if Self::internal_split(&mut cursor, key, value, aug)? {
            self.data = cursor;
            self.extra.calc(aug)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Merges items in fork and put left instead
    pub fn merge(&mut self, key: SliceData) -> bool {
        let mut builder = BuilderData::from_slice(&self.data);
        if builder.update_cell(internal_merge, key) {
            self.data = builder.into();
            true
        } else {
            false
        }
    }

    //////////////////////////////////
    // helper functions
    fn internal_split_next(
        data: &mut Vec<u8>, bits: &mut usize, children: &mut Vec<Arc<CellData>>, (mut key, value, aug): (SliceData, &X, &Y)
    ) -> BlockResult<bool> {
        if let Ok(x) = key.get_next_bit_int() {
            let mut cursor = children[x].clone().into();
            if Self::internal_split(&mut cursor, key, value, aug)? {
               children[x] = cursor.into_cell();
                *data = vec![0x80];
                *bits = 1;
                return Ok(true)
            }
        }
        Ok(false)
    }
    fn internal_split(slice: &mut SliceData, key: SliceData, value: &X, aug: &Y) -> BlockResult<bool> {
        let mut cell = BuilderData::from_slice(&slice);
        if slice.get_next_bit()? {
            if slice.remaining_references() < 2 {
                // fork doesn't have two refs - bad data
                return Ok(false)
            }
            if cell.update_cell(Self::internal_split_next, (key, value, aug))? {
                let mut fork_aug = Y::construct_from::<Y>(slice)?;
                fork_aug.calc(aug)?;
                fork_aug.write_to(&mut cell)?;
                *slice = cell.into();
                return Ok(true)
            }
        } else if key.is_empty() {
            X::skip::<X>(slice)?;
            let mut fork_aug = Y::construct_from::<Y>(slice)?;
            fork_aug.calc(aug)?;
            let mut builder = BuilderData::with_bitstring(vec![0xC0])?;
            builder.append_reference(cell);
            let mut cell = BuilderData::with_raw(vec![0], 1)?;
            value.write_to(&mut cell)?;
            aug.write_to(&mut cell)?;
            builder.append_reference(cell);
            fork_aug.write_to(&mut builder)?;
            *slice = builder.into();
            return Ok(true)
        }
        Ok(false)
    }
}

impl<X: Default + Serializable + Deserializable, Y: Augmentable> Serializable for BinTreeAug<X, Y> {
    fn write_to(&self, cell: &mut BuilderData) -> BlockResult<()> {
        cell.checked_append_references_and_data(&self.data)?;
        Ok(())
    }
}

impl<X: Default + Serializable + Deserializable, Y: Augmentable> Deserializable for BinTreeAug<X, Y> {
    fn read_from(&mut self, slice: &mut SliceData) -> BlockResult<()> {
        self.data = slice.clone();
        if slice.get_next_bit()? {
            slice.shrink_references(2..);
        } else {
            X::skip::<X>(slice)?;
        }
        self.extra.read_from(slice)?;
        self.data.shrink_by_remainder(slice);
        Ok(())
    }
}
