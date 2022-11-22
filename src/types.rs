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
use std::fmt::{self, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;
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
                (value.bits() as usize + 7) >> 3
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
                Self::check_overflow(&val)?;
                Ok($varname(val))
            }

            pub fn is_zero(&self) -> bool {
                self.0.is_zero()
            }

            fn check_overflow(value: &BigInt) -> Result<()> {
                match Self::get_len(&value) > $N {
                    true => fail!("value {} is bigger than {} bytes", value, $N),
                    false => Ok(())
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
                Self::check_overflow(&val).expect("Integer overflow");
                $varname(val)
            }
        }

        impl FromStr for $varname {
            type Err = failure::Error;

            fn from_str(string: &str) -> Result<Self> {
                let result = if let Some(stripped) = string.strip_prefix("0x") {
                    BigInt::parse_bytes(stripped.as_bytes(), 16)
                } else {
                    BigInt::parse_bytes(string.as_bytes(), 10)
                };
                match result {
                    Some(val) => {
                        Self::check_overflow(&val)?;
                        Ok(Self(val))
                    }
                    None => fail!("cannot parse {} for {}", stringify!($varname), string)
                }
            }
        }

        impl AddSub for $varname {
            fn add(&mut self, other: &Self) -> Result<bool> {
                if let Some(result) = self.0.checked_add(&other.0) {
                    if let Err(err) = Self::check_overflow(&result) {
                        log::warn!("{} + {} overflow: {:?}", self, other, err);
                        Ok(false)
                    } else {
                        self.0 = result;
                        Ok(true)
                    }
                } else {
                    Ok(false)
                }
            }
            fn sub(&mut self, other: &Self) -> Result<bool> {
                if let Some(result) = self.0.checked_sub(&other.0) {
                    self.0 = result;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }

        impl Ord for $varname {
            fn cmp(&self, other: &$varname) -> std::cmp::Ordering {
                Ord::cmp(&self.0, &other.0)
            }
        }

        impl PartialOrd for $varname {
            fn partial_cmp(&self, other: &$varname) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for $varname {
            fn eq(&self, other: &$varname) -> bool {
                self.cmp(other) == std::cmp::Ordering::Equal
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
        #[derive( Eq, Copy, Clone, Debug, Default, Ord, PartialEq, PartialOrd)]
        pub struct $varname(pub $tt);

        impl $varname {
            pub const fn default() -> Self { $varname(0) }
            pub fn new(value: $tt) -> Result<Self> {
                Self::check_overflow(&value)?;
                Ok(Self(value))
            }
            pub const fn zero() -> Self { Self(0) }
            pub const fn one() -> Self { Self(1) }
            pub const fn sgn(&self) -> bool { false }
            pub const fn is_zero(&self) -> bool { self.0 == 0 }
            pub fn add_checked(&mut self, other: $tt) -> bool {
                if let Some(result) = self.0.checked_add(other) {
                    if let Err(err) = Self::check_overflow(&result) {
                        log::warn!("{} + {} overflow: {:?}", self, other, err);
                        false
                    } else {
                        self.0 = result;
                        true
                    }
                } else {
                    false
                }
            }
            pub fn sub_checked(&mut self, other: $tt) -> bool {
                if let Some(result) = self.0.checked_sub(other) {
                    self.0 = result;
                    true
                } else {
                    false
                }
            }
            fn check_overflow(value: &$tt) -> Result<()> {
                let bytes = ((0 as $tt).leading_zeros() / 8 - value.leading_zeros() / 8) as usize;
                match bytes > $N {
                    true => fail!("value {} is bigger than {} bytes", value, $N),
                    false => Ok(())
                }
            }
            pub fn get_len(&self) -> usize {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = ((0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8) as usize;
                bits as usize + bytes * 8
            }
            pub const fn inner(&self) -> $tt { self.0 }
            pub const fn as_u32(&self) -> u32 { self.0 as u32 }
            pub const fn as_u64(&self) -> u64 { self.0 as u64 }
            pub const fn as_u128(&self) -> u128 { self.0 as u128 }
        }

        impl Serializable for $varname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                let bits = 8 - ($N as u8).leading_zeros();
                let bytes = ((0 as $tt).leading_zeros() / 8 - self.0.leading_zeros() / 8) as usize;
                if bytes > $N {
                    fail!("cannot store {} {}, required {} bytes", self, stringify!($varname), bytes)
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

        impl AddSub for $varname {
            fn add(&mut self, other: &Self) -> Result<bool> {
                Ok(self.add_checked(other.0))
            }
            fn sub(&mut self, other: &Self) -> Result<bool> {
                Ok(self.sub_checked(other.0))
            }
        }
        impl From<u64> for $varname {
            fn from(value: u64) -> Self {
                Self(value as $tt)
            }
        }
        impl From<i64> for $varname {
            fn from(value: i64) -> Self {
                Self(value as $tt)
            }
        }
        impl From<u16> for $varname {
            fn from(value: u16) -> Self {
                Self(value as $tt)
            }
        }
        impl From<u32> for $varname {
            fn from(value: u32) -> Self {
                Self(value as $tt)
            }
        }
        impl From<i32> for $varname {
            fn from(value: i32) -> Self {
                Self(value as $tt)
            }
        }
        impl From<u128> for $varname {
            fn from(value: u128) -> Self {
                Self(value as $tt)
            }
        }
        impl From<usize> for $varname {
            fn from(value: usize) -> Self {
                Self(value as $tt)
            }
        }

        impl fmt::Display for $varname {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", &self.0)
            }
        }

        impl std::ops::Mul<$tt> for $varname {
            type Output = Self;
            fn mul(mut self, rhs: $tt) -> Self::Output {
                self.0 *= rhs;
                self
            }
        }

        impl std::ops::MulAssign<$tt> for $varname {
            fn mul_assign(&mut self, rhs: $tt) {
                self.0 *= rhs;
                
            }
        }

        impl std::ops::Mul for $varname {
            type Output = Self;
            fn mul(mut self, rhs: Self) -> Self::Output {
                self.0 *= rhs.0;
                self
            }
        }

        impl std::ops::MulAssign for $varname {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl std::ops::Div<$tt> for $varname {
            type Output = Self;
            fn div(mut self, rhs: $tt) -> Self::Output {
                self.0 /= rhs;
                self
            }
        }

        impl std::ops::DivAssign<$tt> for $varname {
            fn div_assign(&mut self, rhs: $tt) {
                self.0 /= rhs;
                
            }
        }

        impl std::ops::Div for $varname {
            type Output = Self;
            fn div(mut self, rhs: Self) -> Self::Output {
                self.0 /= rhs.0;
                self
            }
        }

        impl std::ops::DivAssign for $varname {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl std::ops::Shr<u8> for $varname {
            type Output = Self;
            fn shr(mut self, rhs: u8) -> Self::Output {
                self.0 >>= rhs;
                self
            }
        }

        impl std::ops::ShrAssign<u8> for $varname {
            fn shr_assign(&mut self, rhs: u8) {
                self.0 >>= rhs;
            }
        }

        impl std::ops::Shl<u8> for $varname {
            type Output = Self;
            fn shl(mut self, rhs: u8) -> Self{
                self.0 <<= rhs;
                self
            }
        }

        impl std::ops::ShlAssign<u8> for $varname {
            fn shl_assign(&mut self, rhs: u8) {
                self.0 <<= rhs;
            }
        }

        impl num::CheckedAdd for $varname {
            fn checked_add(&self, rhs: &Self) -> Option<Self> {
                if let Some(result) = self.0.checked_add(rhs.0) {
                    if Self::check_overflow(&result).is_ok() {
                        return Some(Self(result))
                    }
                }
                None
            }
        }

        impl std::ops::Add<$tt> for $varname {
            type Output = Self;
            fn add(mut self, rhs: $tt) -> Self{
                self.0 += rhs;
                self
            }
        }

        impl std::ops::AddAssign<$tt> for $varname {
            fn add_assign(&mut self, rhs: $tt) {
                self.0 += rhs;
            }
        }

        impl std::ops::Add for $varname {
            type Output = Self;
            fn add(mut self, rhs: Self) -> Self{
                self.0 += rhs.0;
                self
            }
        }

        impl std::ops::AddAssign for $varname {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl num::CheckedSub for $varname {
            fn checked_sub(&self, rhs: &Self) -> Option<Self> {
                Some(Self(self.0.checked_sub(rhs.0)?))
            }
        }

        impl std::ops::Sub<$tt> for $varname {
            type Output = Self;
            fn sub(mut self, rhs: $tt) -> Self{
                self.0 -= rhs;
                self
            }
        }

        impl std::ops::SubAssign<$tt> for $varname {
            fn sub_assign(&mut self, rhs: $tt) {
                self.0 -= rhs;
            }
        }

        impl std::ops::Sub for $varname {
            type Output = Self;
            fn sub(mut self, rhs: Self) -> Self{
                self.0 -= rhs.0;
                self
            }
        }

        impl std::ops::SubAssign for $varname {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
    }
}
define_VarIntegerN!(Grams, 15, u128);
define_VarIntegerN!(VarUInteger32, 32, BigInt);
define_VarIntegerN!(VarUInteger3, 3, u32);
define_VarIntegerN!(VarUInteger7, 7, u64);

impl Augmentable for Grams {
    fn calc(&mut self, other: &Self) -> Result<bool> {
        self.add(other)
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

impl FromStr for Grams {
    type Err = failure::Error;

    fn from_str(string: &str) -> Result<Self> {
        if let Some(stripped) = string.strip_prefix("0x") {
            Ok(Self(u128::from_str_radix(stripped, 16)?))
        } else {
            Ok(Self(string.parse::<u128>()?))
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
///
/// number ## N
/// n<=X
///
macro_rules! define_NumberN_up32bit {
    ( $varname:ident, $N:expr ) => {
        #[derive(PartialEq, Eq, Hash, Clone, Debug, Default, PartialOrd, Ord)]
        pub struct $varname(pub u32);

        #[allow(dead_code)]
        impl $varname {
            pub const fn default() -> Self {
                Self(0)
            }
            pub fn new_checked(value: u32, max_value: u32) -> Result<Self> {
                if value > max_value {
                    fail!(BlockError::InvalidArg(
                        format!("value: {} must be <= {}", value, max_value) 
                    ))
                }
                Ok($varname(value))
            }

            pub fn new(value: u32) -> Result<Self> {
                let max_value = Self::get_max_value();
                Self::new_checked(value, max_value)
            }

            pub fn as_u8(&self) -> u8 {
                self.0 as u8
            }

            pub fn as_u16(&self) -> u16 {
                self.0 as u16
            }

            pub fn as_u32(&self) -> u32 {
                self.0
            }

            pub fn as_usize(&self) -> usize {
                self.0 as usize
            }

            pub fn get_max_len() -> usize {
                (((1 as u64) << $N) - 1) as usize
            }

            pub fn get_max_value() -> u32 {
                (((1 as u64) << $N) - 1) as u32
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

impl From<u8> for Number8 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u8> for Number9 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u8> for Number12 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u8> for Number13 {
    fn from(value: u8) -> Self {
        Self(value as u32)
    }
}

impl From<u16> for Number16 {
    fn from(value: u16) -> Self {
        Self(value as u32)
    }
}

impl From<u32> for Number32 {
    fn from(value: u32) -> Self {
        Self(value as u32)
    }
}

impl std::convert::TryFrom<u32> for Number5 {
    type Error = failure::Error;
    fn try_from(value: u32) -> ton_types::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number8 {
    type Error = failure::Error;
    fn try_from(value: u32) -> ton_types::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number9 {
    type Error = failure::Error;
    fn try_from(value: u32) -> ton_types::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number12 {
    type Error = failure::Error;
    fn try_from(value: u32) -> ton_types::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number13 {
    type Error = failure::Error;
    fn try_from(value: u32) -> ton_types::Result<Self> {
        Self::new(value)
    }
}

impl std::convert::TryFrom<u32> for Number16 {
    type Error = failure::Error;
    fn try_from(value: u32) -> ton_types::Result<Self> {
        Self::new(value)
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
    fn calc(&mut self, other: &Self) -> Result<bool> {
        self.add(other)
    }
}

impl CurrencyCollection {
    pub const fn default() -> Self { Self::new() }
    pub const fn new() -> Self {
        Self::from_grams(Grams::zero())
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

    pub fn with_grams(grams: u64) -> Self {
        Self::from_grams(Grams::from(grams))
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
    fn add(&mut self, other: &Self) -> Result<bool>;
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
    fn add(&mut self, other: &Self) -> Result<bool> {
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
        Ok(true)
    }
}

impl fmt::Display for CurrencyCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.grams)?;
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
pub struct InRefValue<X: Deserializable + Serializable>(pub X);

impl<X: Deserializable + Serializable> InRefValue<X> {
    pub fn new(inner: X) -> InRefValue<X> {
        InRefValue(inner)
    }
    pub fn inner(self) -> X {
        self.0
    }
}

impl<X: Deserializable + Serializable> AsRef<X> for InRefValue<X> {
    fn as_ref(&self) -> &X {
        &self.0
    }
}

impl<X: Deserializable + Serializable> Deref for InRefValue<X> {
    type Target = X;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<X: Deserializable + Serializable> Deserializable for InRefValue<X> {
    fn construct_from(slice: &mut SliceData) -> Result<Self> {
        Ok(Self(X::construct_from_reference(slice)?))
    }
}

impl<X: Deserializable + Serializable> Serializable for InRefValue<X> {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.checked_append_reference(self.0.serialize()?)?;
        Ok(())
    }
}

impl<X: Deserializable> Deserializable for Arc<X> {
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
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
                let value = value.write_to_new_cell()?;
                self.0.set_builder(key, &value)?;
                Ok(())
            }
            pub fn setref<K: Serializable>(&mut self, key: &K, value: &Cell) -> Result<()> {
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
                self.0.setref(key, value)?;
                Ok(())
            }
            pub fn add_key<K: Serializable>(&mut self, key: &K) -> Result<()> {
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
                let value = BuilderData::default();
                self.0.set_builder(key, &value)?;
                Ok(())
            }
            pub fn get<K: Serializable>(&self, key: &K) -> Result<Option<$x_type>> {
                self.get_as_slice(key)?
                    .map(|ref mut slice| <$x_type>::construct_from(slice)).transpose()
            }
            pub fn get_as_slice<K: Serializable>(&self, key: &K) -> Result<Option<SliceData>> {
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
                self.get_raw(key)
            }
            pub fn get_raw(&self, key: SliceData) -> Result<Option<SliceData>> {
                self.0.get(key)
            }
            pub fn remove<K: Serializable>(&mut self, key: &K) -> Result<bool> {
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
                let leaf = self.0.remove(key)?;
                Ok(leaf.is_some())
            }
            pub fn check_key<K: Serializable>(&self, key: &K) -> Result<bool> {
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
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
                let key = SliceData::load_builder(key.write_to_new_cell()?)?;
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
    pub const fn default() -> Self { Self(0) }
    pub const fn new(value: u32) -> Self { UnixTime32(value) }
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
    pub fn now() -> Self {
        UnixTime32( SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32 )
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

    pub fn write_maybe_to(cell: &mut BuilderData, s: Option<&Self>) -> Result<()> {
        match s {
            Some(s) => {
                cell.append_bit_one()?;
                cell.append_reference_cell(s.cell());
            }
            None => {
                cell.append_bit_zero()?;
            }
        }
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
                T::construct_from_cell(cell)
            }
            None => Ok(T::default())
        }
    }

    pub fn read_struct_from_option(opt: Option<&Self>) -> Result<Option<T>> {
        if let Some(s) = opt {
            if let Some(cell) = s.cell.as_ref() {
                if cell.cell_type() == CellType::PrunedBranch {
                    fail!(
                        BlockError::PrunedCellAccess(std::any::type_name::<T>().into())
                    )
                }
                return Ok(Some(T::construct_from_cell(cell.clone())?))
            }
        }
        Ok(None)
    }

    pub fn read_from_reference(&mut self, slice: &mut SliceData) -> Result<()> {
        self.cell = Some(slice.checked_drain_reference()?);
        Ok(())
    }

    pub fn construct_from_reference(slice: &mut SliceData) -> Result<Self> {
        let cell = slice.checked_drain_reference()?;
        Ok(Self::with_cell(cell))
    }

    pub fn construct_maybe_from_reference(slice: &mut SliceData) -> Result<Option<Self>> {
        match slice.get_next_bit()? {
            true => Ok(Some(Self::with_cell(slice.checked_drain_reference()?))),
            false => Ok(None)
        }
    }

    pub fn cell(&self)-> Cell {
        match self.cell.as_ref() {
            Some(cell) => cell.clone(),
            None => T::default().serialize().unwrap_or_default()
        }
    }

    pub fn set_cell(&mut self, cell: Cell) {
        self.cell = Some(cell);
    }

    pub fn hash(&self) -> UInt256 {
        match self.cell.as_ref() {
            Some(cell) => cell.repr_hash(),
            None => T::default().serialize().unwrap_or_default().repr_hash()
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
            (Some(cell), None) => cell.eq(&T::default().serialize().unwrap_or_default()),
            (None, None) => true
        }
    }
}
