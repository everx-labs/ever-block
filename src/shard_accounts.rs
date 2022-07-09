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

use crate::{
    define_HashmapAugE,
    accounts::{Account, ShardAccount},
    hashmapaug::{Augmentable, HashmapAugType},
    types::{CurrencyCollection, Number5},
    Serializable, Deserializable, Augmentation,
};
use std::fmt;
use ton_types::{
    error, fail, Result,
    AccountId, UInt256,
    BuilderData, Cell, IBitstring,
    HashmapType, SliceData, hm_label, HashmapSubtree,
};


/////////////////////////////////////////////////////////////////////////////////////////
// 4.1.9. The combined state of all accounts in a shard. The split part
// of the shardchain state (cf. 1.2.1 and 1.2.2) is given by (upd from Lite Client v11):
// _ (HashmapAugE 256 ShardAccount DepthBalanceInfo) = ShardAccounts;
define_HashmapAugE!(ShardAccounts, 256, UInt256, ShardAccount, DepthBalanceInfo);
impl HashmapSubtree for ShardAccounts {}

impl ShardAccounts {
    pub fn insert(&mut self, split_depth: u8, account: &Account, last_trans_hash: UInt256, last_trans_lt: u64) -> Result<Option<AccountId>> {
        match account.get_id() {
            Some(acc_id) => {
                let depth_balance_info = DepthBalanceInfo::new(split_depth, account.get_balance().unwrap())?;
                let sh_account = ShardAccount::with_params(account, last_trans_hash, last_trans_lt)?;
                self.set_builder_serialized(acc_id.clone(), &sh_account.write_to_new_cell()?, &depth_balance_info).unwrap();
                Ok(Some(acc_id))
            }
            _ => Ok(None)
        }
    }

    pub fn account(&self, account_id: &AccountId) -> Result<Option<ShardAccount>> {
        self.get_serialized(account_id.clone())
    }

    pub fn balance(&self, account_id: &AccountId) -> Result<Option<DepthBalanceInfo>> {
        match self.get_serialized_raw(account_id.clone())? {
            Some(mut slice) => Ok(Some(DepthBalanceInfo::construct_from(&mut slice)?)),
            None => Ok(None)
        }
    }

    pub fn full_balance(&self) -> &CurrencyCollection {
        &self.root_extra().balance
    }

    pub fn split_for(&mut self, split_key: &SliceData) -> Result<&DepthBalanceInfo> {
        self.into_subtree_with_prefix(split_key, &mut 0)?;
        self.update_root_extra()
    }
}

impl Augmentation<DepthBalanceInfo> for ShardAccount {
    fn aug(&self) -> Result<DepthBalanceInfo> {
        let account = self.read_account()?;
        let balance = account.balance().cloned().unwrap_or_default();
        let split_depth = account.split_depth().unwrap_or_default();
        Ok(DepthBalanceInfo {
            split_depth,
            balance,
        })
    }
}

/// depth_balance$_ split_depth:(#<= 30) balance:CurrencyCollection = DepthBalanceInfo;
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct DepthBalanceInfo {
    split_depth: Number5,
    balance: CurrencyCollection,
}

impl DepthBalanceInfo {
    pub fn new(split_depth: u8, balance: &CurrencyCollection) -> Result<Self> {
        Ok(Self {
            split_depth: Number5::new_checked(split_depth as u32, 30)?,
            balance: balance.clone(),
        })
    }

    pub fn set_split_depth(&mut self, split_depth: Number5) { self.split_depth = split_depth }

    pub fn set_balance(&mut self, balance: CurrencyCollection) { self.balance = balance }

    pub fn balance(&self) -> &CurrencyCollection { &self.balance }
}

impl Augmentable for DepthBalanceInfo {
    fn calc(&mut self, other: &Self) -> Result<bool> {
        self.balance.calc(&other.balance)
        // TODO: do something with split_depth
    }
}

impl Deserializable for DepthBalanceInfo {
    fn read_from(&mut self, cell: &mut SliceData) -> Result<()> {
        self.split_depth.read_from(cell)?;
        self.balance.read_from(cell)?;
        Ok(())
    }
}

impl Serializable for DepthBalanceInfo {
    fn write_to(&self, cell: &mut BuilderData) -> Result<()> {
        self.split_depth.write_to(cell)?;
        self.balance.write_to(cell)?;
        Ok(())
    }
}
