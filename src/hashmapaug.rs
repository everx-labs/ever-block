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
    Serializable, Deserializable
};
use ton_types::{
    error, fail, Result, IBitstring, BuilderData, Cell, SliceData,
    ExceptionCode, HashmapType, Leaf,
};

pub type AugResult<Y> = Result<(Option<SliceData>, Y)>;

/// trait for types used as Augment to calc aug on forks
pub trait Augmentable: Clone + Default + Serializable + Deserializable {
    fn calc(&mut self, other: &Self) -> Result<()>;
}

/// How to continue hashmap's traverse operation
pub enum TraverseNextStep<R> {
    /// Continue traverse to the "0", "1" or both branches 
    VisitZero,
    VisitOne,
    VisitZeroOne,
    VisitOneZero,
    /// Stop traverse current branch
    Stop,
    /// End traverse and return given result from traverse function
    End(R)
}

///////////////////////////////////////////////
/// Length of key should not exceed bit_len
///
#[macro_export]
macro_rules! define_HashmapAugE {
    ( $varname:ident, $bit_len:expr, $k_type:ty, $x_type:ty, $y_type:ty ) => {

        #[derive(Clone, Debug, Eq, PartialEq)] // cannot Default
        pub struct $varname {
            extra: $y_type,
            bit_len: usize,
            data: Option<Cell>,
        }

        impl $varname {
            // pub fn dump<K: Deserializable + Debug, X: Deserializable + Debug, Y: Augmentable + Debug>(&self) {
            pub fn dump(&self) {
                self.iterate_slices(|ref mut key, ref mut value| {
                    dbg!(<$k_type>::construct_from(key).unwrap());
                    dbg!(<$y_type>::construct_from(value).unwrap());
                    dbg!(<$x_type>::construct_from(value).unwrap());
                    Ok(true)
                }).unwrap();
            }
            /// Constructs new HashmapAugE for bit_len keys
            pub fn with_bit_len(bit_len: usize) -> Self {
                Self {
                    extra: Default::default(),
                    bit_len,
                    data: None,
                }
            }
            /// Deserialization from SliceData - just clone and set window
            pub fn with_data(bit_len: usize, slice: &mut SliceData) -> Result<Self> {
                let data = match slice.get_next_bit()? {
                    true => Some(slice.checked_drain_reference()?),
                    false => None
                };
                let extra = <$y_type>::construct_from(slice)?;
                Ok(Self {
                    extra,
                    bit_len,
                    data
                })
            }
            /// Constructs from cell, extracts total aug
            pub fn with_hashmap(bit_len: usize, data: Option<Cell>) -> Result<Self> {
                let extra = match data {
                    Some(ref root) => Self::find_extra(&mut root.into(), bit_len)?,
                    None => Default::default()
                };
                Ok(Self {
                    extra,
                    bit_len,
                    data,
                })
            }
            /// split map by key
            pub fn split(&self, key: &SliceData) -> Result<(Self, Self)> {
                let (left, right) = self.hashmap_split(key)?;
                Ok((Self::with_hashmap(self.bit_len(), left)?, Self::with_hashmap(self.bit_len(), right)?))
            }
            /// merge maps
            pub fn merge(&mut self, other: &Self, key: &SliceData) -> Result<()> {
                if self.bit_len() != other.bit_len || key.remaining_bits() > self.bit_len() {
                    fail!("data in hashmaps do not correspond each other or key too long")
                }
                if self.data().is_none() {
                    *self.data_mut() = other.data.clone();
                    self.set_root_extra(other.extra.clone());
                } else {
                    let old_data = self.data().cloned();
                    self.extra.calc(&other.extra)?;
                    self.hashmap_merge(other, key)?;
                    if old_data.as_ref() == self.data() { // nothing was changed
                        return Ok(())
                    } else if let Some(root) = self.data() {
                        let mut builder = BuilderData::from(root);
                        self.extra.write_to(&mut builder)?;
                        *self.data_mut() = Some(builder.into());
                    } else {
                        fail!("after merge tree is empty")
                    }
                }
                Ok(())
            }
        }

        // hm_edge#_ {n:#} {X:Type} {l:#} {m:#} label:(HmLabel ~l n)
        // {n = (~m) + l} node:(HashmapAugNode m X) = HashmapAug n X;
        // hmn_leaf#_ {X:Type} value:X = HashmapAugNode 0 X;
        // hmn_fork#_ {n:#} {X:Type} left:^(HashmapAug n X)
        // right:^(HashmapAug n X) = HashmapAugNode (n+1) X;
        impl HashmapType for $varname {
            fn check_key(bit_len: usize, key: &SliceData) -> bool {
                bit_len == key.remaining_bits()
            }
            fn make_cell_with_label(key: SliceData, max: usize) -> Result<BuilderData> {
                hm_label(&key, max)
            }
            fn make_cell_with_label_and_data(
                key: SliceData, 
                max: usize, 
                _is_leaf: bool, 
                data: &SliceData
            ) -> Result<BuilderData> {
                let mut builder = hm_label(&key, max)?;
                // automatically adds reference with data if space is not enought
                if builder.checked_append_references_and_data(data).is_err() {
                    let reference = BuilderData::from_slice(data);
                    builder.append_reference(reference);
                }
                Ok(builder)
            }
            fn is_fork(slice: &mut SliceData) -> Result<bool> {
                Ok(slice.remaining_references() > 1)
            }
            fn is_leaf(_slice: &mut SliceData) -> bool {
                true
            }
            fn data(&self) -> Option<&Cell> {
                self.data.as_ref()
            }
            fn data_mut(&mut self) -> &mut Option<Cell> {
                &mut self.data
            }
            fn bit_len(&self) -> usize {
                self.bit_len
            }
            fn bit_len_mut(&mut self) -> &mut usize {
                &mut self.bit_len
            }
        }

        impl HashmapAugType<$k_type, $x_type, $y_type> for $varname {
            fn root_extra(&self) -> &$y_type {
                &self.extra
            }
            fn set_root_extra(&mut self, aug: $y_type) {
                self.extra = aug;
            }
        }

        impl $varname {
            /*
            /// removes item from hashmapaug
            fn remove(&mut self, key: &K) -> Result<bool> {
                let key = key.write_to_new_cell()?.into();
                self.remove(key).map(|result| result.is_some())
            }
            // /// removes item from hashmapaug
            // fn remove(&mut self, key: &K) -> Result<Option<$x_type>> {
            //     let key = key.write_to_new_cell()?.into();
            //     self.remove(key)?
            //         .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            // }
            */
            fn value_aug(slice: &mut SliceData) -> Result<($x_type, $y_type)> {
                let aug = <$y_type>::construct_from(slice)?;
                let val = <$x_type>::construct_from(slice)?;
                Ok((val, aug))
            }
            /// scans differences in two hashmaps
            pub fn scan_diff_with_aug<F>(&self, other: &Self, mut op: F) -> Result<bool>
            where F: FnMut($k_type, Option<($x_type, $y_type)>, Option<($x_type, $y_type)>) -> Result<bool> {
                self.scan_diff(&other, |mut key, value_aug1, value_aug2| {
                    let key = <$k_type>::construct_from(&mut key)?;
                    let value_aug1 = value_aug1.map(|ref mut slice| Self::value_aug(slice)).transpose()?;
                    let value_aug2 = value_aug2.map(|ref mut slice| Self::value_aug(slice)).transpose()?;
                    op(key, value_aug1, value_aug2)
                })
            }
        }
        impl Default for $varname {
            fn default() -> Self {
                Self {
                    extra: Default::default(),
                    bit_len: $bit_len,
                    data: None
                }
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
                if let Some(root) = self.data() {
                    cell.append_bit_one()?;
                    cell.append_reference_cell(root.clone());
                } else {
                    cell.append_bit_zero()?;
                }
                self.root_extra().write_to(cell)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()>{
                *self = $varname::with_data($bit_len, slice)?;
                Ok(())
            }
        }

        #[cfg_attr(rustfmt, rustfmt_skip)]
        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self.data() {
                    Some(cell) => write!(f, "HashmapAug: {}", cell),
                    None => write!(f, "Empty HashmapAug"),
                }
            }
        }
    }
}

pub trait HashmapAugType<K: Deserializable + Serializable, X: Deserializable + Serializable, Y: Augmentable>: HashmapType {
    fn root_extra(&self) -> &Y;
    fn set_root_extra(&mut self, aug: Y);
    fn get_serialized_raw(&self, key: SliceData) -> Leaf {
        self.hashmap_get(key, &mut 0)
    }
    fn get_serialized_as_slice(&self, key: SliceData) -> Result<Option<SliceData>> {
        self.get_serialized_raw(key)?.map(|mut slice| {
            Y::skip(&mut slice)?;
            Ok(slice)
        }).transpose()
    }
    fn get_serialized(&self, key: SliceData) -> Result<Option<X>> {
        self.get_serialized_as_slice(key)?.map(|mut slice| X::construct_from(&mut slice)).transpose()
    }
    fn get_serialized_with_aug(&self, key: SliceData) -> Result<Option<(X, Y)>> {
        self.get_serialized_raw(key)?.map(|mut slice| {
            let aug = Y::construct_from(&mut slice)?;
            Ok((X::construct_from(&mut slice)?, aug))
        }).transpose()
    }
    /// gets aug and item in combined slice
    fn get_raw(&self, key: &K) -> Leaf {
        let key = key.write_to_new_cell()?;
        self.get_serialized_raw(key.into())
    }
    /// get item as slice
    fn get_as_slice(&self, key: &K) -> Leaf {
        self.get_raw(key)?.map(|mut slice| {
            Y::skip(&mut slice)?;
            Ok(slice)
        }).transpose()
    }
    /// returns item from hasmapaug as cell
    fn get_as_cell(&self, key: &K) -> Result<Option<Cell>> {
        self.get_raw(key)?.map(|mut slice| {
            Y::skip(&mut slice)?;
            slice.reference(0)
        }).transpose()
    }
    /// get item and aug
    fn get(&self, key: &K) -> Result<Option<X>> {
        self.get_as_slice(key)?.map(|mut slice| X::construct_from(&mut slice)).transpose()
    }
    /// get item as slice and aug
    fn get_as_slice_with_aug(&self, key: &K) -> Result<Option<(SliceData, Y)>> {
        match self.get_raw(key)? {
            Some(mut slice) => {
                let aug = Y::construct_from(&mut slice)?;
                Ok(Some((slice, aug)))
            }
            None => Ok(None)
        }
    }
    /// get item and aug
    fn get_with_aug(&self, key: &K) -> Result<Option<(X, Y)>> {
        match self.get_raw(key)? {
            Some(mut slice) => {
                let aug = Y::construct_from(&mut slice)?;
                Ok(Some((X::construct_from(&mut slice)?, aug)))
            }
            None => Ok(None)
        }
    }
    /// sets item to hashmapaug
    fn set(&mut self, key: &K, value: &X, aug: &Y) -> Result<()> {
        let key = key.write_to_new_cell()?;
        let value = value.write_to_new_cell()?;
        self.set_serialized(key.into(), &value.into(), aug)?;
        Ok(())
    }
    /// sets item to hashmapaug as ref
    fn setref(&mut self, key: &K, value: &Cell, aug: &Y) -> Result<()> {
        let key = key.write_to_new_cell()?;
        let value = value.write_to_new_cell()?;
        self.set_serialized(key.into(), &value.into(), aug)?;
        Ok(())
    }

    fn find_key(&self, min: bool, signed: bool) -> Result<Option<(SliceData, SliceData)>> {
        match self.data() {
            Some(root) => {
                let mut path = BuilderData::new();
                let (next_index, index) = match (min, signed) {
                    (true, true) => (0, 1),
                    (true, false) => (0, 0),
                    (false, true) => (1, 0),
                    (false, false) => (1, 1),
                };
                let result = ton_types::get_min_max::<Self>(root.clone(), &mut path, self.bit_len(), next_index, index, &mut 0)?;
                Ok(result.map(|value| (path.into(), value)))
            }
            None => Ok(None)
        }
    }
    /// gets item with minimal key
    fn get_min(&self, signed: bool) -> Result<Option<(K, X)>> {
        match self.find_key(true, signed)? {
            Some((key, mut val)) => {
                let key = K::construct_from(&mut key.into())?;
                Y::skip(&mut val)?;
                let val = X::construct_from(&mut val)?;
                Ok(Some((key, val)))
            },
            None => Ok(None)
        }
    }
    /// gets item with maximal key
    fn get_max(&self, signed: bool) -> Result<Option<(K, X)>> {
        match self.find_key(false, signed)? {
            Some((key, mut val)) => {
                let key = K::construct_from(&mut key.into())?;
                Y::skip(&mut val)?;
                let val = X::construct_from(&mut val)?;
                Ok(Some((key, val)))
            },
            None => Ok(None)
        }
    }
    /// gets item with aug for minimal or maximal key
    fn get_minmax(&self, min: bool, signed: bool) -> Result<Option<(K, X, Y)>> {
        match self.find_key(min, signed)? {
            Some((key, mut val)) => {
                let key = K::construct_from(&mut key.into())?;
                let aug = Y::construct_from(&mut val)?;
                let val = X::construct_from(&mut val)?;
                Ok(Some((key, val, aug)))
            },
            None => Ok(None)
        }
    }

    // /// Checks if HashmapAugE is empty
    // fn is_empty(&self) -> bool {
    //     self.data().is_none()
    // }
    /// Serialization HashmapAug root of HashmapAugE to BuilderData - just append
    fn write_hashmap_root(&self, cell: &mut BuilderData) -> Result<()> {
        if let Some(root) = self.data() {
            cell.checked_append_references_and_data(&SliceData::from(root))?;
            Ok(())
        } else {
            fail!(BlockError::InvalidData("no reference".to_string()))
        }
    }
    /// deserialize not empty root
    fn read_hashmap_root(&mut self, slice: &mut SliceData) -> Result<()> {
        let mut root = slice.clone(); // copy to get as data
        let label = slice.get_label(self.bit_len())?;
        if label.remaining_bits() != self.bit_len() { // fork
            slice.shrink_references(2..); // left, right
            self.set_root_extra(Y::construct_from(slice)?);
        } else { // single leaf as root
            self.set_root_extra(Y::construct_from(slice)?);
            let mut value = X::default();
            value.read_from(slice)?;
        }
        root.shrink_by_remainder(slice);

        *self.data_mut() = Some(root.into_cell());
        Ok(())
    }
    /// return object slice if it is single in hashmap
    fn single(&self) -> Result<Option<SliceData>> {
        if let Some(root) = self.data() {
            let mut slice = SliceData::from(root);
            let label = slice.get_label(self.bit_len())?;
            if label.remaining_bits() == self.bit_len() {
                Y::skip(&mut slice)?;
                return Ok(Some(slice))
            }
        }
        Ok(None)
    }
    /// return object if it is single in hashmap
    fn single_value(&self) -> Result<Option<X>> {
        self.single()?.map(|ref mut slice| X::construct_from(slice)).transpose()
    }
    /// iterates all objects in tree with callback function
    fn iterate_slices_with_keys<F> (&self, mut p: F) -> Result<bool>
    where F: FnMut(K, SliceData) -> Result<bool> {
        self.iterate_slices(|mut key, mut slice| {
            let key = K::construct_from(&mut key)?;
            Y::skip(&mut slice)?;
            p(key, slice)
        })
    }
    /// iterates all objects as slices with keys and augs in tree with callback function
    fn iterate_slices_with_keys_and_aug<F> (&self, mut p: F) -> Result<bool>
    where F: FnMut(K, SliceData, Y) -> Result<bool> {
        self.iterate_slices(|mut key, mut slice| {
            let key = K::construct_from(&mut key)?;
            let aug = Y::construct_from(&mut slice)?;
            p(key, slice, aug)
        })
    }
    /// rename to iterate when method is removed in types
    /// iterates objects
    fn iterate_objects<F>(&self, mut p: F) -> Result<bool>
    where F: FnMut(X) -> Result<bool> {
        self.iterate_slices(|_, mut slice| {
            <Y>::skip(&mut slice)?;
            p(<X>::construct_from(&mut slice)?)
        })
    }
    /// iterate objects with keys
    fn iterate_with_keys<F>(&self, mut p: F) -> Result<bool>
    where F: FnMut(K, X) -> Result<bool> {
        self.iterate_slices(|mut key, mut slice| {
            let key = K::construct_from(&mut key)?;
            <Y>::skip(&mut slice)?;
            p(key, <X>::construct_from(&mut slice)?)
        })
    }
    /// iterate objects with keys and augs
    fn iterate_with_keys_and_aug<F>(&self, mut p: F) -> Result<bool>
    where F: FnMut(K, X, Y) -> Result<bool> {
        self.iterate_slices(|mut key, mut slice| {
            let key = K::construct_from(&mut key)?;
            let aug = <Y>::construct_from(&mut slice)?;
            p(key, <X>::construct_from(&mut slice)?, aug)
        })
    }
    /// Puts element to the tree
    fn set_serialized(&mut self, key: SliceData, leaf: &SliceData, extra: &Y) -> Result<Option<SliceData>> {
        let bit_len = self.bit_len();
        Self::check_key_fail(bit_len, &key)?;
        // ahme_empty$0 {n:#} {X:Type} {Y:Type} extra:Y = HashmapAugE n X Y;
        // ahme_root$1 {n:#} {X:Type} {Y:Type} root:^(HashmapAug n X Y) extra:Y = HashmapAugE n X Y;
        let result = if let Some(mut root) = self.data().cloned() {
            let (result, extra) = Self::put_to_node(&mut root, bit_len, key, leaf, extra)?;
            self.set_root_extra(extra);
            *self.data_mut() = Some(root);
            result
        } else {
            self.set_root_extra(extra.clone());
            *self.data_mut() = Some(Self::make_cell_with_label_and_data(
                key, bit_len, true, &Self::combine(extra, leaf)?
            )?.into());
            None
        };
        Ok(result)
    }
    // Puts element to required branch by first bit
    fn put_to_fork(
        slice: &mut SliceData,
        bit_len: usize,
        mut key: SliceData,
        leaf: &SliceData,
        extra: &Y
    ) -> AugResult<Y> {
        let next_index = key.get_next_bit_int()?;
        // ahmn_fork#_ {n:#} {X:Type} {Y:Type} left:^(HashmapAug n X Y) right:^(HashmapAug n X Y) extra:Y
        // = HashmapAugNode (n + 1) X Y;
        if slice.remaining_references() < 2 {
            fail!(
                BlockError::InvalidArg("slice must contain 2 or more references".to_string())
            )
        }
        let mut references = slice.shrink_references(2..); // left and right, drop extra
        assert_eq!(references.len(), 2);
        let mut fork_extra = Self::find_extra(&mut SliceData::from(references[1 - next_index].clone()), bit_len - 1)?;
        let (result, extra) = Self::put_to_node(&mut references[next_index], bit_len - 1, key, leaf, extra)?;
        fork_extra.calc(&extra)?;
        let mut builder = BuilderData::new();
        for reference in references.drain(..) {
            builder.checked_append_reference(reference)?;
        }
        fork_extra.write_to(&mut builder)?;
        *slice = builder.into();
        Ok((result, fork_extra))
    }
    // Continues or finishes search of place
    fn put_to_node(
        cell: &mut Cell,
        bit_len: usize,
        key: SliceData,
        leaf: &SliceData,
        extra: &Y
    ) -> AugResult<Y> {
        let result;
        let mut slice = SliceData::from(cell.clone());
        let label = slice.get_label(bit_len)?;
        let builder = if label == key {
            // replace existing leaf
            Y::skip(&mut slice)?; // skip extra
            let res_extra = extra.clone();
            result = Ok((Some(slice), res_extra));
            Self::make_cell_with_label_and_data(
                key, bit_len, true, &Self::combine(extra, leaf)?
            )?
        } else if label.is_empty() {
            // 1-bit edge just recalc extra
            result = Self::put_to_fork(&mut slice, bit_len, key, leaf, extra);
            Self::make_cell_with_label_and_data(label, bit_len, false, &slice)?
        } else {
            match SliceData::common_prefix(&label, &key) {
                (label_prefix, Some(label_remainder), Some(key_remainder)) => {
                    // new leaf insert 
                    let extra = Self::slice_edge(
                        &mut slice, bit_len,
                        label_prefix.unwrap_or_default(), label_remainder, key_remainder,
                        leaf, extra,
                    )?;
                    *cell = slice.into_cell();
                    return Ok((None, extra))
                }
                (Some(prefix), None, Some(key_remainder)) => {
                    // next iteration
                    result = Self::put_to_fork(
                        &mut slice, bit_len - prefix.remaining_bits(), key_remainder, leaf, extra
                    );
                    Self::make_cell_with_label_and_data(label, bit_len, false, &slice)?
                }
                error @ (_, _, _) => {
                    log::error!(
                        target: "tvm", 
                        "If we hit this, there's certainly a bug. {:?}. \
                         Passed: label: {}, key: {} ", 
                        error, label, key
                    );
                    fail!(ExceptionCode::FatalError)
                }
            }
        };
        *cell = builder.into();
        result
    }
    // Slices the edge and put new leaf
    fn slice_edge(
        slice: &mut SliceData, // slice without label
        bit_len: usize,
        prefix: SliceData,
        mut label: SliceData,
        mut key: SliceData,
        leaf: &SliceData,
        extra: &Y
    ) -> Result<Y> {
        key.shrink_data(1..);
        let label_bit = label.get_next_bit()?;
        let length = bit_len - 1 - prefix.remaining_bits();
        let is_leaf = length == label.remaining_bits();
        // Common prefix
        let mut builder = Self::make_cell_with_label(prefix, bit_len)?;
        // Remainder of tree
        let existing_cell = Self::make_cell_with_label_and_data(label, length, is_leaf, slice)?;
        // AugResult<Y> for fork
        if !is_leaf {
            if slice.remaining_references() < 2 {
                debug_assert!(false, "fork should have at least two refs");
            }
            slice.shrink_references(2..); // drain left, right
        }
        let mut fork_extra = Y::construct_from(slice)?;
        fork_extra.calc(extra)?;
        // Leaf for fork
        let another_cell = Self::make_cell_with_label_and_data(key, length, true, &Self::combine(extra, leaf)?)?;
        if !label_bit {
            builder.append_reference(existing_cell);
            builder.append_reference(another_cell);
        } else {
            builder.append_reference(another_cell);
            builder.append_reference(existing_cell);
        };
        fork_extra.write_to(&mut builder)?;
        *slice = builder.into();
        Ok(fork_extra)
    }
    // Combines extra with leaf
    fn combine(extra: &Y, leaf: &SliceData) -> Result<SliceData> {
        let mut builder = extra.write_to_new_cell()?;
        builder.checked_append_references_and_data(&leaf)?;
        Ok(builder.into())
    }
    // Gets label then get_extra
    fn find_extra(slice: &mut SliceData, bit_len: usize) -> Result<Y> {
        let label = slice.get_label(bit_len)?;
        if label.remaining_bits() != bit_len { // fork - drain left and right
            if slice.remaining_references() < 2 {
                fail!(ExceptionCode::CellUnderflow)
            }
            slice.shrink_references(2..);
        }
        Y::construct_from(slice)
    }
    // Calc new extra for fork
    fn calc_extra(left: &Cell, right: &Cell, bit_len: usize) -> Result<Y> {
        let mut aug = Self::find_extra(&mut left.into(), bit_len)?;
        aug.calc(&Self::find_extra(&mut right.into(), bit_len)?)?;
        Ok(aug)
    }
    //
    fn traverse<F, R>(&self, mut p: F) -> Result<Option<R>>
    where F: FnMut(&[u8], usize, Y, Option<X>) -> Result<TraverseNextStep<R>> {
        self.traverse_slices(|key_prefix, prefix_len, mut label| {
            let aug = Y::construct_from(&mut label)?;
            if prefix_len == self.bit_len() {
                let val = X::construct_from(&mut label)?;
                p(key_prefix, prefix_len, aug, Some(val))
            } else {
                p(key_prefix, prefix_len, aug, None)
            }
        })
    }
    // 
    fn traverse_slices<F, R>(&self, mut p: F) -> Result<Option<R>>
    where F: FnMut(&[u8], usize, SliceData) -> Result<TraverseNextStep<R>> {
        if let Some(root) = self.data() {
            Self::traverse_internal(
                &mut SliceData::from(root),
                BuilderData::default(),
                self.bit_len(),
                &mut |k, l, n| p(k, l, n))
        } else {
            Ok(None)
        }
    }
    /// recursive traverse tree and call callback function
    fn traverse_internal<F, R>(
        cursor: &mut SliceData, 
        mut key: BuilderData, 
        mut bit_len: usize, 
        callback: &mut F
    ) -> Result<Option<R>>
    where F: FnMut(&[u8], usize, SliceData) -> Result<crate::hashmapaug::TraverseNextStep<R>> {
        let label = cursor.get_label(bit_len)?;
        let label_length = label.remaining_bits();
        if label_length < bit_len {
            bit_len -= label_length + 1;

            let mut aug = cursor.clone();
            aug.checked_drain_reference()?;
            aug.checked_drain_reference()?;
            key.checked_append_references_and_data(&label)?;
            let to_visit = match callback(key.data(), key.length_in_bits(), aug)? {
                TraverseNextStep::Stop => return Ok(None),
                TraverseNextStep::End(r) => return Ok(Some(r)),
                TraverseNextStep::VisitZero => [Some(0), None],
                TraverseNextStep::VisitOne => [Some(1), None],
                TraverseNextStep::VisitZeroOne => [Some(0), Some(1)],
                TraverseNextStep::VisitOneZero => [Some(1), Some(0)],
            };
            for i in to_visit.iter() {
                if let Some(i) = i {
                    let mut key = key.clone();
                    key.append_bit_bool(*i != 0)?;
                    let ref mut child = SliceData::from(cursor.reference(*i)?);
                    if let Some(r) = Self::traverse_internal(child, key, bit_len, callback)? {
                        return Ok(Some(r))
                    }
                }
            }
        } else if label_length == bit_len {
            key.checked_append_references_and_data(&label)?;
            return match callback(key.data(), key.length_in_bits(), cursor.clone())? {
               TraverseNextStep::End(r) => Ok(Some(r)),
                _ => Ok(None),
            }
        } else {
            fail!(BlockError::InvalidData("label_length > bit_len".to_string()))
        }
        Ok(None)
    }
}

// TODO: move private operations here
trait HashmapAugOperations {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HashmapFilterResult {
    Cancel, // cancel traverse and skip changes
    Stop,   // cancel traverse and accept changes
    Remove, // remove element and continue
    Accept, // accept element and continue
}

pub trait HashmapAugRemover<K, X, Y>: HashmapAugType<K, X, Y>
where K: Deserializable + Serializable, X: Deserializable + Serializable, Y: Augmentable {
    fn filter<F: FnMut(K, X, Y) -> Result<HashmapFilterResult>> (&mut self, mut func: F) -> Result<()> {
        let bit_len = self.bit_len();
        let mut result = HashmapFilterResult::Accept;
        hashmap_filter::<Self, K, X, Y, F>(self.data_mut(), &mut BuilderData::default(), bit_len, &mut result, &mut func)?;
        if result == HashmapFilterResult::Stop {
            let aug = match self.data() {
                Some(root) => Self::find_extra(&mut root.into(), bit_len)?,
                None => Y::default()
            };
            self.set_root_extra(aug);
        }
        Ok(())
    }
}

fn hashmap_filter<T, K, X, Y, F>(
    cell_opt: &mut Option<Cell>,
    key: &mut BuilderData,
    mut bit_len: usize,
    result: &mut HashmapFilterResult,
    func: &mut F
) -> Result<(bool, Option<SliceData>)> // is_removed and remainder
where
    K: Deserializable + Serializable,
    X: Deserializable + Serializable,
    Y: Augmentable,
    T: HashmapAugType<K, X, Y> + ?Sized,
    F: FnMut(K, X, Y) -> Result<HashmapFilterResult>
{
    if *result == HashmapFilterResult::Cancel || *result == HashmapFilterResult::Stop {
        return Ok((false, None))
    }
    let mut cursor = match cell_opt {
        None => { // it only for root
            *result = HashmapFilterResult::Cancel;
            return Ok((false, None))
        }
        Some(cell) => SliceData::from(cell.clone()),
    };
    let key_length = key.length_in_bits();
    let this_bit_len = bit_len;
    *key = cursor.get_label_raw(&mut bit_len, std::mem::replace(key, BuilderData::default()))?;
    let remainder = cursor.clone();
    if bit_len == 0 {
        let key = K::construct_from(&mut key.into())?;
        let aug = Y::construct_from(&mut cursor)?;
        let val = X::construct_from(&mut cursor)?;
        let removed = match func(key, val, aug)? {
            HashmapFilterResult::Remove => {
                *cell_opt = None;
                true
            }
            HashmapFilterResult::Accept => false,
            new_result => {
                *result = new_result;
                false
            }
        };
        return Ok((removed, Some(remainder)))
    }
    let mut changed = false;
    bit_len -= 1;
    let mut next = vec![];
    for i in 0..2 {
        let mut key = key.clone();
        key.append_bit_bool(i == 1)?;
        let mut cell = Some(cursor.checked_drain_reference()?);
        let (removed, remainder) = hashmap_filter::<T, K, X, Y, F>(&mut cell, &mut key, bit_len, result, func)?;
        if *result == HashmapFilterResult::Cancel {
            return Ok((false, None))
        }
        changed |= removed;
        if let Some(cell) = cell {
            next.push((cell, key, remainder));
        }
    }
    if !changed {
        return Ok((false, Some(remainder)))
    } else if let Some((right, new_key, next_remainder)) = next.pop() {
        if let Some((left, _, _)) = next.pop() { // prepare new fork
            let aug = T::calc_extra(&left, &right, bit_len)?;
            let mut label = SliceData::from(key);
            label.move_by(key_length)?;
            let mut builder = T::make_cell_with_label(label, this_bit_len)?; // TODO: add make_fork_with_label
            let mut remainder = BuilderData::new();
            remainder.checked_append_reference(left)?;
            remainder.checked_append_reference(right)?;
            aug.write_to(&mut remainder)?;
            builder.append_builder(&remainder)?;
            *cell_opt = Some(builder.into());
            return Ok((true, Some(remainder.into())))
        } else { // replace fork with edge
            *key = new_key;
            let mut label = SliceData::from(key);
            label.move_by(key_length)?;
            let mut builder = T::make_cell_with_label(label, this_bit_len)?;
            if let Some(ref remainder) = next_remainder {
                builder.checked_append_references_and_data(remainder)?;
            } else {
                panic!("should be")
            }
            *cell_opt = Some(builder.into());
            return Ok((true, next_remainder))
        }
    } else {
        *cell_opt = None;
        return Ok((true, None))
    }
}

