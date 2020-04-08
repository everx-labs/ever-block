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

use super::*;
use super::{Deserializable, Serializable};
use super::hashmapaug::Augmentable;
use num::{BigInt, Zero, One};
use num::bigint::Sign;
use {BuilderData, IBitstring, SliceData};
use std::cmp::Ordering;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{self, Display, Formatter};
use ExceptionCode;
use std::marker::PhantomData;


///
/// var_uint$_ {n:#} len:(#< n) value:(uint (len * 8)) = VarUInteger n;
/// 

/// var_int$_ {n:#} len:(#< n) value:(int (len * 8)) = VarInteger n;
/// nanograms$_ amount:(VarUInteger 16) = Grams;
/// 
/// If one wants to represent x nanograms, one selects an integer l < 16 such
/// that x < 2^8*l, and serializes first l as an unsigned 4-bit integer, then x itself
/// as an unsigned 8`-bit integer. Notice that four zero bits represent a zero
/// amount of Grams.

macro_rules! define_VarIntegerN {
    ( $varname:ident, $N:expr, BigInt ) => {
        #[derive( Eq, Hash, Clone, Debug)]
        pub struct $varname(pub BigInt);

        #[allow(dead_code)]
        impl $varname {

            fn get_len(value: &BigInt) -> usize {
                (value.bits() + 7) >> 3
            }

            pub fn value(&self) -> &BigInt {
                &self.0
            }

            pub fn value_mut(&mut self) -> &mut BigInt {
                &mut self.0
            }

            pub fn zero() -> Self {
                $varname(Zero::zero())
            }

            pub fn one() -> Self {
                $varname(One::one())
            }

            pub fn from_two_u128(hi: u128, lo: u128) -> Result<Self> {
                let val = (BigInt::from(hi) << 128) | BigInt::from(lo);
                Self::check_owerflow(&val)?;
                Ok($varname(val))
            }

            pub fn is_zero(&self) -> bool {
                self.0.is_zero()
            }

            fn check_owerflow(value: &BigInt) -> Result<()> {
                if Self::get_len(&value) > $N {
                    fail!(
                        BlockError::InvalidArg(
                            format!("value is bigger than {} bytes", $N)
                        )
                    )
                } else {
                    Ok(())
                }
            }

            // determine the size of the len field, using the formula from 3.3.4 VM 
            fn get_len_len() -> usize {
                let max_bits = ($N - 1) as f64;
                max_bits.log2() as usize + 1
            }

            // Interface to write value with type rule
            pub fn write_to_cell(value: &BigInt) -> Result<BuilderData> {
                let len = Self::get_len(value);
                if len >= $N {
                    fail!(ExceptionCode::RangeCheckError)
                }

                let mut cell = BuilderData::default();
                cell.append_bits(len, Self::get_len_len())?;
                let value = value.to_bytes_be().1;
                cell.append_raw(&value, len * 8)?;
                Ok(cell)
            }

            pub fn read_from_cell(cell: &mut SliceData) -> Result<BigInt> {
                let len = cell.get_next_int(Self::get_len_len())? as usize;
                if len >= $N {
                    fail!(ExceptionCode::RangeCheckError)
                }
                Ok(BigInt::from_bytes_be(Sign::Plus, &cell.get_next_bytes(len)?))
            }

        }

        impl<T: Into<BigInt>> From<T> for $varname {
            fn from(value: T) -> Self {
                let val = BigInt::from(value.into());
                Self::check_owerflow(&val).expect("Integer overflow");
                $varname(val)
            }
        }

        impl AddSub for $varname {
            fn add(&mut self, other: &$varname) -> Result<()> {
                self.0 += &other.0;
                Ok(())
            }
            fn sub(&mut self, other: &$varname) -> Result<bool> {
                if self.0 >= other.0 {
                    self.0 -= &other.0;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }

        impl Ord for $varname {
            fn cmp(&self, other: &$varname) -> Ordering {
                Ord::cmp(&self.0, &other.0)
            }
        }

        impl PartialOrd for $varname {
            fn partial_cmp(&self, other: &$varname) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for $varname {
            fn eq(&self, other: &$varname) -> bool {
                self.cmp(other) == Ordering::Equal
            }
        }


        impl Default for $varname {
            fn default() -> Self {
                $varname(BigInt::default())
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                let data = Self::write_to_cell(&self.0)?;
                cell.append_builder(&data)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
                self.0 = Self::read_from_cell(cell)?;
                Ok(())
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "vui{}[len = {}, value = {}]", $N, Self::get_len(&self.0), &self.0
                )
            }
        }
    };
    ( $varname:ident, $N:expr, $tt:ty ) => {
        #[derive( Eq, Hash, Clone, Debug, Default, Ord, PartialEq, PartialOrd)]
        pub struct $varname(pub $tt);

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = (0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8;
                if bytes > $N {
                    fail!(ExceptionCode::IntegerOverflow)
                }
                cell.append_bits(bytes as usize, bits as usize)?;
                cell.append_bits(self.0 as usize, bytes as usize * 8)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = slice.get_next_int(bits as usize)?;
                self.0 = slice.get_next_int(bytes as usize * 8)? as $tt;
                Ok(())
            }
        }

        impl From<$tt> for $varname {
            fn from(value: $tt) -> Self {
                Self(value)
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "vui{}[value = {}]", $N, &self.0
                )
            }
        }
    }
}

define_VarIntegerN!(Grams, 16, BigInt);
define_VarIntegerN!(VarUInteger32, 32, BigInt);
define_VarIntegerN!(VarUInteger3, 3, u32);
define_VarIntegerN!(VarUInteger7, 7, u64);

impl Augmentable for Grams {
    fn calc(&mut self, other: &Self) -> Result<()> {
        self.0 += &other.0;
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// munber ## N
/// n<=X
///
macro_rules! define_NumberN_up32bit {
    ( $varname:ident, $N:expr ) => {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
        pub struct $varname(pub u32);

        #[allow(dead_code)]
        impl $varname {
            pub fn from_u32(value: u32, max_value: u32) -> Result<Self> {
                if value > max_value {
                    fail!(
                        BlockError::InvalidArg(
                            format!("value: {} must be <= {}", value, max_value) 
                        )
                    )
                }
                Ok($varname(value))
            }

            pub fn get_max_len() -> usize {
                (((1 as u64) << $N) - 1) as usize
            }
        }

        impl Default for $varname {
            fn default() -> Self {
                $varname(0)
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                cell.append_bits(self.0 as usize, $N)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
                self.0 = cell.get_next_int($N)? as u32;
                Ok(())
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "vui{}[value = {}]", $N, self.0
                )
            }
        }
    };
}
define_NumberN_up32bit!(Number5, 5);
define_NumberN_up32bit!(Number8, 8);
define_NumberN_up32bit!(Number9, 9);
define_NumberN_up32bit!(Number12, 12);
define_NumberN_up32bit!(Number13, 13);
define_NumberN_up32bit!(Number16, 16);
define_NumberN_up32bit!(Number32, 32);

/*
extra_currencies$_
    dict:(HashMapE 32 (VarUInteger 32))
= ExtraCurrencyCollection;

currencies$_
    grams: Grams
    other:ExtraCurrencyCollection
= CurrencyCollection;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyCollection {
    pub grams: Grams,
    pub other: HashmapE
}

impl Default for CurrencyCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl Augmentable for CurrencyCollection {
    fn calc(&mut self, other: &Self) -> Result<()> {
        self.add(other)
    }
}

impl CurrencyCollection {
    pub fn new() -> Self {
        Self::from_grams(Grams::zero())
    }

    pub fn set_other(&mut self, key: u32, other: u128) {
        self.set_other_ex(key, &VarUInteger32::from_two_u128(0, other).unwrap())
    }

    pub fn set_other_ex(&mut self, key: u32, other: &VarUInteger32) {
        let key = key.write_to_new_cell().unwrap();
        self.other.set(key.into(), &other.write_to_new_cell().unwrap().into()).unwrap();
    }

    pub fn with_grams(grams: u64) -> Self {
        Self::from_grams(Grams(grams.into()))
    }

    pub fn from_grams(grams: Grams) -> Self {
        CurrencyCollection {
            grams,
            other: HashmapE::with_bit_len(32)
        }
    }

    pub fn is_zero(&self) -> Result<bool> {
        if !self.grams.is_zero() {
            return Ok(false)
        }
        self.other
            .iterate(&mut |_, ref mut slice| VarUInteger32::construct_from(slice).map(|value| value.is_zero()))
    }
}

impl Serializable for CurrencyCollection {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.grams.write_to(cell)?;
        self.other.write_to(cell)?;
        Ok(())
    }
}

impl Deserializable for CurrencyCollection {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()>{
        self.grams.read_from(cell)?;
        self.other.read_from(cell)?;
        Ok(())
    }
}

pub trait AddSub {
    fn sub(&mut self, other: &Self) -> Result<bool>;
    fn add(&mut self, other: &Self) -> Result<()>;
}

impl AddSub for CurrencyCollection {
    fn sub(&mut self, other: &Self) -> Result<bool> {
        if self.grams < other.grams {
            return Ok(false)
        }
        let mut result = self.other.clone();
        if other.other.iterate(&mut |key, ref mut slice| -> Result<bool> {
            let b = VarUInteger32::construct_from(slice)?;
            if let Some(ref mut slice) = self.other.get(key.clone())? {
                let mut a: VarUInteger32 = VarUInteger32::construct_from(slice)?;
                if a >= b {
                    a.sub(&b)?;
                    result.set(key, &a.write_to_new_cell()?.into())?;
                    return Ok(true)
                }
            }
            Ok(false) // coin not found in mine or amount is smaller - cannot subtract
        })? {
            self.other = result;
            self.grams.sub(&other.grams)
        } else {
            Ok(false)
        }
    }
    fn add(&mut self, other: &Self) -> Result<()> {
        self.grams.add(&other.grams)?;
        let mut result = self.other.clone();
        other.other.iterate(&mut |key, ref mut slice_b| -> Result<bool> {
            match self.other.get(key.clone())? {
                Some(ref mut slice_a) => {
                    let b = VarUInteger32::construct_from(slice_b)?;
                    let mut a: VarUInteger32 = VarUInteger32::construct_from(slice_a)?;
                    a.add(&b)?;
                    result.set(key, &a.write_to_new_cell()?.into())?;
                }
                None => {
                    result.set(key, slice_b)?;
                }
            }
            Ok(true)
        })?;
        self.other = result;
        Ok(())
    }
}

impl fmt::Display for CurrencyCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CurrencyCollection: Grams {}, other curencies:\n", self.grams)?;
        let mut len = 0;
        self.other.iterate(&mut |key, ref mut slice| -> Result<bool> {
            let value = VarUInteger32::construct_from(slice)?;
            write!(f, "key: {}, value: {}\n", key, value).unwrap();
            len += 1;
            Ok(true)
        }).unwrap();
        write!(f, "count: {}", len)
    }
}

impl From<u64> for CurrencyCollection {
    fn from(value: u64) -> Self {
        Self::with_grams(value)
    }
}

impl From<u32> for CurrencyCollection {
    fn from(value: u32) -> Self {
        Self::with_grams(value as u64)
    }
}

impl Serializable for u64 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u64(*self)?;
        Ok(())
    }
}

impl Deserializable for u64 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_u64()?;
        Ok(())
    }
}

impl Serializable for u8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(*self)?;
        Ok(())
    }
}

impl Deserializable for u8 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_byte()?;
        Ok(())
    }
}

impl Serializable for i32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i32(*self)?;
        Ok(())
    }
}

impl Deserializable for u32 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_u32()?;
        Ok(())
    }
}

impl Serializable for u32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u32(*self)?;
        Ok(())
    }
}

impl Serializable for u128 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u128(*self)?;
        Ok(())
    }
}

impl Deserializable for i32 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_i32()?;
        Ok(())
    }
}

impl Serializable for i8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i8(*self)?;
        Ok(())
    }
}

impl Deserializable for i8 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_byte()? as i8;
        Ok(())
    }
}

impl Serializable for i16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i16(*self)?;
        Ok(())
    }
}

impl Deserializable for i16 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_i16()?;
        Ok(())
    }
}

impl Serializable for u16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u16(*self)?;
        Ok(())
    }
}

impl Deserializable for u16 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_u16()?;
        Ok(())
    }
}

impl Serializable for bool {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bit_bool(*self)?;
        Ok(())
    }
}

impl Deserializable for bool {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = slice.get_next_bit()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InRefValue<X: Default + Deserializable + Serializable>(pub X);

impl<X: Default + Deserializable + Serializable> Deserializable for InRefValue<X> {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.0 = X::construct_from(&mut slice.checked_drain_reference()?.into())?;
        Ok(())
    }
}

impl<X: Default + Deserializable + Serializable> Serializable for InRefValue<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_reference(self.0.write_to_new_cell()?);
        Ok(())
    }
}

impl<X: Default + Deserializable> Deserializable for Arc<X> {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        *self = Arc::new(X::construct_from(slice)?);
        Ok(())
    }
}

impl<X: Serializable> Serializable for Arc<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.deref().write_to(cell)
    }
}

#[macro_export]
macro_rules! define_HashmapE {
    ( $varname:ident, $bit_len:expr, $x_type:ty ) => {
        #[derive(PartialEq, Clone, Debug, Eq)]
        pub struct $varname(HashmapE);

        #[allow(dead_code)]
        impl $varname {
            /// Used for not empty Hashmaps
            pub fn read_hashmap_root(&mut self, slice: &mut SliceData) -> Result<()> {
                self.0.read_hashmap_root(slice).map_err(|e| e.into())
            }
            /// Used for not empty Hashmaps
            pub fn write_hashmap_root(&self, cell: &mut BuilderData) -> Result<()> {
                self.0.write_hashmap_root(cell).map_err(|e| e.into())
            }
            /// Return true if no items
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
            /// Calculates length
            pub fn len(&self) -> Result<usize> {
                self.0.len().map_err(|e| e.into())
            }
            /// iterates items
            pub fn iterate<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut($x_type) -> Result<bool> {
                self.0.iterate(&mut |_, ref mut slice| p(<$x_type>::construct_from(slice)?))
            }
            /// iterates items as raw slices
            pub fn iterate_slices<F>(&self, p: &mut F) -> Result<bool>
            where F: FnMut(SliceData) -> Result<bool> {
                self.0.iterate(&mut |_, slice| p(slice))
            }
            /// iterates keys
            pub fn iterate_keys<K, F>(&self, p: &mut F) -> Result<bool>
            where K: Default + Deserializable, F: FnMut(K) -> Result<bool> {
                self.0.iterate(&mut |mut key, _| p(
                    K::construct_from(&mut key)?
                ))
            }
            /// iterates items with keys
            pub fn iterate_with_keys<K, F>(&self, p: &mut F) -> Result<bool>
            where K: Default + Deserializable, F: FnMut(K, $x_type) -> Result<bool> {
                self.0.iterate(&mut |ref mut key, ref mut slice| p(
                    K::construct_from(key)?,
                    <$x_type>::construct_from(slice)?
                ))
            }
            pub fn set<K: Serializable>(&mut self, key: &K, value: &$x_type) -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                let value = value.write_to_new_cell()?.into();
                self.0.set(key, &value).map(|_|()).map_err(|e| e.into())
            }
            pub fn setref<K: Serializable>(&mut self, key: &K, value: &Cell) -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                self.0.setref(key, value)?;
                Ok(())
            }
            pub fn get<K: Serializable>(&self, key: &K) -> Result<Option<$x_type>> {
                let key = key.write_to_new_cell()?.into();
                self.0.get(key)?
                    .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            }
            pub fn remove<K: Serializable>(&mut self, key: &K) -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                self.0.remove(key).map(|_|()).map_err(|e| e.into())
            }
            pub fn export_vector(&self) -> Result<Vec<$x_type>> {
                let mut vec = Vec::new();
                self.0.iterate(&mut |_, ref mut slice| {
                    vec.push(<$x_type>::construct_from(slice)?);
                    Ok(true)
                })?;
                Ok(vec)
            }
        }

        impl Default for $varname {
            fn default() -> Self {
                $varname(HashmapE::with_bit_len($bit_len))
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
                self.0.write_to(cell)
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()>{
                self.0.read_from(slice)
            }
        }
    }
}

#[macro_export]
macro_rules! define_HashmapE_empty_val {
    ( $varname:ident, $bit_len:expr ) => {
        #[derive(PartialEq, Clone, Debug, Eq)]
        pub struct $varname(HashmapE);

        #[allow(dead_code)]
        impl $varname {
            /// Used for not empty Hashmaps
            pub fn read_hashmap_root(&mut self, slice: &mut SliceData) -> Result<()> {
                self.0.read_hashmap_root(slice).map_err(|e| e.into())
            }
            /// Used for not empty Hashmaps
            pub fn write_hashmap_root(&self, cell: &mut BuilderData) -> Result<()> {
                self.0.write_hashmap_root(cell).map_err(|e| e.into())
            }

            pub fn len(&self) -> Result<usize> {
                self.0.len().map_err(|e| e.into())
            }
            /// iterates keys
            pub fn iterate_keys<K, F>(&self, p: &mut F) -> Result<bool>
            where K: Default + Deserializable, F: FnMut(K) -> Result<bool> {
                self.0.iterate(&mut |mut key, _| p(
                    K::construct_from(&mut key)?
                ))
            }
            pub fn add_key<K: Serializable>(&mut self, key: &K) -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                let value = SliceData::new_empty();
                self.0.set(key, &value).map(|_|()).map_err(|e| e.into())
            }
            pub fn remove<K: Serializable>(&mut self, key: &K) -> Result<()> {
                let key = key.write_to_new_cell()?.into();
                self.0.remove(key).map(|_|()).map_err(|e| e.into())
            }
            pub fn check_key<K: Serializable>(&self, key: &K) -> Result<bool> {
                let key = key.write_to_new_cell()?.into();
                self.0.get(key).map(|value| value.is_some()).map_err(|e| e.into())
            }
        }

        impl Default for $varname {
            fn default() -> Self {
                $varname(HashmapE::with_bit_len($bit_len))
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
                self.0.write_to(cell)
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()>{
                self.0.read_from(slice)
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Eq, Default)]
pub struct UnixTime32(pub u32);

impl UnixTime32 {
    pub fn now() -> Self {
        UnixTime32 { 0: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32 }
    }
}

impl From<u32> for UnixTime32 {
    fn from(value: u32) -> Self {
        UnixTime32(value)
    }
}

impl Serializable for UnixTime32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
        self.0.write_to(cell)
    } 
}

impl Deserializable for UnixTime32 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()>{
        self.0.read_from(slice)
    }
}

impl Display for UnixTime32 {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChildCell<T: Default + Serializable + Deserializable> {
    cell: Cell,
    phantom: PhantomData<T>
}

impl<T: Default + Serializable + Deserializable + Clone> ChildCell<T> {

    pub fn with_struct(s: &T) -> Result<Self> {
        Ok(
            ChildCell {
                cell: s.write_to_new_cell()?.into(),
                phantom: PhantomData
            }
        )
    }

    pub fn write_struct(&mut self, s: &T) -> Result<()> {
        self.cell = s.write_to_new_cell()?.into();
        Ok(())
    }

    pub fn read_struct(&self) -> Result<T> {
        if self.cell.cell_type() == CellType::PrunedBranch {
            fail!(
                BlockError::PrunedCellAccess(std::any::type_name::<T>().into())
            )
        }
        T::construct_from(&mut SliceData::from(self.cell.clone()))
    }

    pub fn cell(&self) -> &Cell {
        &self.cell
    }

    pub fn hash(&self) -> UInt256 {
        self.cell.repr_hash()
    }
}

impl<T: Default + Serializable + Deserializable + Clone> Default for ChildCell<T> {
    fn default() -> Self { 
        ChildCell::with_struct(&T::default()).unwrap()
    }
}

impl<T: Default + Serializable + Deserializable> Serializable for ChildCell<T> {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        if !builder.is_empty() {
            fail!(
                BlockError::InvalidArg("The `builder` must be empty".to_string())
            )
        }
        *builder = BuilderData::from(&self.cell);
        Ok(())
    }
}

impl<T: Default + Clone + Serializable + Deserializable> Deserializable for ChildCell<T> {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        if !slice.is_full_cell_slice() {
            fail!(
                BlockError::InvalidArg("The `slice` must have zero position".to_string())
            )
        }
        self.cell = slice.cell().clone();
        Ok(())
    }
}