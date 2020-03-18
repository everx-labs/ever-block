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

#![cfg_attr(feature = "ci_run", deny(warnings))]
//#![recursion_limit="128"] // needs for error_chain

// External
extern crate core;
#[macro_use]
extern crate log;
extern crate num;
extern crate sha2;

#[macro_use]
extern crate failure;

extern crate ton_types;

#[macro_use]
pub mod error;
pub use self::error::*;

#[macro_use]
pub mod types;
pub use self::types::*;

#[macro_use]
mod hashmapaug;
pub use self::hashmapaug::HashmapAugE;

pub mod blocks;
pub use self::blocks::*;

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

pub mod logical_time_generator;
pub use self::logical_time_generator::*;

pub mod validators;
pub use self::validators::*;

pub mod miscellaneous;
pub use self::miscellaneous::*;

pub mod signature;
pub use self::signature::*;

extern crate rand;
extern crate ed25519_dalek;
pub mod signed_block;
pub use self::signed_block::*;

pub mod config_params;
pub use self::config_params::*;

use std::collections::HashMap;
use std::hash::Hash;
use ton_types::{BuilderData, Cell, IBitstring, SliceData};

use ton_types::dictionary::{HashmapE, HashmapType};
use std::sync::Arc;

pub use ton_types::*;
pub use ton_types::types::*;

impl<K, V> Serializable for HashMap<K, V>
where
    K: Clone + Eq + Hash + Default + Deserializable + Serializable,
    V: Serializable
{
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        let bit_len = K::default().write_to_new_cell()?.length_in_bits();
        let mut dictionary = HashmapE::with_bit_len(bit_len);
        for (key, value) in self.iter() {
            let key = key.write_to_new_cell()?;
            dictionary.set(key.into(), &value.write_to_new_cell()?.into())?;
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
        dictionary.iterate(&mut |ref mut key, ref mut value| {
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
}

pub trait Deserializable {
    fn construct_from<X: Default + Deserializable>(slice: &mut SliceData) -> Result<X> {
        let mut x = X::default();
        x.read_from(slice)?;
        Ok(x)
    }
    // Override it to implement skipping
    fn skip<X: Default + Deserializable>(slice: &mut SliceData) -> Result<()> {
        X::construct_from::<X>(slice)?;
        Ok(())
    }
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()>; 
}

pub trait MaybeSerialize {
    fn write_maybe_to(&self, cell: &mut BuilderData) -> Result<()>;
}

impl Deserializable for Cell {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = cell.checked_drain_reference()?.clone();
        Ok(())
    }
}

impl Serializable for Cell {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(BuilderData::from(self));
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
impl<T: Serializable> MaybeSerialize for Option<T> {
    fn write_maybe_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            Some(x) => {
                cell.append_bit_one()?;
                x.write_to(cell)?;
            }
            None => {
                cell.append_bit_zero()?;
            }
        }
        Ok(())
    }
}

pub trait MaybeDeserialize {
    fn read_maybe_from<T: Deserializable + Default> (slice: &mut SliceData) -> Result<Option<T>> {
        match slice.get_next_bit_int() {
            Ok(1) => {
                let mut res = T::default();
                res.read_from(slice)?;
                Ok(Some(res))
            }
            Ok(0) => Ok(None),
            _ => failure::bail!(ExceptionCode::CellUnderflow)
        }
    }
}

impl<T: Deserializable> MaybeDeserialize for T {}

pub trait GetRepresentationHash: Serializable {
    fn hash(&self) -> Result<UInt256> {
        let cell: Cell = self.write_to_new_cell()?.into();
        Ok(cell.repr_hash())
    }
}

impl<T: Serializable> GetRepresentationHash for T {}

impl Deserializable for UInt256 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = UInt256::from(cell.get_next_bytes(32)?);
        Ok(())
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
            failure::bail!(BlockError::TvmException(ExceptionCode::CellUnderflow))
        }
        cell.append_bytestring(&self)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrueTlbType;

impl Default for TrueTlbType{
    fn default() -> Self {
        TrueTlbType
    }
}

impl Serializable for TrueTlbType{
    fn write_to(&self, _cell: &mut BuilderData) -> Result<()> {
        Ok(())
    }    
}

impl Deserializable for TrueTlbType {
    fn read_from(&mut self, _cell: &mut SliceData) -> Result<()> {
        Ok(())        
    }
}

pub fn id_from_key(key: &ed25519_dalek::PublicKey) -> u64 {
    let bytes = key.to_bytes();
    u64::from_be_bytes([ 
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

