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
    define_HashmapE, define_HashmapE_empty_val,
    error::BlockError,
    signature::{CryptoSignature, SigPubKey},
    types::{ChildCell, Grams, Number8, Number12, Number16, Number13, Number32, VarUInteger32},
    validators::ValidatorSet,
    Serializable, Deserializable,
};
use std::ops::Deref;
use std::sync::Arc;
use ton_types::{
    error, fail, Result,
    UInt256,
    BuilderData, Cell, IBitstring, SliceData, HashmapE, HashmapType,
};

/*
1.6.3. Quick access through the header of masterchain blocks
_ config_addr:uint256
config:^(Hashmap 32 ^Cell) = ConfigParams;
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParams {
    pub config_addr: UInt256,
    pub config_params: HashmapE // <u32, SliceData>
}

impl Default for ConfigParams {
    fn default() -> ConfigParams {
        ConfigParams {
            config_addr: UInt256::default(),
            config_params: HashmapE::with_bit_len(32)
        }
    }
}

impl ConfigParams {
    /// create new instance ConfigParams
    pub fn new() -> Self {
        Self::default()
    }

    /// get config by index
    pub fn config(&self, index: u32) -> Result<Option<ConfigParamEnum>> {
        let key = index.write_to_new_cell().unwrap();
        if let Ok(Some(slice)) = self.config_params.get(key.into()) {
            if let Ok(cell) = slice.reference(0) {
                return Ok(Some(ConfigParamEnum::construct_from_slice_and_number(&mut cell.into(), index)?));
            }
        }
        Ok(None)
    }

    /// set config
    pub fn set_config(&mut self, config: ConfigParamEnum) -> Result<()> {

        let mut value = BuilderData::new();
        let index = config.write_to_cell(&mut value)?;
        let mut key = BuilderData::default();
        key.append_u32(index).unwrap();
        self.config_params.set(key.into(), &value.into())?;
        Ok(())
    }   
}

impl Deserializable for ConfigParams {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.config_addr.read_from(cell)?;
        *self.config_params.data_mut() = Some(cell.checked_drain_reference()?.clone());
        Ok(())
    }
}


impl Serializable for ConfigParams {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.config_params.is_empty() {
            // Due to ChildCell it is need sometimes to serialize default ConfigParams.
            // So need to wtite something.
            cell.append_reference_cell(Cell::default());
        } else {
            cell.append_reference_cell(self.config_params.data().unwrap().clone());
        }
        self.config_addr.write_to(cell)?;
        
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigParamEnum {
    ConfigParam0(ConfigParam0),
    ConfigParam1(ConfigParam1),
    ConfigParam2(ConfigParam2),
    ConfigParam3(ConfigParam3),
    ConfigParam4(ConfigParam4),
    ConfigParam6(ConfigParam6),
    ConfigParam7(ConfigParam7),
    ConfigParam8(ConfigParam8),
    ConfigParam9(ConfigParam9),
    ConfigParam10(ConfigParam10),
    ConfigParam11(ConfigParam11),
    ConfigParam12(ConfigParam12),
    ConfigParam14(ConfigParam14),
    ConfigParam15(ConfigParam15),
    ConfigParam16(ConfigParam16),
    ConfigParam17(ConfigParam17),
    ConfigParam18(ConfigParam18),
    ConfigParam20(GasLimitsPrices),
    ConfigParam21(GasLimitsPrices),
    ConfigParam22(ConfigParam22),
    ConfigParam23(ConfigParam23),
    ConfigParam24(MsgForwardPrices),
    ConfigParam25(MsgForwardPrices),
    ConfigParam28(CatchainConfig),
    ConfigParam29(ConfigParam29),
    ConfigParam31(ConfigParam31),
    ConfigParam32(ConfigParam32),
    ConfigParam33(ConfigParam33),
    ConfigParam34(ConfigParam34),
    ConfigParam35(ConfigParam35),
    ConfigParam36(ConfigParam36),
    ConfigParam37(ConfigParam37),
    ConfigParam39(ConfigParam39),
    ConfigParamAny(u32, SliceData),
}

macro_rules! read_config {
    ( $cpname:ident, $cname:ident, $slice:expr ) => {
        {
            let mut c = $cname::default();
            c.read_from($slice)?;
            Ok(ConfigParamEnum::$cpname(c))
        }
    }
}

impl ConfigParamEnum {
    
    /// read config from cell
    pub fn construct_from_slice_and_number(slice: &mut SliceData, index: u32) -> Result<ConfigParamEnum> {
        match index {
            0 => { read_config!(ConfigParam0, ConfigParam0, slice) },
            1 => { read_config!(ConfigParam1, ConfigParam1, slice) },
            2 => { read_config!(ConfigParam2, ConfigParam2, slice) },
            3 => { read_config!(ConfigParam3, ConfigParam3, slice) },
            4 => { read_config!(ConfigParam4, ConfigParam4, slice) },
            6 => { read_config!(ConfigParam6, ConfigParam6, slice) },
            7 => { read_config!(ConfigParam7, ConfigParam7, slice) },
            8 => { read_config!(ConfigParam8, ConfigParam8, slice) },
            9 => { read_config!(ConfigParam9, ConfigParam9, slice) },
            10 => { read_config!(ConfigParam10, ConfigParam10, slice) },
            11 => { read_config!(ConfigParam11, ConfigParam11, slice) },
            12 => { read_config!(ConfigParam12, ConfigParam12, slice) },
            14 => { read_config!(ConfigParam14, ConfigParam14, slice) },
            15 => { read_config!(ConfigParam15, ConfigParam15, slice) },
            16 => { read_config!(ConfigParam16, ConfigParam16, slice) },
            17 => { read_config!(ConfigParam17, ConfigParam17, slice) },
            18 => { read_config!(ConfigParam18, ConfigParam18, slice) },
            20 => { read_config!(ConfigParam20, GasLimitsPrices, slice) },
            21 => { read_config!(ConfigParam21, GasLimitsPrices, slice) },
            22 => { read_config!(ConfigParam22, ConfigParam22, slice) },
            23 => { read_config!(ConfigParam23, ConfigParam23, slice) },
            24 => { read_config!(ConfigParam24, MsgForwardPrices, slice) },
            25 => { read_config!(ConfigParam25, MsgForwardPrices, slice) },
            28 => { read_config!(ConfigParam28, CatchainConfig, slice) },
            29 => { read_config!(ConfigParam29, ConfigParam29, slice) },
            31 => { read_config!(ConfigParam31, ConfigParam31, slice) },
            32 => { read_config!(ConfigParam32, ConfigParam32, slice) },
            33 => { read_config!(ConfigParam33, ConfigParam33, slice) },
            34 => { read_config!(ConfigParam34, ConfigParam34, slice) },
            35 => { read_config!(ConfigParam35, ConfigParam35, slice) },
            36 => { read_config!(ConfigParam36, ConfigParam36, slice) },
            37 => { read_config!(ConfigParam37, ConfigParam37, slice) },
            39 => { read_config!(ConfigParam39, ConfigParam39, slice) },
            index @ _ => Ok(ConfigParamEnum::ConfigParamAny(index, slice.clone())),
        }
    }

    /// Save config to cell
    pub fn write_to_cell(&self, cell: &mut BuilderData) -> Result<u32> {
        match self {
            ConfigParamEnum::ConfigParam0(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(0)},
            ConfigParamEnum::ConfigParam1(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(1)},
            ConfigParamEnum::ConfigParam2(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(2)},
            ConfigParamEnum::ConfigParam3(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(3)},
            ConfigParamEnum::ConfigParam4(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(4)},
            ConfigParamEnum::ConfigParam6(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(6)},
            ConfigParamEnum::ConfigParam7(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(7)},
            ConfigParamEnum::ConfigParam8(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(8)},
            ConfigParamEnum::ConfigParam9(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(9)},
            ConfigParamEnum::ConfigParam10(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(10)},
            ConfigParamEnum::ConfigParam11(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(11)},
            ConfigParamEnum::ConfigParam12(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(12)},
            ConfigParamEnum::ConfigParam14(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(14)},
            ConfigParamEnum::ConfigParam15(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(15)},
            ConfigParamEnum::ConfigParam16(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(16)},
            ConfigParamEnum::ConfigParam17(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(17)},
            ConfigParamEnum::ConfigParam18(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(18)},
            ConfigParamEnum::ConfigParam20(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(20)},
            ConfigParamEnum::ConfigParam21(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(21)},
            ConfigParamEnum::ConfigParam22(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(22)},
            ConfigParamEnum::ConfigParam23(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(23)},
            ConfigParamEnum::ConfigParam24(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(24)},
            ConfigParamEnum::ConfigParam25(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(25)},
            ConfigParamEnum::ConfigParam28(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(28)},
            ConfigParamEnum::ConfigParam29(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(29)},
            ConfigParamEnum::ConfigParam31(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(31)},
            ConfigParamEnum::ConfigParam32(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(32)},
            ConfigParamEnum::ConfigParam33(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(33)},
            ConfigParamEnum::ConfigParam34(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(34)},
            ConfigParamEnum::ConfigParam35(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(35)},
            ConfigParamEnum::ConfigParam36(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(36)},
            ConfigParamEnum::ConfigParam37(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(37)},
            ConfigParamEnum::ConfigParam39(ref c) => { cell.append_reference(c.write_to_new_cell()?); Ok(39)},
            ConfigParamEnum::ConfigParamAny(index, slice) => { cell.append_reference_cell(slice.into_cell()); Ok(*index)},
        }
    }
}

/*
_ config_addr:bits256 = ConfigParam 0;
*/

///
/// Config Param 0 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam0 {
    pub config_addr: UInt256,
}

impl ConfigParam0 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam0 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.config_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam0 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.config_addr.write_to(cell)?;
        Ok(())
    }
}

/*
_ elector_addr:bits256 = ConfigParam 1;
*/

///
/// Config Param 1 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam1 {
    pub elector_addr: UInt256,
}

impl ConfigParam1 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam1 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.elector_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam1 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.elector_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 2 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam2 {
    pub minter_addr: UInt256,
}

impl ConfigParam2 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam2 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.minter_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam2 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.minter_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 3 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam3 {
    pub fee_collector_addr: UInt256,
}

impl ConfigParam3 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam3 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.fee_collector_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam3 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.fee_collector_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 4 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam4 {
    pub dns_root_addr: UInt256,
}

impl ConfigParam4 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam4 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.dns_root_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam4 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.dns_root_addr.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 6 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam6 {
    pub mint_new_price: Grams,
    pub mint_add_price: Grams,
}

impl ConfigParam6 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam6 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.mint_new_price.read_from(cell)?;
        self.mint_add_price.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam6 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.mint_new_price.write_to(cell)?;
        self.mint_add_price.write_to(cell)?;
        Ok(())
    }
}

define_HashmapE!{ExtraCurrencyCollection, 32, VarUInteger32}

///
/// Config Param 7 structure
/// 
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ConfigParam7 {
    pub to_mint: ExtraCurrencyCollection,
}

impl ConfigParam7 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam7 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.to_mint.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam7 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.to_mint.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 8 structure
/// 
// capabilities#c4 version:uint32 capabilities:uint64 = GlobalVersion;
// _ GlobalVersion = ConfigParam 8;  // all zero if absent

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GlobalVersion {
    pub version: u32,
    pub capabilities: u64,
}

impl GlobalVersion {
    pub fn new() -> Self {
        Self::default()
    }
}

const GLOBAL_VERSION_TAG: u8 = 0xC4;

impl Deserializable for GlobalVersion {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != GLOBAL_VERSION_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "GlobalVersion".to_string()
                }
            )
        }
        self.version.read_from(cell)?;
        self.capabilities.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for GlobalVersion {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(GLOBAL_VERSION_TAG)?;
        self.version.write_to(cell)?;
        self.capabilities.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam8 {
    pub global_version: GlobalVersion
}

impl ConfigParam8 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam8 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.global_version.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam8 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.global_version.write_to(cell)?;
        Ok(())
    }
}

// _ mandatory_params:(Hashmap 32 True) = ConfigParam 9;

define_HashmapE_empty_val!{MandatoryParams, 32}

///
/// Config Param 9 structure
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam9 {
    pub mandatory_params: MandatoryParams,
}

impl ConfigParam9 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam9 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.mandatory_params.read_hashmap_root(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam9 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.mandatory_params.write_hashmap_root(cell)?;
        Ok(())
    }
}

///
/// Config Param 10 structure
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam10 {
    pub critical_params: MandatoryParams,
}

impl ConfigParam10 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam10 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.critical_params.read_hashmap_root(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam10 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.critical_params.write_hashmap_root(cell)?;
        Ok(())
    }
}

///
/// Config Param 14 structure
/// 
// block_grams_created#6b masterchain_block_fee:Grams basechain_block_fee:Grams
//   = BlockCreateFees;
// _ BlockCreateFees = ConfigParam 14;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BlockCreateFees {
    pub masterchain_block_fee: Grams,
    pub basechain_block_fee: Grams,
}

impl BlockCreateFees {
    pub fn new() -> Self {
        Self::default()
    }
}

const BLOCK_CREATE_FEES: u8 = 0x6b;

impl Deserializable for BlockCreateFees {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != BLOCK_CREATE_FEES {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "BlockCreateFees".to_string()
                }
            )
        }
        self.masterchain_block_fee.read_from(cell)?;
        self.basechain_block_fee.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for BlockCreateFees {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(BLOCK_CREATE_FEES)?;
        self.masterchain_block_fee.write_to(cell)?;
        self.basechain_block_fee.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam14 {
    pub block_create_fees: BlockCreateFees
}

impl ConfigParam14 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam14 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.block_create_fees.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam14 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.block_create_fees.write_to(cell)?;
        Ok(())
    }
}

///
/// Config Param 15 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam15 {
    pub validators_elected_for: u32,
    pub elections_start_before: u32,
    pub elections_end_before: u32,
    pub stake_held_for: u32,
}

impl ConfigParam15 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam15 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.validators_elected_for.read_from(cell)?;
        self.elections_start_before.read_from(cell)?;
        self.elections_end_before.read_from(cell)?;
        self.stake_held_for.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam15 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.validators_elected_for.write_to(cell)?;
        self.elections_start_before.write_to(cell)?;
        self.elections_end_before.write_to(cell)?;
        self.stake_held_for.write_to(cell)?;
        Ok(())
    }
}

/*
_ max_validators:(## 16) max_main_validators:(## 16) min_validators:(## 16) 
  { max_validators >= max_main_validators } 
  { max_main_validators >= min_validators } 
  { min_validators >= 1 }
  = ConfigParam 16;
*/

///
/// Config Param 16 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam16 {
    pub max_validators: Number16,
    pub max_main_validators: Number16,
    pub min_validators: Number16,
}

impl ConfigParam16 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam16 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.max_validators.read_from(cell)?;
        self.max_main_validators.read_from(cell)?;
        self.min_validators.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam16 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.max_validators.write_to(cell)?;
        self.max_main_validators.write_to(cell)?;
        self.min_validators.write_to(cell)?;
        Ok(())
    }
}

/*
_ 
    min_stake: Grams 
    max_stake: Grams 
    min_total_stake: Grams 
    max_stake_factor: uint32 
= ConfigParam 17;
*/

///
/// Config Param 17 structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam17 {
    pub min_stake: Grams,
    pub max_stake: Grams,
    pub min_total_stake: Grams,
    pub max_stake_factor: u32,
}

impl ConfigParam17 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam17 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.min_stake.read_from(cell)?;
        self.max_stake.read_from(cell)?;
        self.min_total_stake.read_from(cell)?;
        self.max_stake_factor.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam17 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.min_stake.write_to(cell)?;
        self.max_stake.write_to(cell)?;
        self.min_total_stake.write_to(cell)?;
        self.max_stake_factor.write_to(cell)?;
        Ok(())
    }
}

/*
_#cc 
    utime_since:uint32 
    bit_price_ps:uint64 
    cell_price_ps:uint64 
    mc_bit_price_ps:uint64 
    mc_cell_price_ps:uint64 
= StoragePrices;

_ (Hashmap 32 StoragePrices) = ConfigParam 18;
*/

///
/// StoragePrices structure
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct StoragePrices {
    pub utime_since: u32,
    pub bit_price_ps: u64,
    pub cell_price_ps: u64,
    pub mc_bit_price_ps: u64,
    pub mc_cell_price_ps: u64,
}

impl StoragePrices {
    pub fn new() -> Self {
        Self::default()
    }
}

const STORAGE_PRICES_TAG: u8 = 0xCC;

impl Deserializable for StoragePrices {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != STORAGE_PRICES_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "StoragePrices".to_string()
                }
            )
        }
        self.utime_since.read_from(cell)?;
        self.bit_price_ps.read_from(cell)?;
        self.cell_price_ps.read_from(cell)?;
        self.mc_bit_price_ps.read_from(cell)?;
        self.mc_cell_price_ps.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for StoragePrices {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(STORAGE_PRICES_TAG)?;
        self.utime_since.write_to(cell)?;
        self.bit_price_ps.write_to(cell)?;
        self.cell_price_ps.write_to(cell)?;
        self.mc_bit_price_ps.write_to(cell)?;
        self.mc_cell_price_ps.write_to(cell)?;
        Ok(())
    }
}

///
/// ConfigParam 18 struct
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParam18 {
    pub map: HashmapE,
    index: u32
}

impl Default for ConfigParam18 {
    fn default() -> Self {
        ConfigParam18 {
            map: HashmapE::with_bit_len(32),
            index: 0,
        }
    }
}

impl ConfigParam18 {
    /// new instance of ConfigParam18
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> Result<usize> {
        self.map.len().map_err(|e| e.into())
    } 

    /// get value by index
    pub fn get(&self, index: u32) -> Result<StoragePrices> {
        let key = index.write_to_new_cell().unwrap().into();
        let mut s = self.map.get(key)?.ok_or(BlockError::InvalidIndex(index as usize))?;
        StoragePrices::construct_from(&mut s)
    }

    /// insert value
    pub fn insert(&mut self, sp: StoragePrices) {
        self.index += 1;
        let key = self.index.write_to_new_cell().unwrap();
        self.map.set(key.into(), &sp.write_to_new_cell().unwrap().into()).unwrap();
    }
}


impl Deserializable for ConfigParam18 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.map.read_hashmap_root(slice)?;
        self.index = self.map.len()? as u32;
        Ok(())
    }
}

impl Serializable for ConfigParam18 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.map.is_empty() {
            fail!(BlockError::InvalidOperation("self.map is empty".to_string()))
        }
        self.map.write_hashmap_root(cell)?;
        Ok(())
    }
}

/*
gas_prices#dd 
    gas_price:uint64 
    gas_limit:uint64 
    gas_credit:uint64 
    block_gas_limit:uint64 
    freeze_due_limit:uint64 
    delete_due_limit:uint64 
= GasLimitsPrices; 

gas_prices_ext#de
  gas_price:uint64
  gas_limit:uint64
  special_gas_limit:uint64
  gas_credit:uint64
  block_gas_limit:uint64
  freeze_due_limit:uint64
  delete_due_limit:uint64 
  = GasLimitsPrices;

gas_flat_pfx#d1
  flat_gas_limit:uint64
  flat_gas_price:uint64
  other:GasLimitsPrices
= GasLimitsPrices;
*/

///
/// GasLimitsPrices
/// 

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GasPrices {
    pub gas_price: u64,
    pub gas_limit: u64,
    pub gas_credit: u64,
    pub block_gas_limit: u64,
    pub freeze_due_limit: u64,
    pub delete_due_limit: u64,
}

impl Deserializable for GasPrices {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.gas_price.read_from(cell)?;
        self.gas_limit.read_from(cell)?;
        self.gas_credit.read_from(cell)?;
        self.block_gas_limit.read_from(cell)?;
        self.freeze_due_limit.read_from(cell)?;
        self.delete_due_limit.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for GasPrices {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.gas_price.write_to(cell)?;
        self.gas_limit.write_to(cell)?;
        self.gas_credit.write_to(cell)?;
        self.block_gas_limit.write_to(cell)?;
        self.freeze_due_limit.write_to(cell)?;
        self.delete_due_limit.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GasPricesEx {
    pub gas_price: u64,
    pub gas_limit: u64,
    pub special_gas_limit: u64,
    pub gas_credit: u64,
    pub block_gas_limit: u64,
    pub freeze_due_limit: u64,
    pub delete_due_limit: u64,
}

impl Deserializable for GasPricesEx {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.gas_price.read_from(cell)?;
        self.gas_limit.read_from(cell)?;
        self.special_gas_limit.read_from(cell)?;
        self.gas_credit.read_from(cell)?;
        self.block_gas_limit.read_from(cell)?;
        self.freeze_due_limit.read_from(cell)?;
        self.delete_due_limit.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for GasPricesEx {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.gas_price.write_to(cell)?;
        self.gas_limit.write_to(cell)?;
        self.special_gas_limit.write_to(cell)?;
        self.gas_credit.write_to(cell)?;
        self.block_gas_limit.write_to(cell)?;
        self.freeze_due_limit.write_to(cell)?;
        self.delete_due_limit.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GasFlatPfx {
    pub flat_gas_limit: u64,
    pub flat_gas_price: u64,
    pub other: Arc<GasLimitsPrices>,
}

impl Deserializable for GasFlatPfx {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.flat_gas_limit.read_from(cell)?;
        self.flat_gas_price.read_from(cell)?;
        self.other = Arc::new(GasLimitsPrices::construct_from(cell)?);
        if let GasLimitsPrices::FlatPfx(_) = self.other.deref() {
            fail!(
                BlockError::InvalidData("GasFlatPfx.other can't be GasFlatPfx".to_string())
            )
        }
        Ok(())
    }
}

impl Serializable for GasFlatPfx {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.flat_gas_limit.write_to(cell)?;
        self.flat_gas_price.write_to(cell)?;
        if let GasLimitsPrices::FlatPfx(_) = self.other.deref() {
            fail!(
                BlockError::InvalidData("GasFlatPfx.other can't be GasFlatPfx".to_string())
            )
        }
        self.other.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GasLimitsPrices {
    Std(GasPrices),
    Ex(GasPricesEx),
    FlatPfx(GasFlatPfx),
}

impl Default for GasLimitsPrices {
    fn default() -> Self {
        GasLimitsPrices::Std(GasPrices::default())
    }
}

impl GasLimitsPrices {
    pub fn new() -> Self {
        Self::default()
    }
}

const GAS_PRICES_TAG: u8 = 0xDD;
const GAS_PRICES_EXT_TAG: u8 = 0xDE;
const GAS_FLAT_PFX_TAG: u8 = 0xD1;

impl Deserializable for GasLimitsPrices {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        *self = match cell.get_next_byte()? {
            GAS_PRICES_TAG => GasLimitsPrices::Std(GasPrices::construct_from(cell)?),
            GAS_PRICES_EXT_TAG => GasLimitsPrices::Ex(GasPricesEx::construct_from(cell)?),
            GAS_FLAT_PFX_TAG => GasLimitsPrices::FlatPfx(GasFlatPfx::construct_from(cell)?),
            tag => {
                fail!(
                    BlockError::InvalidConstructorTag {
                        t: tag as u32,
                        s: "GasLimitsPrices".to_string()
                    }
                )
            }
        };
        Ok(())
    }
}

impl Serializable for GasLimitsPrices {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        match self {
            GasLimitsPrices::Std(gp) => {
                cell.append_u8(GAS_PRICES_TAG)?;
                gp.write_to(cell)?;
            },
            GasLimitsPrices::Ex(gp) => {
                cell.append_u8(GAS_PRICES_EXT_TAG)?;
                gp.write_to(cell)?;
            },
            GasLimitsPrices::FlatPfx(gp) => {
                cell.append_u8(GAS_FLAT_PFX_TAG)?;
                gp.write_to(cell)?;
            },
        };
        Ok(())
    }
}

/*
config_mc_gas_prices#_ GasLimitsPrices = ConfigParam 20;
*/
/*
config_gas_prices#_ GasLimitsPrices = ConfigParam 21;
*/


/*

// msg_fwd_fees = (lump_price + ceil((bit_price * msg.bits + cell_price * msg.cells)/2^16)) nanograms
// ihr_fwd_fees = ceil((msg_fwd_fees * ihr_price_factor)/2^16) nanograms
// bits in the root cell of a message are not included in msg.bits (lump_price pays for them)
msg_forward_prices#ea 
    lump_price:uint64 
    bit_price:uint64 
    cell_price:uint64
    ihr_price_factor:uint32 
    first_frac:uint16 
    next_frac:uint16 
= MsgForwardPrices;

*/

///
/// MsgForwardPrices
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct MsgForwardPrices {
    pub lump_price: u64,
    pub bit_price: u64,
    pub cell_price: u64,
    pub ihr_price_factor: u32,
    pub first_frac: u16,
    pub next_frac: u16,
}

impl MsgForwardPrices {
    pub fn new() -> Self {
        Self::default()
    }
}

const MSG_FWD_PRICES_TAG: u8 = 0xEA;

impl Deserializable for MsgForwardPrices {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != MSG_FWD_PRICES_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "MsgForwardPrices".to_string()
                }
            )
        }
        self.lump_price.read_from(cell)?;
        self.bit_price.read_from(cell)?;
        self.cell_price.read_from(cell)?;
        self.ihr_price_factor.read_from(cell)?;
        self.first_frac.read_from(cell)?;
        self.next_frac.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for MsgForwardPrices {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(MSG_FWD_PRICES_TAG)?;
        self.lump_price.write_to(cell)?;
        self.bit_price.write_to(cell)?;
        self.cell_price.write_to(cell)?;
        self.ihr_price_factor.write_to(cell)?;
        self.first_frac.write_to(cell)?;
        self.next_frac.write_to(cell)?;
        Ok(())
    }
}

/*
// used for messages to/from masterchain
config_mc_fwd_prices#_ MsgForwardPrices = ConfigParam 24;
// used for all other messages
config_fwd_prices#_ MsgForwardPrices = ConfigParam 25;

*/


/*
catchain_config#c1 
    mc_catchain_lifetime:uint32 
    shard_catchain_lifetime:uint32 
    shard_validators_lifetime:uint32 
    shard_validators_num:uint32 
= CatchainConfig;

catchain_config_new#c2
    flags: (## 7) 
    { flags = 0 } 
    shuffle_mc_validators: Bool
    mc_catchain_lifetime: uint3
    shard_catchain_lifetime: uint32
    shard_validators_lifetime: uint32 
    shard_validators_num: uint32 
= CatchainConfig;
*/

///
/// MsgForwardPrices
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CatchainConfig {
    pub shuffle_mc_validators: bool,
    pub mc_catchain_lifetime: u32,
    pub shard_catchain_lifetime: u32,
    pub shard_validators_lifetime: u32,
    pub shard_validators_num: u32,
}

impl CatchainConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

const CATCHAIN_CONFIG_TAG_1: u8 = 0xC1;
const CATCHAIN_CONFIG_TAG_2: u8 = 0xC2;

impl Deserializable for CatchainConfig {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if (tag != CATCHAIN_CONFIG_TAG_1) && (tag != CATCHAIN_CONFIG_TAG_2) {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "CatchainConfig".to_string()
                }
            )
        }
        if tag == CATCHAIN_CONFIG_TAG_2 {
            let flags = u8::construct_from(cell)?;
            self.shuffle_mc_validators = flags == 1;
            if flags >> 1 != 0 {
                fail!(BlockError::InvalidArg("`flags` should be zero".to_string()))
            }
        }
        self.mc_catchain_lifetime.read_from(cell)?;
        self.shard_catchain_lifetime.read_from(cell)?;
        self.shard_validators_lifetime.read_from(cell)?;
        self.shard_validators_num.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for CatchainConfig {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(CATCHAIN_CONFIG_TAG_2)?;
        cell.append_u8(self.shuffle_mc_validators as u8)?;
        self.mc_catchain_lifetime.write_to(cell)?;
        self.shard_catchain_lifetime.write_to(cell)?;
        self.shard_validators_lifetime.write_to(cell)?;
        self.shard_validators_num.write_to(cell)?;
        Ok(())
    }
}

/*
 _ CatchainConfig = ConfigParam 28;
 */


/*
_ fundamental_smc_addr:(HashmapE 256 True) = ConfigParam 31;

consensus_config#d6
    round_candidates:# { round_candidates >= 1 }
    next_candidate_delay_ms:uint32
    consensus_timeout_ms:uint32
    fast_attempts:uint32
    attempt_duration:uint32
    catchain_max_deps:uint32
    max_block_bytes:uint32
    max_collated_bytes:uint32
= ConsensusConfig;

consensus_config_new#d7
    flags: (## 7)
    { flags = 0 }
    new_catchain_ids: Bool
    round_candidates: (## 8) { round_candidates >= 1 }
    next_candidate_delay_ms: uint32 
    consensus_timeout_ms: uint32
    fast_attempts: uint32
    attempt_duration: uint32
    catchain_max_deps: uint32
    max_block_bytes: uint32
    max_collated_bytes: uint32 
= ConsensusConfig;
*/

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConsensusConfig {
    pub new_catchain_ids: bool,
    pub round_candidates: u32,
    pub next_candidate_delay_ms: u32,
    pub consensus_timeout_ms: u32,
    pub fast_attempts: u32,
    pub attempt_duration: u32,
    pub catchain_max_deps: u32,
    pub max_block_bytes: u32,
    pub max_collated_bytes: u32,
}

impl ConsensusConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

const CONSENSUS_CONFIG_TAG_1: u8 = 0xD6;
const CONSENSUS_CONFIG_TAG_2: u8 = 0xD7;

impl Deserializable for ConsensusConfig {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if (tag != CONSENSUS_CONFIG_TAG_1) && (tag != CONSENSUS_CONFIG_TAG_2) {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ConsensusConfig".to_string()
                }
            )
        }
        if tag == CONSENSUS_CONFIG_TAG_1 {
            self.round_candidates.read_from(cell)?;
        } else {
            let flags = u8::construct_from(cell)?;
            self.new_catchain_ids = flags == 1;
            if flags >> 1 != 0 {
                fail!(BlockError::InvalidArg("`flags` should be zero".to_string()))
            }
            self.round_candidates = u8::construct_from(cell)? as u32;
            if self.round_candidates == 0 {
                fail!(BlockError::InvalidArg("`round_candidates` should be positive".to_string()))
            }
        }
        self.next_candidate_delay_ms.read_from(cell)?;
        self.consensus_timeout_ms.read_from(cell)?;
        self.fast_attempts.read_from(cell)?;
        self.attempt_duration.read_from(cell)?;
        self.catchain_max_deps.read_from(cell)?;
        self.max_block_bytes.read_from(cell)?;
        self.max_collated_bytes.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConsensusConfig {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.round_candidates == 0 {
            fail!(BlockError::InvalidArg("`round_candidates` should be positive".to_string()))
        }
        cell.append_u8(CONSENSUS_CONFIG_TAG_2)?;
        cell.append_u8(self.new_catchain_ids as u8)?;
        (self.round_candidates as u8).write_to(cell)?;
        self.next_candidate_delay_ms.write_to(cell)?;
        self.consensus_timeout_ms.write_to(cell)?;
        self.fast_attempts.write_to(cell)?;
        self.attempt_duration.write_to(cell)?;
        self.catchain_max_deps.write_to(cell)?;
        self.max_block_bytes.write_to(cell)?;
        self.max_collated_bytes.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct ConfigParam29 {
    pub consensus_config: ConsensusConfig,
}

impl ConfigParam29 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deserializable for ConfigParam29 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.consensus_config.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam29 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.consensus_config.write_to(cell)?;
        Ok(())
    }
}

/*
_ fundamental_smc_addr:(HashmapE 256 True) = ConfigParam 31;
*/

define_HashmapE_empty_val!{FundamentalSmcAddresses, 256}

///
/// ConfigParam 31;
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam31 {
    pub fundamental_smc_addr: FundamentalSmcAddresses,
}

impl ConfigParam31 {
    pub fn new() -> Self {
        ConfigParam31::default()
    }

    pub fn add_address(&mut self, address: UInt256) {
        self.fundamental_smc_addr.add_key(&address).unwrap();
    }
}

impl Deserializable for ConfigParam31 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.fundamental_smc_addr.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for ConfigParam31 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.fundamental_smc_addr.write_to(cell)?;
        Ok(())
    }
}

macro_rules! define_configparams {
    ( $cpname:ident, $pname:ident ) => {
        ///
        /// $cpname structure
        /// 
        #[derive(Clone, Debug, Eq, PartialEq, Default)]
        pub struct $cpname {
            pub $pname: ValidatorSet,
        }

        impl $cpname {
            /// create new instance of $cpname
            pub fn new() -> Self {
                $cpname::default()
            }
        }

        impl Deserializable for $cpname {
            fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
                self.$pname.read_from(cell)?;
                Ok(())
            }
        }

        impl Serializable for $cpname {
            fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
                self.$pname.write_to(cell)?;
                Ok(())
            }
        }
    }
}

// _ prev_validators:ValidatorSet = ConfigParam 32;
define_configparams!(ConfigParam32, prev_validators);

// _ prev_temp_validators: ValidatorSet = ConfigParam 33;
define_configparams!(ConfigParam33, prev_temp_validators);

// _ cur_validators:ValidatorSet = ConfigParam 34;
define_configparams!(ConfigParam34, cur_validators);

// _ cur_temp_validators: ValidatorSet = ConfigParam 35;
define_configparams!(ConfigParam35, cur_temp_validators);

//_ next_validators:ValidatorSet = ConfigParam 36;
define_configparams!(ConfigParam36, next_validators);

// _ next_temp_validators: ValidatorSet = ConfigParam 37;
define_configparams!(ConfigParam37, next_temp_validators);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkchainFormat {
    Basic(WorkchainFormat1),
    Extended(WorkchainFormat0),
}

impl Default for WorkchainFormat {
    fn default() -> Self {
        WorkchainFormat::Basic(WorkchainFormat1::default())
    }
}

impl Deserializable for WorkchainFormat {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        cell.get_next_bits(3)?;
        *self = match cell.get_next_bit()? {
            true => {
                let mut val = WorkchainFormat1::default();
                val.read_from(cell)?;
                WorkchainFormat::Basic(val)
            }
            false => {
                let mut val = WorkchainFormat0::default();
                val.read_from(cell)?;
                WorkchainFormat::Extended(val)
            }
        };
        Ok(())
    }
}

impl Serializable for WorkchainFormat {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_bits(0, 3)?;
        match self {
            WorkchainFormat::Basic(ref val) => {
                cell.append_bit_one()?;
                val.write_to(cell)?;
            },
            WorkchainFormat::Extended(val) => {
                cell.append_bit_zero()?;
                val.write_to(cell)?;
            }
        }
        Ok(())
    }
}

/*
wfmt_basic#1 
	vm_version:int32 
	vm_mode:uint64 
= WorkchainFormat 1;
*/

///
/// Workchain format basic
/// 
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct WorkchainFormat1 {
    pub vm_version: i32,
    pub vm_mode: u64,
}

impl WorkchainFormat1 {
    ///
    /// Create empty intance of WorkchainFormat1
    /// 
    pub fn new() -> Self {
        WorkchainFormat1::default()
    }

    ///
    /// Create new instance of WorkchainFormat1
    /// 
    pub fn with_params(vm_version: i32, vm_mode: u64) -> Self {
        WorkchainFormat1 {
            vm_version,
            vm_mode
        }
    }
}


impl Deserializable for WorkchainFormat1 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.vm_version.read_from(cell)?;
        self.vm_mode.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for WorkchainFormat1 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.vm_version.write_to(cell)?;
        self.vm_mode.write_to(cell)?;
        Ok(())
    }
}


/*
wfmt_ext#0 
	min_addr_len:(## 12) 
	max_addr_len:(## 12) 
	addr_len_step:(## 12)
  { min_addr_len >= 64 } { min_addr_len <= max_addr_len } 
  { max_addr_len <= 1023 } { addr_len_step <= 1023 }
  workchain_type_id:(## 32) { workchain_type_id >= 1 }
= WorkchainFormat 0;
*/

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkchainFormat0 {
    min_addr_len: u16,  //use 12 bit
    max_addr_len: u16,  //use 12 bit
    addr_len_step: u16, // use 12 bit
    workchain_type_id: u32
}

impl Default for WorkchainFormat0 {
    fn default() -> Self {
        WorkchainFormat0 {
            min_addr_len: 64,
            max_addr_len: 64,
            addr_len_step: 0,
            workchain_type_id: 1
        }
    }
}

impl WorkchainFormat0 {
    ///
    /// Create empty new instance of WorkchainFormat0
    /// 
    pub fn new() -> Self {
        WorkchainFormat0::default()
    }

    ///
    /// Create new instance of WorkchainFormat0
    /// 
    pub fn with_params(min_addr_len: u16, max_addr_len: u16, addr_len_step: u16, workchain_type_id: u32 ) -> Result<WorkchainFormat0> {
        if min_addr_len >= 64 && min_addr_len <= max_addr_len &&
           max_addr_len <= 1023 && addr_len_step <= 1023 &&
           workchain_type_id >= 1 {
               Ok(
                   WorkchainFormat0 {
                        min_addr_len,
                        max_addr_len,
                        addr_len_step,
                        workchain_type_id, 
                   }
               )
           }
        else {
            fail!(
                BlockError::InvalidData(
                    "min_addr_len >= 64 && min_addr_len <= max_addr_len \
                     && max_addr_len <= 1023 && addr_len_step <= 1023 \
                     && workchain_type_id >= 1".to_string()
                )
            )
        }
    }

    ///
    /// Getter for min_addr_len
    /// 
    pub fn min_addr_len(&self) -> u16 {
        self.min_addr_len
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_min_addr_len(&mut self, min_addr_len: u16) -> Result<()> {
        if min_addr_len >= 64 && min_addr_len <= 1023 {
            self.min_addr_len = min_addr_len;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_addr_len >= 64 && min_addr_len <= 1023".to_string()
                )
            )
        }
    }    

    ///
    /// Getter for min_addr_len
    /// 
    pub fn max_addr_len(&self) -> u16 {
        self.max_addr_len
    }

    ///
    /// Setter for max_addr_len
    /// 
    pub fn set_max_addr_len(&mut self, max_addr_len: u16) -> Result<()> {
        if max_addr_len >= 64 && max_addr_len <= 1024 && self.min_addr_len <= max_addr_len {
            self.max_addr_len = max_addr_len;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: max_addr_len >= 64 && max_addr_len <= 1024 \
                     && self.min_addr_len <= max_addr_len".to_string()
                )
            )
        }
    }        

    ///
    /// Getter for addr_len_step
    /// 
    pub fn addr_len_step(&self) -> u16 {
        self.addr_len_step
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_addr_len_step(&mut self, addr_len_step: u16) -> Result<()> {
        if addr_len_step <= 1024 {
            self.addr_len_step = addr_len_step;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData("should: addr_len_step <= 1024".to_string())
            )
        }
    }       

    ///
    /// Getter for workchain_type_id
    /// 
    pub fn workchain_type_id(&self) -> u32 {
        self.workchain_type_id
    }

    ///
    /// Setter for min_addr_len
    /// 
    pub fn set_workchain_type_id(&mut self, workchain_type_id: u32) -> Result<()> {
        if workchain_type_id >= 1 {
            self.workchain_type_id = workchain_type_id;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData("should: workchain_type_id >= 1".to_string())
            )
        }
    } 
}

impl Deserializable for WorkchainFormat0 {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let mut val = Number12::default();
        val.read_from(cell)?;
        self.min_addr_len = val.0 as u16;
        val.read_from(cell)?;
        self.max_addr_len = val.0 as u16;
        val.read_from(cell)?;
        self.addr_len_step = val.0 as u16;
        let mut val = Number32::default();
        val.read_from(cell)?;
        self.workchain_type_id = val.0;
        if self.min_addr_len >= 64 && self.min_addr_len <= self.max_addr_len &&
           self.max_addr_len <= 1023 && self.addr_len_step <= 1023 &&
           self.workchain_type_id >= 1 {
                Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_addr_len >= 64 && min_addr_len <= max_addr_len \
                     && max_addr_len <= 1023 && addr_len_step <= 1023".to_string()
                )
            )
        }
    }
}

impl Serializable for WorkchainFormat0 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.min_addr_len >= 64 && self.min_addr_len <= self.max_addr_len &&
           self.max_addr_len <= 1023 && self.addr_len_step <= 1023 &&
           self.workchain_type_id >= 1 {
                let min = Number12(self.min_addr_len as u32);
                let max = Number12(self.max_addr_len as u32);
                let len = Number12(self.addr_len_step as u32);
                let id = Number32(self.workchain_type_id);
                min.write_to(cell)?;
                max.write_to(cell)?;
                len.write_to(cell)?;
                id.write_to(cell)?;
                Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_addr_len >= 64 && min_addr_len <= max_addr_len \
                     && max_addr_len <= 1023 && addr_len_step <= 1023".to_string()
                )
            )
        }
    }
}



/*
workchain#a5 
	enabled_since:uint32 
	min_split:(## 8)
	 max_split:(## 8)
  { min_split <= max_split } { max_split <= 60 }
  basic:(## 1) 
	active:Bool 
	accept_msgs:Bool 
	flags:(## 13) { flags = 0 }
  zerostate_root_hash:bits256 
	zerostate_file_hash:bits256
  version:uint32 
	format:(WorkchainFormat basic)
= WorkchainDescr;
*/

///
/// WorkchainDescr structure
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkchainDescr {
    pub enabled_since: u32,
    actual_min_split: u8,
    min_split: u8,
    max_split: u8,
    //basic: bool, // depends on format 
    pub active: bool,
    pub accept_msgs: bool,
    pub flags: u16, // 13 bit
    pub zerostate_root_hash: UInt256,
    pub zerostate_file_hash: UInt256,
    pub version: u32,
    pub format: WorkchainFormat
}

impl WorkchainDescr {
    ///
    /// Create empty instance of WorkchainDescr
    /// 
    pub fn new() -> Self {
        WorkchainDescr::default()
    }

    ///
    /// Getter for min_split
    /// 
    pub fn min_split(&self) -> u8 {
        self.min_split
    }

    ///
    /// Setter for min_split
    /// 
    pub fn set_min_split(&mut self, min_split: u8) -> Result<()> {
        if min_split <= 60 {
            self.min_split = min_split;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_split <= max_split && max_split <= 60".to_string()
                )
            )
        }
    } 

    ///
    /// Getter for actual_min_split
    /// 
    pub fn actual_min_split(&self) -> u8 {
        self.actual_min_split
    }

    ///
    /// Getter for max_split
    /// 
    pub fn max_split(&self) -> u8 {
        self.max_split
    }

    ///
    /// Setter for max_split
    /// 
    pub fn set_max_split(&mut self, max_split: u8) -> Result<()> {
        if self.min_split <= max_split && max_split <= 60 {
            self.max_split = max_split;
            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_split <= max_split && max_split <= 60".to_string()
                )
            )
        }
    } 

}

const WORKCHAIN_DESCRIPTOR_TAG : u8 = 0xA6;

impl Deserializable for WorkchainDescr {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        let tag = cell.get_next_byte()?;
        if tag != WORKCHAIN_DESCRIPTOR_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "WorkchainDescr".to_string()
                }
            )
        }
        self.enabled_since.read_from(cell)?;
        let mut min = Number8::default();
        min.read_from(cell)?;
        self.actual_min_split = min.0 as u8;
        let mut min = Number8::default();
        min.read_from(cell)?;
        self.min_split = min.0 as u8;
        let mut max = Number8::default();
        max.read_from(cell)?;
        self.max_split = max.0 as u8;
        cell.get_next_bit()?; // basic
        self.active = cell.get_next_bit()?;
        self.accept_msgs = cell.get_next_bit()?;
        let mut flags = Number13::default();
        flags.read_from(cell)?;
        self.flags = flags.0 as u16;
        self.zerostate_root_hash.read_from(cell)?;
        self.zerostate_file_hash.read_from(cell)?;
        self.version.read_from(cell)?;
        self.format.read_from(cell)?;

        Ok(())
    }
}

impl Serializable for WorkchainDescr {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        if self.min_split <= self.max_split && self.max_split <= 60 {

            cell.append_u8(WORKCHAIN_DESCRIPTOR_TAG)?;

            self.enabled_since.write_to(cell)?;

            let min = Number8(self.actual_min_split as u32);
            min.write_to(cell)?;

            let min = Number8(self.min_split as u32);
            min.write_to(cell)?;

            let max = Number8(self.max_split as u32);
            max.write_to(cell)?;

            if let WorkchainFormat::Basic(_) = self.format {
                cell.append_bit_one()?;
            } else {
                cell.append_bit_zero()?;
            }

            if self.active {
                cell.append_bit_one()?;
            } else {
                cell.append_bit_zero()?;
            }

            if self.accept_msgs {
                cell.append_bit_one()?;
            } else {
                cell.append_bit_zero()?;
            }

            let flags = Number13(self.flags as u32);
            flags.write_to(cell)?;
            self.zerostate_root_hash.write_to(cell)?;
            self.zerostate_file_hash.write_to(cell)?;
            self.version.write_to(cell)?;
            self.format.write_to(cell)?;

            Ok(())
        } else {
            fail!(
                BlockError::InvalidData(
                    "should: min_split <= max_split && max_split <= 60".to_string()
                )
            )
        }
    }
}

/*
cfg_vote_cfg#36
    min_tot_rounds: uint8
    max_tot_rounds: uint8
    min_wins: uint8
    max_losses: uint8
    min_store_sec: uint32
    max_store_sec: uint32
    bit_price: uint32
    cell_price: uint32
= ConfigProposalSetup;

cfg_vote_setup#91
    normal_params: ^ConfigProposalSetup
    critical_params: ^ConfigProposalSetup
= ConfigVotingSetup;

_ ConfigVotingSetup = ConfigParam 11;
*/

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigProposalSetup {
    pub min_tot_rounds: u8,
    pub max_tot_rounds: u8,
    pub min_wins: u8,
    pub max_losses: u8,
    pub min_store_sec: u32,
    pub max_store_sec: u32,
    pub bit_price: u32,
    pub cell_price: u32,
}

const CONFIG_PROPOSAL_SETUP_TAG : u8 = 0x36;

impl Deserializable for ConfigProposalSetup {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != CONFIG_PROPOSAL_SETUP_TAG {
            fail!(BlockError::InvalidConstructorTag {
                t: tag as u32,
                s: "ConfigProposalSetup".into()
            })
        }
        self.min_tot_rounds.read_from(slice)?;
        self.max_tot_rounds.read_from(slice)?;
        self.min_wins.read_from(slice)?;
        self.max_losses.read_from(slice)?;
        self.min_store_sec.read_from(slice)?;
        self.max_store_sec.read_from(slice)?;
        self.bit_price.read_from(slice)?;
        self.cell_price.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigProposalSetup {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(CONFIG_PROPOSAL_SETUP_TAG)?;
        self.min_tot_rounds.write_to(cell)?;
        self.max_tot_rounds.write_to(cell)?;
        self.min_wins.write_to(cell)?;
        self.max_losses.write_to(cell)?;
        self.min_store_sec.write_to(cell)?;
        self.max_store_sec.write_to(cell)?;
        self.bit_price.write_to(cell)?;
        self.cell_price.write_to(cell)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigVotingSetup {
    normal_params: ChildCell<ConfigProposalSetup>,
    critical_params: ChildCell<ConfigProposalSetup>,
}

impl ConfigVotingSetup {

    pub fn new(normal_params: &ConfigProposalSetup, critical_params: &ConfigProposalSetup) -> Result<Self> {
        Ok(
            ConfigVotingSetup {
                normal_params: ChildCell::with_struct(normal_params)?,
                critical_params: ChildCell::with_struct(critical_params)?,
            }
        )
    }

    pub fn read_normal_params(&self) -> Result<ConfigProposalSetup> {
        self.normal_params.read_struct()
    }

    pub fn write_normal_params(&mut self, value: &ConfigProposalSetup) -> Result<()> {
        self.normal_params.write_struct(value)
    }

    pub fn normal_params_cell(&self) -> &Cell {
        self.normal_params.cell()
    }

    pub fn read_critical_params(&self) -> Result<ConfigProposalSetup> {
        self.critical_params.read_struct()
    }

    pub fn write_critical_params(&mut self, value: &ConfigProposalSetup) -> Result<()> {
        self.critical_params.write_struct(value)
    }

    pub fn critical_params_cell(&self) -> &Cell {
        self.critical_params.cell()
    }
}

const CONFIG_VOTING_SETUP_TAG : u8 = 0x91;

impl Deserializable for ConfigVotingSetup {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != CONFIG_VOTING_SETUP_TAG {
            fail!(BlockError::InvalidConstructorTag {
                t: tag as u32,
                s: "ConfigVotingSetup".into()
            })
        }
        self.normal_params.read_from(&mut slice.checked_drain_reference()?.into())?;
        self.critical_params.read_from(&mut slice.checked_drain_reference()?.into())?;

        Ok(())
    }
}

impl Serializable for ConfigVotingSetup {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(CONFIG_VOTING_SETUP_TAG)?;
        cell.append_reference(self.normal_params.write_to_new_cell()?);
        cell.append_reference(self.critical_params.write_to_new_cell()?);
        Ok(())
    }
}

pub type ConfigParam11 = ConfigVotingSetup;

/*
_ workchains:(HashmapE 32 WorkchainDescr) = ConfigParam 12;
*/



define_HashmapE!{Workchains, 32, WorkchainDescr}

///
/// ConfigParam 12 struct
/// 
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigParam12 {
    pub workchains: Workchains,
}

impl ConfigParam12 {
    /// new instance of ConfigParam18
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> Result<usize> {
        Ok(self.workchains.len()?)
    } 

    /// get value by index
    pub fn get(&self, workchain_id: i32) -> Result<Option<WorkchainDescr>> {
        self.workchains.get(&workchain_id)
    }

    /// insert value
    pub fn insert(&mut self, workchain_id: i32, sp: &WorkchainDescr) {
        self.workchains.set(&workchain_id, sp).unwrap();
    }
}


impl Deserializable for ConfigParam12 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.workchains.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam12 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.workchains.write_to(cell)?;
        Ok(())
    }
}

// validator_temp_key#3
//     adnl_addr:bits256
//     temp_public_key:SigPubKey
//     seqno:#
//     valid_until:uint32
// = ValidatorTempKey;

const VALIDATOR_TEMP_KEY_TAG: u8 = 0x3;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ValidatorTempKey {
    adnl_addr: UInt256,
    temp_public_key: SigPubKey,
    seqno: u32,
    valid_until: u32
}

impl ValidatorTempKey {
    /// new instance of ValidatorTempKey
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_params(adnl_addr: UInt256, temp_public_key: SigPubKey, seqno: u32, valid_until: u32) 
        -> Self {
        Self {
            adnl_addr,
            temp_public_key,
            seqno,
            valid_until 
        }
    }

    pub fn set_adnl_addr(&mut self, adnl_addr: UInt256) {
        self.adnl_addr = adnl_addr
    }

    pub fn adnl_addr(&self) -> &UInt256 {
        &self.adnl_addr
    }

    pub fn set_key(&mut self, temp_public_key: SigPubKey) {
        self.temp_public_key = temp_public_key
    }

    pub fn temp_public_key(&self) -> &SigPubKey {
        &self.temp_public_key
    }

    pub fn set_seqno(&mut self, seqno: u32) {
        self.seqno = seqno
    }

    pub fn seqno(&self) -> u32 {
        self.seqno
    }

    pub fn set_valid_until(&mut self, valid_until: u32) {
        self.valid_until = valid_until
    }

    pub fn valid_until(&self) -> u32 {
        self.valid_until
    }
}

impl Deserializable for ValidatorTempKey {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?; // TODO what is tag length in bits???
        if tag != VALIDATOR_TEMP_KEY_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ValidatorTempKey".to_string()
                }
            )
        }
        self.adnl_addr.read_from(slice)?;
        self.temp_public_key.read_from(slice)?;
        self.seqno.read_from(slice)?;
        self.valid_until.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ValidatorTempKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(VALIDATOR_TEMP_KEY_TAG)?; // TODO what is tag length in bits???
        self.adnl_addr.write_to(cell)?;
        self.temp_public_key.write_to(cell)?;
        self.seqno.write_to(cell)?;
        self.valid_until.write_to(cell)?;
        Ok(())
    }
}


// signed_temp_key#4
//     key:^ValidatorTempKey
//     signature:CryptoSignature
// = ValidatorSignedTempKey;

const VALIDATOR_SIGNED_TEMP_KEY_TAG: u8 = 0x4;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ValidatorSignedTempKey {
    key: ValidatorTempKey,
    signature: CryptoSignature
}

impl ValidatorSignedTempKey {

    /// new instance of 
    pub fn with_key_and_signature(key: ValidatorTempKey, signature: CryptoSignature) -> Self {
        Self {key, signature}
    }

    pub fn key(&self) -> &ValidatorTempKey {
        &self.key
    }

    pub fn signature(&self) -> &CryptoSignature {
        &self.signature
    }
}

impl Deserializable for ValidatorSignedTempKey {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?; // TODO what is tag length in bits???
        if tag != VALIDATOR_SIGNED_TEMP_KEY_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ValidatorSignedTempKey".to_string()
                }
            )
        }
        self.signature.read_from(slice)?;
        self.key.read_from(&mut slice.checked_drain_reference()?.into())?;
        Ok(())
    }
}

impl Serializable for ValidatorSignedTempKey {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(VALIDATOR_SIGNED_TEMP_KEY_TAG)?; // TODO what is tag length in bits???
        self.signature.write_to(cell)?;
        cell.append_reference(self.key.write_to_new_cell()?);
        Ok(())
    }
}

///
/// ConfigParam 39 struct
/// 
// _ (HashmapE 256 ValidatorSignedTempKey) = ConfigParam 39;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigParam39 {
    pub validator_keys: HashmapE,
}

impl Default for ConfigParam39 {
    fn default() -> Self {
        ConfigParam39 {
            validator_keys: HashmapE::with_bit_len(256),
        }
    }
}

impl ConfigParam39 {
    /// new instance of ConfigParam39
    pub fn new() -> Self {
        Self::default()
    }

    /// get length
    pub fn len(&self) -> Result<usize> {
        Ok(self.validator_keys.len()?)
    } 

    /// get value by key
    pub fn get(&self, key: UInt256) -> Result<ValidatorSignedTempKey> {
        self.validator_keys.get(key.write_to_new_cell().unwrap().into())
            .map(|ref mut s| -> Result<ValidatorSignedTempKey> 
                {
                    let mut sp = ValidatorSignedTempKey::default(); 
                    if let Some(s) = s {
                       sp.read_from(s)?
                    } else {
                        fail!(
                            BlockError::NotFound("ValidatorSignedTempKey".to_string())
                        )
                    }; 
                    Ok(sp)
                }).unwrap()
    }

    /// insert value
    pub fn insert(&mut self, key: &UInt256, validator_key: &ValidatorSignedTempKey) {
        let key = key.write_to_new_cell().unwrap();
        self.validator_keys.set(key.into(), &validator_key.write_to_new_cell().unwrap().into()).unwrap();
    }
}

impl Deserializable for ConfigParam39 {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        self.validator_keys = HashmapE::with_bit_len(256);
        self.validator_keys.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for ConfigParam39 {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.validator_keys.write_to(cell)?;
        Ok(())
    }
}


///
///  struct ParamLimits
/// 
// param_limits#c3
//     underload:#
//     soft_limit:#
//     { underload <= soft_limit }
//     hard_limit:#
//     { soft_limit <= hard_limit }
// = ParamLimits;

const PARAM_LIMITS_TAG: u8 = 0xc3;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct ParamLimits {
    underload: u32,
    soft_limit: u32,
    hard_limit: u32
}

impl ParamLimits {
    /// new instance of ParamLimits
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limits(underload: u32, soft_limit: u32, hard_limit: u32) -> Result<Self> {
        if underload > soft_limit { 
            fail!(
                BlockError::InvalidArg(
                    "`underload` have to be less or equal `soft_limit`".to_string() 
                )
            )
        }
        if soft_limit > hard_limit { 
            fail!(
                BlockError::InvalidArg(
                   "`soft_limit` have to be less or equal `hard_limit`".to_string() 
                )
            )
        }
        Ok(ParamLimits{ underload, soft_limit, hard_limit })
    }

    pub fn underload(&self) -> u32 {
        self.underload
    }

    pub fn set_underload(&mut self, underload: u32) -> Result<()>{
        if underload > self.soft_limit { 
            fail!(
                BlockError::InvalidArg(
                    "`underload` have to be less or equal `soft_limit`".to_string() 
                )
            )
        }
        self.underload = underload;
        Ok(())
    }

    pub fn soft_limit(&self) -> u32 {
        self.soft_limit
    }

    pub fn set_soft_limit(&mut self, soft_limit: u32) -> Result<()>{
        if soft_limit > self.hard_limit { 
            fail!(
                BlockError::InvalidArg(
                    "`soft_limit` have to be less or equal `hard_limit`".to_string() 
                )
            )
        }
        self.soft_limit = soft_limit;
        Ok(())
    }

    pub fn hard_limit(&self) -> u32 {
        self.hard_limit
    }

    pub fn set_hard_limit(&mut self, hard_limit: u32) -> Result<()>{
        if self.soft_limit > hard_limit { 
            fail!(
                BlockError::InvalidArg(
                    "`hard_limit` have to be larger or equal `soft_limit`".to_string()
                )
            )
        }
        self.hard_limit = hard_limit;
        Ok(())
    }
}

impl Deserializable for ParamLimits {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != PARAM_LIMITS_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "ParamLimits".to_string()
                }
            )
        }
        self.underload.read_from(slice)?;
        self.soft_limit.read_from(slice)?;
        self.hard_limit.read_from(slice)?;
        if self.underload > self.soft_limit {
            fail!(
                BlockError::InvalidData(
                    "`underload` have to be less or equal `soft_limit`".to_string()
                )
            )
        }
        if self.soft_limit > self.hard_limit {
            fail!( 
                BlockError::InvalidData(
                    "`soft_limit` have to be less or equal `hard_limit`".to_string()
                )
            )
        }
        Ok(())
    }
}

impl Serializable for ParamLimits {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(PARAM_LIMITS_TAG)?;
        self.underload.write_to(cell)?;
        self.soft_limit.write_to(cell)?;
        self.hard_limit.write_to(cell)?;
        Ok(())
    }
}

///
///  struct BlockLimits
/// 
// block_limits#5d
//     bytes:ParamLimits
//     gas:ParamLimits
//     lt_delta:ParamLimits
// = BlockLimits;

const BLOCK_LIMITS_TAG: u8 = 0x5d;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct BlockLimits {
    bytes: ParamLimits,
    gas: ParamLimits,
    lt_delta: ParamLimits
}

impl BlockLimits {
    /// new instance of ConfigParam39
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limits(bytes: ParamLimits, gas: ParamLimits, lt_delta: ParamLimits) -> Self {
        Self { bytes, gas, lt_delta }
    }

    pub fn bytes(&self) -> &ParamLimits {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut ParamLimits {
        &mut self.bytes
    }

    pub fn gas(&self) -> &ParamLimits {
        &self.gas
    }

    pub fn gas_mut(&mut self) -> &mut ParamLimits {
        &mut self.gas
    }

    pub fn lt_delta(&self) -> &ParamLimits {
        &self.lt_delta
    }

    pub fn lt_delta_mut(&mut self) -> &mut ParamLimits {
        &mut self.lt_delta
    }
}

impl Deserializable for BlockLimits {
    fn read_from(&mut self, slice: &mut SliceData) -> Result<()> {
        let tag = slice.get_next_byte()?;
        if tag != BLOCK_LIMITS_TAG {
            fail!(
                BlockError::InvalidConstructorTag {
                    t: tag as u32,
                    s: "BlockLimits".to_string()
                }
            )
        }
        self.bytes.read_from(slice)?;
        self.gas.read_from(slice)?;
        self.lt_delta.read_from(slice)?;
        Ok(())
    }
}

impl Serializable for BlockLimits {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        cell.append_u8(BLOCK_LIMITS_TAG)?;
        self.bytes.write_to(cell)?;
        self.gas.write_to(cell)?;
        self.lt_delta.write_to(cell)?;
        Ok(())
    }
}

type ConfigParam22 = BlockLimits;
type ConfigParam23 = BlockLimits;

