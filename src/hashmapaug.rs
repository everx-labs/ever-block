/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
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

use crate::{
    error::BlockError,
    Serializable, Deserializable
};
use std::{fmt, marker::PhantomData};
use ton_types::{
    error, fail, Result,
    ExceptionCode, BuilderData, Cell, IBitstring, SliceData, HashmapType, Leaf, hm_label
};

type AugResult<Y> = Result<(Option<SliceData>, Y)>;

/// trait for types used as Augment to calc aug on forks
pub trait Augmentable: Clone + Default + Serializable + Deserializable {
    fn calc(&mut self, other: &Self) -> Result<()>;
}

/// Dummy for test purposes without augs
#[derive(Default, Clone, Eq, PartialEq, Debug)]
pub struct Dummy {}

impl Serializable for Dummy {
    fn write_to(&self, _cell: &mut BuilderData) -> Result<()>{
        Ok(())
    } 
}

impl Deserializable for Dummy {
    fn read_from(&mut self, _slice: &mut SliceData) -> Result<()>{
        Ok(())
    }
}

impl Augmentable for Dummy {
    fn calc(&mut self, _other: &Self) -> Result<()> {
        Ok(())
    }
}

#[macro_export]
macro_rules! define_HashmapAugE {
    ( $varname:ident, $bit_len:expr, $x_type:ty, $y_type:ty ) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $varname(HashmapAugE<$x_type, $y_type>);

        impl $varname {
            pub fn read_hashmap_root(&mut self, slice: &mut SliceData) -> Result<()> {
                self.0.read_hashmap_root(slice)
            }
            pub fn write_hashmap_root(&self, cell: &mut BuilderData) -> Result<()> {
                self.0.write_hashmap_root(cell)
            }
            pub fn len(&self) -> Result<usize> {
                self.0.len()
            }
            pub fn count(&self, max: usize) -> Result<usize> {
                self.0.count(max)
            }
            pub fn single(&self) -> Result<Option<$x_type>> {
                match self.0.single()? {
                    Some(ref mut slice) => Some(<$x_type>::construct_from(slice)).transpose(),
                    None => Ok(None)
                }
            }
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
            pub fn iterate<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut($x_type) -> Result<bool> {
                self.0.iterate(&mut |_, ref mut slice| p(<$x_type>::construct_from(slice)?))
            }
            pub fn iterate_with_keys<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut(SliceData, $x_type) -> Result<bool> {
                self.0.iterate(&mut |key, ref mut slice| p(key, <$x_type>::construct_from(slice)?))
            }
            pub fn iterate_with_keys_and_aug<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut(SliceData, $x_type, $y_type) -> Result<bool> {
                self.0.iterate_with_aug(&mut |key, ref mut slice, aug| p(key, <$x_type>::construct_from(slice)?, aug))
            }
            pub fn iterate_slices_with_keys_and_aug<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut(SliceData, SliceData, $y_type) -> Result<bool> {
                self.0.iterate_with_aug(&mut |key, slice, aug| p(key, slice, aug))
            }
            pub fn iterate_slices<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut(SliceData, SliceData) -> Result<bool> {
                self.0.iterate(p)
            }
            /// sets item to hashmapaug
            pub fn set<K: Serializable>(&mut self, key: &K, value: &$x_type, aug: &$y_type)
            -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                let value = value.write_to_new_cell()?.into();
                self.0.set(key, &value, aug).map(|_|())
            }
            /// sets item to hashmapaug as ref
            pub fn setref<K: Serializable>(&mut self, key: &K, value: &Cell, aug: &$y_type)
            -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                let value = value.write_to_new_cell()?.into();
                self.0.set(key, &value, aug).map(|_|())
            }
            /// sets serialized item to hashmapaug
            pub fn set_serialized<K: Serializable>(&mut self, key: &K, value: &SliceData, aug: &$y_type)
            -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                self.0.set(key, &value, aug).map(|_|())
            }
            /// returns item from hasmapaug
            pub fn get<K: Serializable>(&self, key: &K) -> Result<Option<$x_type>> {
                let key = key.write_to_new_cell()?.into();
                self.0.get(key)?
                    .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            }
            /// returns item with aug from hasmapaug
            pub fn get_with_aug<K: Serializable>(&self, key: &K) -> Result<Option<($x_type, $y_type)>> {
                let key = key.write_to_new_cell()?.into();
                match self.0.get_with_aug(key)? {
                    Some((mut slice, aug)) => Ok(Some((<$x_type>::construct_from(&mut slice)?, aug))),
                    _ => Ok(None)
                }
            }
            /// returns item from hasmapaug as slice
            pub fn get_as_slice<K: Serializable>(&self, key: &K) -> Result<Option<SliceData>> {
                let key = key.write_to_new_cell()?.into();
                self.0.get(key)
            }
            /// returns item from hasmapaug as cell
            pub fn get_as_cell<K: Serializable>(&self, key: &K) -> Result<Option<Cell>> {
                let key = key.write_to_new_cell()?.into();
                self.0.get(key)?.map(|slice| slice.reference(0)).transpose()
            }
            /// removes item from hashmapaug
            pub fn remove<K: Serializable>(&mut self, key: &K) -> Result<bool> {
                let key = key.write_to_new_cell()?.into();
                self.0.remove(key).map(|result| result.is_some())
            }
            // /// sets item to hashmapaug, returns previously set item
            // pub fn set<K: Serializable>(&mut self, key: &K, value: &$x_type, aug: &$y_type)
            // -> Result<Option<$x_type>> {
            //     let key = key.write_to_new_cell()?.into();
            //     let value = value.write_to_new_cell()?.into();
            //     self.0.set(key, value, aug)?
            //         .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            // }
            // /// removes item from hashmapaug
            // pub fn remove<K: Serializable>(&mut self, key: &K) -> Result<Option<$x_type>> {
            //     let key = key.write_to_new_cell()?.into();
            //     self.0.remove(key)?
            //         .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            // }
            // returns root augmentation
            pub fn root_extra(&self) -> &$y_type {
                self.0.root_extra()
            }
            /// splits tree by key for two trees
            pub fn split(&self, split_key: &SliceData) -> Result<($varname, $varname)> {
                self.0.split(split_key).map(|(left, right)| ($varname(left), $varname(right)))
            }
            /// merge self with other tree using merge key
            pub fn merge(&mut self, other: &$varname, merge_key: &SliceData) -> Result<()> {
                self.0.merge(&other.0, merge_key)
            }
            /// gets item with minimal key
            pub fn get_min<K: Deserializable>(&self, signed: bool) -> Result<Option<(K, $x_type)>> {
                match self.0.find_key(false, signed)? {
                    Some((key, mut val)) => {
                        let key = K::construct_from(&mut key.into())?;
                        let val = <$x_type>::construct_from(&mut val)?;
                        // let _aug = <$y_type>::construct_from(&mut val)?;
                        Ok(Some((key, val)))
                    },
                    None => Ok(None)
                }
            }
            /// gets item with maximal key
            pub fn get_max<K: Deserializable>(&self, signed: bool) -> Result<Option<(K, $x_type)>> {
                match self.0.find_key(true, signed)? {
                    Some((key, mut val)) => {
                        let key = K::construct_from(&mut key.into())?;
                        let val = <$x_type>::construct_from(&mut val)?;
                        // let _aug = <$y_type>::construct_from(&mut val)?;
                        Ok(Some((key, val)))
                    },
                    None => Ok(None)
                }
            }
            pub fn find_key<K: Deserializable>(&self, max: bool, signed: bool) -> Result<K> {
                match self.0.find_key(max, signed)? {
                    Some((mut key, _)) => K::construct_from(&mut key),
                    None => Ok(K::default())
                }
            }
            fn value_aug(slice: &mut SliceData) -> Result<($x_type, $y_type)> {
                let aug = <$y_type>::construct_from(slice)?;
                let val = <$x_type>::construct_from(slice)?;
                Ok((val, aug))
            }
            /// scans differences in two hashmaps
            pub fn scan_diff<K, F>(&self, other: &Self, mut op: F) -> Result<bool>
            where K: Deserializable, F: FnMut(K, Option<($x_type, $y_type)>, Option<($x_type, $y_type)>) -> Result<bool> {
                self.0.scan_diff(&other.0, |mut key, value_aug1, value_aug2| {
                    let key = K::construct_from(&mut key)?;
                    let value_aug1 = value_aug1.map(|ref mut slice| Self::value_aug(slice)).transpose()?;
                    let value_aug2 = value_aug2.map(|ref mut slice| Self::value_aug(slice)).transpose()?;
                    op(key, value_aug1, value_aug2)
                })
            }
            /// puts filtered elements to new dictionary
            pub fn filter<K, F>(&mut self, _op: F) -> Result<()>
            where K: Deserializable, F: FnMut(K, $x_type, $y_type) -> Result<bool> {
                Ok(())
            }
        }

        impl Default for $varname {
            fn default() -> Self {
                $varname(HashmapAugE::with_bit_len($bit_len))
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
                if let Some(root) = self.0.data() {
                    cell.append_bit_one()?;
                    cell.append_reference_cell(root.clone());
                } else {
                    cell.append_bit_zero()?;
                }
                self.0.root_extra().write_to(cell)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()>{
                self.0 = HashmapAugE::with_data($bit_len, slice)?;
                Ok(())
            }
        }
    }
}

///////////////////////////////////////////////
/// Length of key should not exceed bit_len
///
#[derive(Clone, Debug, Eq, PartialEq)] // cannot Default
pub struct HashmapAugE<X: Default + Deserializable + Serializable, Y: Augmentable> {
    phantom: PhantomData<X>, 
    extra: Y,
    bit_len: usize,
    data: Option<Cell>,
}

impl<X: Default + Deserializable + Serializable, Y: Augmentable> HashmapAugE<X, Y> {
    fn get_raw(&self, key: SliceData) -> Leaf {
        self.hashmap_get(key, &mut 0)
    }

    pub fn get(&self, key: SliceData) -> Leaf {
        match self.get_raw(key)? {
            Some(mut slice) => {
                Y::skip(&mut slice)?;
                Ok(Some(slice))
            }
            None => Ok(None)
        }
    }

    pub fn get_with_aug(&self, key: SliceData) -> Result<Option<(SliceData, Y)>> {
        match self.get_raw(key)? {
            Some(mut slice) => {
                let aug = Y::construct_from(&mut slice)?;
                Ok(Some((slice, aug)))
            }
            None => Ok(None)
        }
    }

    pub fn split(&self, key: &SliceData) -> Result<(Self, Self)> {
        let (left, right) = self.hashmap_split(key)?;
        Ok((Self::with_hashmap(self.bit_len, left)?, Self::with_hashmap(self.bit_len, right)?))
    }

    pub fn merge(&mut self, other: &Self, key: &SliceData) -> Result<()> {
        if self.bit_len != other.bit_len || key.remaining_bits() > self.bit_len {
            fail!("data in hashmaps do not correspond each other or key too long")
        }
        if self.data.is_none() {
            self.data = other.data.clone();
            self.extra = other.extra.clone();
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
    pub fn find_key(&self, max: bool, signed: bool) -> Result<Option<(SliceData, SliceData)>> {
        let result = match max {
            true  => ton_types::get_max::<Self>(self.data().cloned(), self.bit_len(), self.bit_len(), signed, &mut 0)?,
            false => ton_types::get_min::<Self>(self.data().cloned(), self.bit_len(), self.bit_len(), signed, &mut 0)?
        };
        match result {
            (Some(key), Some(val)) => Ok(Some((key.into(), val))),
            _ => Ok(None)
        }
    }
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl<X: Default + Deserializable + Serializable, Y: Augmentable> fmt::Display for HashmapAugE<X, Y> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data() {
            Some(cell) => write!(f, "HashmapAug: {}", cell),
            None => write!(f, "Empty HashmapAug"),
        }
    }
}

// hm_edge#_ {n:#} {X:Type} {l:#} {m:#} label:(HmLabel ~l n)
// {n = (~m) + l} node:(HashmapAugNode m X) = HashmapAug n X;
// hmn_leaf#_ {X:Type} value:X = HashmapAugNode 0 X;
// hmn_fork#_ {n:#} {X:Type} left:^(HashmapAug n X)
// right:^(HashmapAug n X) = HashmapAugNode (n+1) X;
impl<X: Default + Deserializable + Serializable, Y: Augmentable> HashmapType for HashmapAugE<X, Y> {
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

impl<X: Default + Deserializable + Serializable, Y: Augmentable> HashmapAugE<X, Y> {
    /// Checks if HashmapAugE is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }
    /// Constructs new HashmapAugE for bit_len keys
    pub fn with_bit_len(bit_len: usize) -> Self {
        Self {
            phantom: PhantomData::<X>,
            extra: Y::default(),
            bit_len,
            data: None,
        }
    }
    /// Deserialization from SliceData - just clone and set window
    pub fn with_data(bit_len: usize, slice: &mut SliceData) -> Result<Self> {
        let data;
        let extra = match slice.get_next_bit()? {
            true => {
                data = Some(slice.checked_drain_reference()?);
                Y::construct_from(slice)?
            }
            false => {
                data = None;
                Y::default()
            }
        };
        Ok(Self {
            phantom: PhantomData::<X>,
            extra,
            bit_len,
            data
        })
    }
    /// Constructs from cell, extracts total aug
    pub fn with_hashmap(bit_len: usize, data: Option<Cell>) -> Result<Self> {
        let extra = match data {
            Some(ref root) => Self::find_extra(&mut root.into(), bit_len)?,
            None => Y::default()
        };
        Ok(Self {
            phantom: PhantomData::<X>, 
            extra,
            bit_len,
            data,
        })
    }
    /// Serialization HashmapAug root of HashmapAugE to BuilderData - just append
    pub fn write_hashmap_root(&self, cell: &mut BuilderData) -> Result<()> {
        if let Some(root) = self.data() {
            cell.checked_append_references_and_data(&SliceData::from(root))?;
            self.root_extra().write_to(cell)?;
            Ok(())
        } else {
            fail!(BlockError::InvalidData("no reference".to_string()))
        }
    }
    /// deserialize not empty root
    pub fn read_hashmap_root(&mut self, slice: &mut SliceData) -> Result<()> {
        let mut root = slice.clone(); // copy to get as data
        let label = slice.get_label(self.bit_len)?;
        if label.remaining_bits() != self.bit_len { // fork
            slice.shrink_references(2..); // left, right
            self.extra = Y::construct_from(slice)?;
        } else { // single leaf as root
            self.extra = Y::construct_from(slice)?;
            let mut value = X::default();
            value.read_from(slice)?;
        }
        root.shrink_by_remainder(slice);

        self.data = Some(root.into_cell());
        Ok(())
    }
    /// Root augmentation
    pub fn root_extra(&self) -> &Y {
        &self.extra
    }
    /// removes object and returns old value as object
    pub fn remove(&mut self, mut _key: SliceData) -> Result<Option<SliceData>> {
        unimplemented!()
        // result?.map(|ref mut slice| {
        // }).ok_or_else(|| exception!(ExceptionCode::CellUnderflow))
    }
    /// return object if it is single in hashmap    
    pub fn single(&self) -> Result<Option<SliceData>> {
        if let Some(root) = self.data() {
            let mut slice = SliceData::from(root);
            let label = slice.get_label(self.bit_len)?;
            if label.remaining_bits() == self.bit_len {
                Y::skip(&mut slice)?;
                return Ok(Some(slice))
            }
        }
        Ok(None)
    }
    /// returns count of objects in tree - don't use it - try is_empty()
    pub fn len(&self) -> Result<usize> {
        let mut len = 0;
        self.iterate(&mut |_,_| {
            len += 1;
            Ok(true)
        })?;
        Ok(len)
    }
    /// returns count of objects in tree - it can be used as validate
    pub fn count(&self, max: usize) -> Result<usize> {
        let mut len = 0;
        self.iterate(&mut |_,_| {
            len += 1;
            Ok(len < max)
        })?;
        Ok(len)
    }
    /// iterates all objects in tree with callback function
    pub fn iterate<F> (&self, p: &mut F) -> Result<bool>
    where F: FnMut(SliceData, SliceData) -> Result<bool> {
        if let Some(root) = self.data() {
            Self::iterate_internal(
                &mut SliceData::from(root),
                BuilderData::default(),
                self.bit_len,
                &mut |k, v, _| p(k, v))
        } else {
            Ok(true)
        }
    }
    pub fn iterate_with_aug<F> (&self, p: &mut F) -> Result<bool>
    where F: FnMut(SliceData, SliceData, Y) -> Result<bool> {
        if let Some(root) = self.data() {
            Self::iterate_internal(
                &mut SliceData::from(root),
                BuilderData::default(),
                self.bit_len,
                p)
        } else {
            Ok(true)
        }
    }
    // internal recursive iterates all elements with callback function
    fn iterate_internal<F>(
        cursor: &mut SliceData, 
        mut key: BuilderData, 
        mut bit_len: usize, 
        found: &mut F
    ) -> Result<bool>
    where 
        F: FnMut(SliceData, SliceData, Y) -> Result<bool> 
    {
        let label = cursor.get_label(bit_len)?;
        let label_length = label.remaining_bits();
        if label_length < bit_len {
            bit_len -= label_length + 1;
            if cursor.remaining_references() < 2 {
                fail!(ExceptionCode::CellUnderflow);
            }
            for i in 0..2 {
                let mut key = key.clone();
                key.checked_append_references_and_data(&label)?;
                key.append_bit_bool(i != 0)?;
                let ref mut child = SliceData::from(cursor.reference(i)?);
                if !Self::iterate_internal(child, key, bit_len, found)? {
                    return Ok(false)
                }
            }
        } else if label_length == bit_len {
            key.checked_append_references_and_data(&label)?;
            let aug: Y = Y::construct_from(cursor)?;
            return found(key.into(), cursor.clone(), aug)
        } else {
            fail!(BlockError::InvalidData("label_length > bit_len".to_string()))
        }
        Ok(true)
    }
    /// Puts element to the tree
    pub fn set(&mut self, key: SliceData, leaf: &SliceData, extra: &Y) -> Result<Option<SliceData>> {
        let bit_len = self.bit_len;
        Self::check_key_fail(bit_len, &key)?;
        // ahme_empty$0 {n:#} {X:Type} {Y:Type} extra:Y = HashmapAugE n X Y;
        // ahme_root$1 {n:#} {X:Type} {Y:Type} root:^(HashmapAug n X Y) extra:Y = HashmapAugE n X Y;
        let result = if let Some(mut root) = self.data.clone() {
            let (result, extra) = Self::put_to_node(&mut root, bit_len, key, leaf, extra)?;
            self.extra = extra;
            self.data = Some(root);
            result
        } else {
            self.extra = (*extra).clone();
            self.data = Some(Self::make_cell_with_label_and_data(
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
            builder.append_reference(BuilderData::from(&reference));
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
        let existing_cell = Self::make_cell_with_label_and_data(
            label, length, is_leaf, slice)?;
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
        let another_cell = Self::make_cell_with_label_and_data(
            key, length, true, &Self::combine(extra, leaf)?
        )?;
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
}

