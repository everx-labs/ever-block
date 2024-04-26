/*
* Copyright (C) 2019-2023 EverX. All Rights Reserved.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific EVERX DEV software governing permissions and
* limitations under the License.
*/

pub mod types;
pub use self::types::*;

pub mod cell;
pub use self::cell::*;

pub mod crypto;
pub use self::crypto::*;

pub mod dictionary;
pub use self::dictionary::*;

pub mod boc;
pub use boc::*;
use smallvec::SmallVec;

pub mod wrappers;
pub use self::wrappers::*;

pub mod bls;
pub use bls::*;

pub mod error;
pub use self::error::*;

pub mod blocks;
pub use self::blocks::*;

pub mod shard;
pub use self::shard::*;

pub mod accounts;
pub use self::accounts::*;

pub mod messages;
pub use self::messages::*;

pub mod inbound_messages;
pub use self::inbound_messages::*;

pub mod master;
pub use self::master::*;

pub mod envelope_message;
pub use self::envelope_message::*;

pub mod outbound_messages;
pub use self::outbound_messages::*;

pub mod shard_accounts;
pub use self::shard_accounts::*;

pub mod transactions;
pub use self::transactions::*;

pub mod bintree;
pub use self::bintree::*;

pub mod out_actions;
pub use self::out_actions::*;

pub mod merkle_proof;
pub use self::merkle_proof::*;

pub mod merkle_update;
pub use self::merkle_update::*;

pub mod validators;
pub use self::validators::*;

pub mod miscellaneous;
pub use self::miscellaneous::*;

pub mod signature;
pub use self::signature::*;

pub mod config_params;
pub use self::config_params::*;

use std::{collections::HashMap, hash::Hash};

include!("../common/src/info.rs");

pub trait Mask {
    fn bit(&self, bits: Self) -> bool;
    fn mask(&self, mask: Self) -> Self;
    fn any(&self, bits: Self) -> bool;
    fn non(&self, bits: Self) -> bool;
}

impl Mask for u8 {
    fn bit(&self, bits: Self) -> bool {
        (self & bits) == bits
    }
    fn mask(&self, mask: Self) -> u8 {
        self & mask
    }
    fn any(&self, bits: Self) -> bool {
        (self & bits) != 0
    }
    fn non(&self, bits: Self) -> bool {
        (self & bits) == 0
    }
}

pub trait GasConsumer {
    fn finalize_cell(&mut self, builder: BuilderData) -> Result<Cell>;
    fn load_cell(&mut self, cell: Cell) -> Result<SliceData>;
    fn finalize_cell_and_load(&mut self, builder: BuilderData) -> Result<SliceData>;
}

impl GasConsumer for u64 {
    fn finalize_cell(&mut self, builder: BuilderData) -> Result<Cell> {
        builder.into_cell()
    }
    fn load_cell(&mut self, cell: Cell) -> Result<SliceData> {
        SliceData::load_cell(cell)
    }
    fn finalize_cell_and_load(&mut self, builder: BuilderData) -> Result<SliceData> {
        SliceData::load_builder(builder)
    }
}

pub fn parse_slice_base(slice: &str, mut bits: usize, base: u32) -> Option<SmallVec<[u8; 128]>> {
    debug_assert!(bits < 8, "it is offset to get slice parsed");
    let mut acc = 0u8;
    let mut data = SmallVec::new();
    let mut completion_tag = false;
    for ch in slice.chars() {
        if completion_tag {
            return None
        }
        match ch.to_digit(base) {
            Some(x) => if bits < 4 {
                acc |= (x << (4 - bits)) as u8;
                bits += 4;
            } else {
                data.push(acc | (x as u8 >> (bits - 4)));
                acc = (x << (12 - bits)) as u8;
                bits -= 4;
            }
            None => match ch {
                '_' => completion_tag = true,
                _ => return None
            }
        }
    }
    if bits != 0 {
        if !completion_tag {
            acc |= 1 << (7 - bits);
        }
        if acc != 0 || data.is_empty() {
            data.push(acc);
        }
    } else if !completion_tag {
        data.push(0x80);
    }
    Some(data)
}

impl<K, V> Serializable for HashMap<K, V>
where
    K: Clone + Eq + Hash + Default + Deserializable + Serializable,
    V: Serializable
{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let bit_len = K::default().write_to_new_cell()?.length_in_bits();
        let mut dictionary = HashmapE::with_bit_len(bit_len);
        for (key, value) in self.iter() {
            let key = key.write_to_bitstring()?;
            dictionary.set_builder(key, &value.write_to_new_cell()?)?;
        }
        dictionary.write_to(cell)
    }
}

impl<K, V> Deserializable for HashMap<K, V>
where
    K: Eq + Hash + Default + Deserializable + Serializable,
    V: Deserializable + Default
{
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let bit_len = K::default().write_to_new_cell()?.length_in_bits();
        let mut dictionary = HashmapE::with_bit_len(bit_len);
        dictionary.read_hashmap_data(slice)?;
        dictionary.iterate_slices(|ref mut key, ref mut value| {
            let key = K::construct_from(key)?;
            let value = V::construct_from(value)?;
            self.insert(key, value);
            Ok(true)
        }).map(|_|())
    }
}

impl Serializable for HashmapE {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.write_hashmap_data(cell)?;
        Ok(())
    }
}

impl Deserializable for HashmapE {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.read_hashmap_data(slice)?;
        Ok(())
    }
}

pub trait Serializable {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()>;

    fn write_to_new_cell(&self) -> Result<BuilderData> {
        let mut cell = BuilderData::new();
        self.write_to(&mut cell)?;
        Ok(cell)
    }

    fn write_to_bitstring(&self) -> Result<SliceData> {
        let mut cell = BuilderData::new();
        self.write_to(&mut cell)?;
        SliceData::load_bitstring(cell)
    }

    fn write_to_bytes(&self) -> Result<Vec<u8>> {
        let cell = self.serialize()?;
        write_boc(&cell)
    }

    fn write_to_file(&self, file_name: impl AsRef<std::path::Path>) -> Result<()> {
        let bytes = self.write_to_bytes()?;
        match std::fs::write(file_name.as_ref(), bytes) {
            Ok(_) => Ok(()),
            Err(err) => fail!("failed to write to file {}: {}", file_name.as_ref().display(), err)
        }
    }

    fn serialize(&self) -> Result<Cell> {
        self.write_to_new_cell()?.into_cell()
    }
}

pub trait Deserializable: Default {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        let mut x = Self::default();
        x.read_from(slice)?;
        Ok(x)
    }
    fn construct_maybe_from(slice: &mut SliceData) -> Result<Option<Self>> {
        match slice.get_next_bit()? {
            true => Ok(Some(Self::construct_from(slice)?)),
            false => Ok(None)
        }
    }
    fn construct_from_full_cell(cell: Cell) -> Result<Self> {
        let mut slice = SliceData::load_cell(cell)?;
        let obj = Self::construct_from(&mut slice)?;
        if slice.remaining_bits() != 0 || slice.remaining_references() != 0 {
            fail!("cell is not empty after deserializing {}", std::any::type_name::<Self>().to_string())
        }
        Ok(obj)
    }
    fn construct_from_cell(cell: Cell) -> Result<Self> {
        Self::construct_from(&mut SliceData::load_cell(cell)?)
    }
    fn construct_from_reference(slice: &mut SliceData) -> Result<Self> {
        Self::construct_from_cell(slice.checked_drain_reference()?)
    }
    /// adapter for tests
    fn construct_from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::construct_from_cell(read_single_root_boc(bytes)?)
    }
    /// adapter for tests
    fn construct_from_file(file_name: impl AsRef<std::path::Path>) -> Result<Self> {
        match std::fs::read(file_name.as_ref()) {
            Ok(bytes) => Self::construct_from_bytes(&bytes),
            Err(err) => fail!("failed to read from file {}: {}", file_name.as_ref().display(), err)
        }
    }
    /// adapter for tests
    fn construct_from_base64(string: &str) -> Result<Self> {
        let bytes = base64_decode(string)?;
        Self::construct_from_bytes(&bytes)
    }
    // Override it to implement skipping
    fn skip(slice: &mut SliceData) -> Result<()> {
        Self::construct_from(slice)?;
        Ok(())
    }
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = Self::construct_from(slice)?;
        Ok(())
    }
    fn read_from_cell(&mut self, cell: Cell) -> Result<()> {
        self.read_from(&mut SliceData::load_cell(cell)?)
    }
    // fn read_from_reference(&mut self, slice: &mut SliceData) -> Result<()> {
    //     self.read_from_cell(slice.checked_drain_reference()?)
    // }
    fn invalid_tag(t: u32) -> BlockError {
        let s = std::any::type_name::<Self>().to_string();
        BlockError::InvalidConstructorTag { t, s }
    }
}

impl Deserializable for Cell {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = cell.checked_drain_reference()?;
        Ok(())
    }
}

impl Serializable for Cell {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_reference(self.clone())?;
        Ok(())
    }
}
/* for future use
impl Serializable for SliceData {
    fn write_to_new_cell(&self) -> Result<BuilderData> {
        Ok(BuilderData::from_slice(self))
    }
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_references_and_data(self)?;
        Ok(())
    }
}
*/

impl<T: Serializable> Serializable for Option<T> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if let Some(x) = self {
            cell.append_bit_one()?;
            x.write_to(cell)?;
        } else {
            cell.append_bit_zero()?;
        }
        Ok(())
    }
}

impl<T: Deserializable> Deserializable for Option<T> {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        match slice.get_next_bit_int()? {
            1 => Ok(Some(T::construct_from(slice)?)),
            _ => Ok(None)
        }
    }
}

pub trait GetRepresentationHash: Serializable + std::fmt::Debug {
    fn hash(&self) -> Result<UInt256> {
        match self.serialize() {
            Err(err) => {
                log::error!("err: {}, wrong hash calculation for {:?}", err, self);
                Err(err)
            }
            Ok(cell) => Ok(cell.repr_hash())
        }
    }
}

impl<T: Serializable + std::fmt::Debug> GetRepresentationHash for T {}

impl Deserializable for UInt256 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_hash()
    }
}

impl Serializable for UInt256 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_raw(self.as_slice(), 256)?;
        Ok(())
    }
}

impl Deserializable for AccountId {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = cell.get_next_slice(256)?;
        Ok(())
    }
}

impl Serializable for AccountId {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.remaining_bits() != 256 {
            fail!("account_id must contain 256 bits, but {}", self.remaining_bits())
        }
        cell.append_bytestring(self)?;
        Ok(())
    }
}

impl Deserializable for () {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        if cell.remaining_bits() == 0 && cell.remaining_references() == 0 {
            Ok(())
        } else {
            fail!("It must be True by TLB, but some data is present: {:x}", cell)
        }
    }
}

impl Serializable for () {
    fn write_to(&self, _cell: &mut BuilderData) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
pub fn write_read_and_assert<T>(s: T) -> T
where T: Serializable + Deserializable + Default + std::fmt::Debug + PartialEq {
    let cell = s.write_to_new_cell().unwrap();
    let mut slice = SliceData::load_builder(cell).unwrap();
    println!("slice: {}", slice);
    let s2 = T::construct_from(&mut slice).unwrap();
    s2.serialize().unwrap();
    pretty_assertions::assert_eq!(s, s2);
    s2
}
