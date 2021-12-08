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

use std::convert::TryInto;
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use num::{BigInt, BigUint, bigint::Sign, One, Zero};
use num_traits::cast::ToPrimitive;
use ton_types::{
    error, fail,
    Result, BuilderData, Cell, CellType, IBitstring, HashmapE, HashmapType, SliceData, UInt256
};

use crate::{
    define_HashmapE,
    error::BlockError,
    hashmapaug::Augmentable,
    Serializable, Deserializable,
};


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
        #[derive( Eq, Clone, Debug)]
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

            pub fn sgn(&self) -> bool {
                self.0.sign() != Sign::NoSign
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
            fn write_to_cell(value: &BigInt) -> Result<BuilderData> {
                let len = Self::get_len(value);
                if len >= $N {
                    fail!("serialization of {} error {} >= {}", stringify!($varname), len, $N)
                }

                let mut cell = BuilderData::default();
                cell.append_bits(len, Self::get_len_len())?;
                let value = value.to_bytes_be().1;
                cell.append_raw(&value, len * 8)?;
                Ok(cell)
            }

            fn read_from_cell(cell: &mut SliceData) -> Result<BigInt> {
                let len = cell.get_next_int(Self::get_len_len())? as usize;
                if len >= $N {
                    fail!("deserialization of {} error {} >= {}", stringify!($varname), len, $N)
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
                write!(f, "{}", &self.0)
            }
        }
    };
    ( $varname:ident, $N:expr, $tt:ty ) => {
        #[derive( Eq, Clone, Debug, Default, Ord, PartialEq, PartialOrd)]
        pub struct $varname(pub $tt);

        impl $varname {
            pub const fn default() -> Self { $varname(0) }
            pub const fn new() -> Self { $varname(0) }
            pub fn get_len(&self) -> usize {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = ((0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8) as usize;
                bits as usize + bytes * 8
            }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = ((0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8) as usize;
                if bytes > $N {
                    fail!("cannot store {} grams, required {} bytes", self, bytes)
                }
                cell.append_bits(bytes, bits as usize)?;
                let be_bytes = self.0.to_be_bytes();
                cell.append_raw(&be_bytes[be_bytes.len() - bytes..], bytes * 8)?;
                Ok(())
            }
        }

        impl Deserializable for $varname {
            fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = slice.get_next_int(bits as usize)?;
                let max = std::mem::size_of::<$tt>();
                let mut bytes = slice.get_next_bytes(bytes as usize)?;
                bytes.reverse();
                bytes.resize_with(max, || 0);
                self.0 = <$tt>::from_le_bytes(bytes.as_slice().try_into()?);
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
                write!(f, "{}", &self.0)
            }
        }
    }
}
define_VarIntegerN!(Grams, 15, u128);
define_VarIntegerN!(VarUInteger32, 32, BigInt);
define_VarIntegerN!(VarUInteger3, 3, u32);
define_VarIntegerN!(VarUInteger7, 7, u64);

impl Augmentable for Grams {
    fn calc(&mut self, other: &Self) -> Result<()> {
        self.0 += &other.0;
        Ok(())
    }
}

impl AddSub for Grams {
    fn add(&mut self, other: &Grams) -> Result<()> {
        self.0 += &other.0;
        Ok(())
    }
    fn sub(&mut self, other: &Grams) -> Result<bool> {
        if self.0 >= other.0 {
            self.0 -= &other.0;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Grams {
    pub const fn shr(mut self, shr: u8) -> Self {
        self.0 >>= shr as usize;
        self
    }

    pub fn value(&self) -> BigInt {
        BigInt::from(self.0)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn one() -> Self {
        Self(1)
    }

    pub const fn sgn(&self) -> bool {
        false
    }

    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl From<BigInt> for Grams {
    fn from(value: BigInt) -> Self {
        Self::from(&value)
    }
}
impl From<&BigInt> for Grams {
    fn from(value: &BigInt) -> Self {
        match value.to_u128() {
            Some(value) => Self(value),
            None => {
                log::error!("Cannot convert BigInt {} to u128", value);
                Self(0)
            }
        }
    }
}
impl From<BigUint> for Grams {
    fn from(value: BigUint) -> Self {
        Self::from(&value)
    }
}
impl From<&BigUint> for Grams {
    fn from(value: &BigUint) -> Self {
        match value.to_u128() {
            Some(value) => Self(value),
            None => {
                log::error!("Cannot convert BigUint {} to u128", value);
                Self(0)
            }
        }
    }
}
impl From<u64> for Grams {
    fn from(value: u64) -> Self {
        Self(value as u128)
    }
}
impl From<i64> for Grams {
    fn from(value: i64) -> Self {
        Self(value as u128)
    }
}
impl From<u32> for Grams {
    fn from(value: u32) -> Self {
        Self(value as u128)
    }
}
impl From<i32> for Grams {
    fn from(value: i32) -> Self {
        Self(value as u128)
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
            pub const fn default() -> Self {
                Self(0)
            }
            pub fn from_u32(value: u32, max_value: u32) -> Result<Self> {
                if value > max_value {
                    fail!(BlockError::InvalidArg(
                        format!("value: {} must be <= {}", value, max_value) 
                    ))
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

        impl From<i64> for $varname {
            fn from(value: i64) -> Self {
                Self(value as u32)
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

define_HashmapE!{ExtraCurrencyCollection, 32, VarUInteger32}

impl From<HashmapE> for ExtraCurrencyCollection {
    fn from(other: HashmapE) -> Self {
        Self(other)
    }
}
/*
extra_currencies$_
    dict:(HashMapE 32 (VarUInteger 32))
= ExtraCurrencyCollection;

currencies$_
    grams: Grams
    other:ExtraCurrencyCollection
= CurrencyCollection;
*/
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyCollection {
    pub grams: Grams,
    pub other: ExtraCurrencyCollection,
}

impl Augmentable for CurrencyCollection {
    fn calc(&mut self, other: &Self) -> Result<()> {
        self.add(other)
    }
}

impl CurrencyCollection {
    pub const fn default() -> Self { Self::new() }
    pub const fn new() -> Self {
        Self::from_grams(Grams::default())
    }

    pub fn get_other(&self, key: u32) -> Result<Option<VarUInteger32>> {
        self.other.get(&key)
    }

    pub fn set_other(&mut self, key: u32, other: u128) -> Result<()> {
        self.set_other_ex(key, &VarUInteger32::from_two_u128(0, other)?)?;
        Ok(())
    }

    pub fn set_other_ex(&mut self, key: u32, other: &VarUInteger32) -> Result<()> {
        self.other.set(&key, other)?;
        Ok(())
    }

    pub fn other_as_hashmap(&self) -> HashmapE {
        self.other.0.clone()
    }

    pub const fn with_grams(grams: u64) -> Self {
        Self::from_grams(Grams(grams as u128))
    }

    pub const fn from_grams(grams: Grams) -> Self {
        CurrencyCollection {
            grams,
            other: ExtraCurrencyCollection::default()
        }
    }

    pub fn is_zero(&self) -> Result<bool> {
        if !self.grams.is_zero() {
            return Ok(false)
        }
        self.other.iterate(|value| Ok(value.is_zero()))
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
        if !self.grams.sub(&other.grams)? {
            return Ok(false)
        }
        other.other.iterate_with_keys(|key: u32, b| -> Result<bool> {
            if let Some(mut a) = self.other.get(&key)? {
                if a >= b {
                    a.sub(&b)?;
                    self.other.set(&key, &a)?;
                    return Ok(true)
                }
            }
            Ok(false) // coin not found in mine or amount is smaller - cannot subtract
        })
    }
    fn add(&mut self, other: &Self) -> Result<()> {
        self.grams.add(&other.grams)?;
        let mut result = self.other.clone();
        other.other.iterate_with_keys(|key: u32, b| -> Result<bool> {
            match self.other.get(&key)? {
                Some(mut a) => {
                    a.add(&b)?;
                    result.set(&key, &a)?;
                }
                None => {
                    result.set(&key, &b)?;
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
        write!(f, "{}", self.grams.0)?;
        if !self.other.is_empty() {
            let mut len = 0;
            write!(f, ", other: {{")?;
            self.other.iterate_with_keys(|key: u32, value| {
                len += 1;
                write!(f, " {} => {},", key, value.0)?;
                Ok(true)
            }).ok();
            write!(f, " count: {} }}", len)?;
        }
        Ok(())
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
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_u64()
    }
}

impl Serializable for u8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(*self)?;
        Ok(())
    }
}

impl Deserializable for u8 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_byte()
    }
}

impl Serializable for i32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i32(*self)?;
        Ok(())
    }
}

impl Deserializable for u32 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_u32()
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
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_i32()
    }
}

impl Serializable for i8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i8(*self)?;
        Ok(())
    }
}

impl Deserializable for i8 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_byte().map(|v| v as i8)
    }
}

impl Serializable for i16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_i16(*self)?;
        Ok(())
    }
}

impl Deserializable for i16 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_i16()
    }
}

impl Serializable for u16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u16(*self)?;
        Ok(())
    }
}

impl Deserializable for u16 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_u16()
    }
}

impl Serializable for bool {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bit_bool(*self)?;
        Ok(())
    }
}

impl Deserializable for bool {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        slice.get_next_bit()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InRefValue<X: Default + Deserializable + Serializable>(pub X);

impl<X: Default + Deserializable + Serializable> InRefValue<X> {
    pub fn new(inner: X) -> InRefValue<X> {
        InRefValue(inner)
    }
    pub fn inner(self) -> X {
        self.0
    }
}

impl<X: Default + Deserializable + Serializable> Deserializable for InRefValue<X> {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Self(X::construct_from_reference(slice)?))
    }
}

impl<X: Default + Deserializable + Serializable> Serializable for InRefValue<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_reference(self.0.serialize()?)?;
        Ok(())
    }
}

impl<X: Default + Deserializable> Deserializable for Arc<X> {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Arc::new(X::construct_from(slice)?))
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
            /// default const constructor
            pub const fn default() -> Self { Self::new() }
            /// default const constructor
            pub const fn new() -> Self {
                Self(HashmapE::with_hashmap($bit_len, None))
            }
            /// constructor with HashmapE root
            pub const fn with_hashmap(data: Option<Cell>) -> Self {
                Self(HashmapE::with_hashmap($bit_len, data))
            }
            pub fn root(&self) -> Option<&Cell> {
                self.0.data()
            }
            pub fn inner(self) -> HashmapE {
                self.0
            }
            /// Used for not empty Hashmaps
            pub fn read_hashmap_root(&mut self, slice: &mut SliceData) -> Result<()> {
                self.0.read_hashmap_root(slice)
            }
            /// Used for not empty Hashmaps
            pub fn write_hashmap_root(&self, cell: &mut BuilderData) -> Result<()> {
                self.0.write_hashmap_root(cell)
            }
            /// Return true if no items
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
            /// Calculates length
            pub fn len(&self) -> Result<usize> {
                self.0.len()
            }
            pub fn count(&self, max: usize) -> Result<usize> {
                self.0.count(max)
            }
            pub fn count_cells(&self, max: usize) -> Result<usize> {
                self.0.count_cells(max)
            }
            /// iterates items
            pub fn iterate<F>(&self, mut p: F) -> Result<bool>
            where F: FnMut($x_type) -> Result<bool> {
                self.0.iterate_slices(|_, ref mut slice| p(<$x_type>::construct_from(slice)?))
            }
            /// iterates items as raw slices
            pub fn iterate_slices<F>(&self, mut p: F) -> Result<bool>
            where F: FnMut(SliceData) -> Result<bool> {
                self.0.iterate_slices(|_, slice| p(slice))
            }
            /// iterates keys
            pub fn iterate_keys<K, F>(&self, mut p: F) -> Result<bool>
            where K: Default + Deserializable, F: FnMut(K) -> Result<bool> {
                self.0.iterate_slices(|mut key, _| p(
                    K::construct_from(&mut key)?
                ))
            }
            /// iterates items with keys
            pub fn iterate_with_keys<K, F>(&self, mut p: F) -> Result<bool>
            where K: Default + Deserializable, F: FnMut(K, $x_type) -> Result<bool> {
                self.0.iterate_slices(|ref mut key, ref mut slice| p(
                    K::construct_from(key)?,
                    <$x_type>::construct_from(slice)?
                ))
            }
            /// iterates items as slices with keys
            pub fn iterate_slices_with_keys<F>(&self, mut p: F) -> Result<bool>
            where F: FnMut(SliceData, SliceData) -> Result<bool> {
                self.0.iterate_slices(|key, slice| p(key, slice))
            }
            pub fn set<K: Serializable>(&mut self, key: &K, value: &$x_type) -> Result<()> {
                let key = key.serialize()?.into();
                let value = value.write_to_new_cell()?;
                self.0.set_builder(key, &value)?;
                Ok(())
            }
            pub fn setref<K: Serializable>(&mut self, key: &K, value: &Cell) -> Result<()> {
                let key = key.serialize()?.into();
                self.0.setref(key, value)?;
                Ok(())
            }
            pub fn add_key<K: Serializable>(&mut self, key: &K) -> Result<()> {
                let key = key.serialize()?.into();
                let value = BuilderData::default();
                self.0.set_builder(key, &value)?;
                Ok(())
            }
            pub fn get<K: Serializable>(&self, key: &K) -> Result<Option<$x_type>> {
                self.get_as_slice(key)?
                    .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            }
            pub fn get_as_slice<K: Serializable>(&self, key: &K) -> Result<Option<SliceData>> {
                let key = key.serialize()?.into();
                self.get_raw(key)
            }
            pub fn get_raw(&self, key: SliceData) -> Result<Option<SliceData>> {
                self.0.get(key)
            }
            pub fn remove<K: Serializable>(&mut self, key: &K) -> Result<bool> {
                let key = key.serialize()?.into();
                let leaf = self.0.remove(key)?;
                Ok(leaf.is_some())
            }
            pub fn check_key<K: Serializable>(&self, key: &K) -> Result<bool> {
                let key = key.serialize()?.into();
                self.0.get(key).map(|value| value.is_some())
            }
            pub fn export_vector(&self) -> Result<Vec<$x_type>> {
                let mut vec = Vec::new();
                self.0.iterate_slices(|_, ref mut slice| {
                    vec.push(<$x_type>::construct_from(slice)?);
                    Ok(true)
                })?;
                Ok(vec)
            }
            pub fn merge(&mut self, other: &Self, split_key: &SliceData) -> Result<()> {
                self.0.merge(&other.0, split_key)
            }
            pub fn split(&self, split_key: &SliceData) -> Result<(Self, Self)> {
                self.0.split(split_key).map(|(left, right)| (Self(left), Self(right)))
            }
            pub fn combine_with(&mut self, other: &Self) -> Result<bool> {
                self.0.combine_with(&other.0)
            }
            pub fn scan_diff<K, F>(&self, other: &Self, mut op: F) -> Result<bool>
            where K: Deserializable, F: FnMut(K, Option<$x_type>, Option<$x_type>) -> Result<bool> {
                self.0.scan_diff(&other.0, |mut key, value1, value2| {
                    let key = K::construct_from(&mut key)?;
                    let value1 = value1.map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()?;
                    let value2 = value2.map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()?;
                    op(key, value1, value2)
                })
            }

            pub fn filter<K, F>(&mut self, mut op: F) -> Result<()>
            where K: Deserializable, K : Serializable, F: FnMut(&K, &$x_type) -> Result<bool> {
                let mut other_tree = $varname(HashmapE::with_bit_len($bit_len));
                self.iterate_with_keys(&mut |key : K, value| {
                    if op(&key, &value)? {
                        other_tree.set(&key, &value)?;
                    };
                    return Ok(true);
                })?;
                *self = other_tree;
                Ok(())
            }

            pub fn export_keys<K: Deserializable>(&self) -> Result<Vec<K>> {
                let mut keys = Vec::new();
                self.iterate_keys(|key: K| {
                    keys.push(key);
                    Ok(true)
                })?;
                Ok(keys)
            }

            pub fn find_leaf<K: Deserializable + Serializable>(
                &self,
                key: &K,
                next: bool,
                eq: bool,
                signed_int: bool,
            ) -> Result<Option<(K, $x_type)>> {
                let key = key.serialize()?.into();
                if let Some((k, mut v)) = self.0.find_leaf(key, next, eq, signed_int, &mut 0)? {
                    // BuilderData, SliceData
                    let key = K::construct_from_cell(k.into_cell()?)?;
                    let value = <$x_type>::construct_from(&mut v)?;
                    Ok(Some((key, value)))
                } else {
                    Ok(None)
                }
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

#[derive(PartialEq, Copy, Clone, Debug, Eq, Default, Hash)]
pub struct UnixTime32(pub u32);

impl UnixTime32 {
    pub const fn default() -> Self { Self::new() }
    pub const fn new() -> Self {
        Self(0)
    }
    pub fn now() -> Self {
        UnixTime32 { 0: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32 }
    }
}

impl From<u32> for UnixTime32 {
    fn from(value: u32) -> Self {
        UnixTime32(value)
    }
}

#[allow(clippy::from_over_into)]
impl Into<u32> for UnixTime32 {
    fn into(self) -> u32 {
        self.0
    }
}

impl Serializable for UnixTime32 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()>{
        self.0.write_to(cell)
    } 
}

impl Deserializable for UnixTime32 {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Self(slice.get_next_u32()?))
    }
}

impl Display for UnixTime32 {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Default, Clone, Eq)]
pub struct ChildCell<T: Default + Serializable + Deserializable> {
    cell: Option<Cell>,
    phantom: PhantomData<T>
}

impl<T: Default + Serializable + Deserializable + Clone> ChildCell<T> {
    pub fn default() -> Self {
        Self {
            cell: None,
            phantom: PhantomData
        }
    }

    pub fn with_cell(cell: Cell) -> Self {
        Self {
            cell: Some(cell),
            phantom: PhantomData
        }
    }
    pub fn with_struct(s: &T) -> Result<Self> {
        Ok(
            ChildCell {
                cell: Some(s.serialize()?),
                phantom: PhantomData
            }
        )
    }

    pub fn write_struct(&mut self, s: &T) -> Result<()> {
        self.cell = Some(s.serialize()?);
        Ok(())
    }

    pub fn read_struct(&self) -> Result<T> {
        match self.cell.clone() {
            Some(cell) => {
                if cell.cell_type() == CellType::PrunedBranch {
                    fail!(
                        BlockError::PrunedCellAccess(std::any::type_name::<T>().into())
                    )
                }
                T::construct_from(&mut SliceData::from(cell))
            }
            None => Ok(T::default())
        }
    }

    pub fn read_from_reference(&mut self, slice: &mut SliceData) -> Result<()> {
        self.cell = Some(slice.checked_drain_reference()?);
        Ok(())
    }

    pub fn construct_from_reference(slice: &mut SliceData) -> Result<Self> {
        let cell = slice.checked_drain_reference()?;
        Ok(Self::with_cell(cell))
    }

    pub fn cell(&self)-> Cell {
        match self.cell.as_ref() {
            Some(cell) => cell.clone(),
            None => T::default().serialize().unwrap()
        }
    }

    pub fn set_cell(&mut self, cell: Cell) {
        self.cell = Some(cell);
    }

    pub fn hash(&self) -> UInt256 {
        match self.cell.as_ref() {
            Some(cell) => cell.repr_hash(),
            None => T::default().serialize().unwrap().repr_hash()
        }
    }
}

impl<T: Default + Serializable + Deserializable> PartialEq for ChildCell<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.cell == other.cell {
            return true
        }
        match (self.cell.as_ref(), other.cell.as_ref()) {
            (Some(cell), Some(other)) => cell.eq(other),
            (None, Some(cell)) |
            (Some(cell), None) => cell.eq(&T::default().serialize().unwrap()),
            (None, None) => true
        }
    }
}

impl<T: Default + Serializable + Deserializable> Serializable for ChildCell<T> {
    fn write_to(&self, builder: &mut BuilderData) -> Result<()> {
        if !builder.is_empty() {
            fail!(
                BlockError::InvalidArg("The `builder` must be empty".to_string())
            )
        }
        *builder = match self.cell.clone() {
            Some(cell) => BuilderData::from(cell),
            None => T::default().write_to_new_cell()?
        };
        Ok(())
    }
}
