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
use super::hashmapaug::Augmentable;
use {SliceData, BuilderData};
use AccountId;


/////////////////////////////////////////////////////////////////////////////////////////
// 4.1.9. The combined state of all accounts in a shard. The split part
// of the shardchain state (cf. 1.2.1 and 1.2.2) is given by (upd from Lite Client v11):
// _ (HashmapAugE 256 ShardAccount DepthBalanceInfo) = ShardAccounts;
define_HashmapAugE!(ShardAccounts, 256, ShardAccount, DepthBalanceInfo);

impl ShardAccounts {
    pub fn insert(&mut self, split_depth: u8, account: Account, last_trans_hash: UInt256, last_trans_lt: u64) -> Result<Option<AccountId>> {
        match account.get_id() {
            Some(acc_id) => {
                let depth_balance_info = DepthBalanceInfo::new(split_depth, account.get_balance().unwrap())?;
                let sh_account = ShardAccount::with_params(account, last_trans_hash, last_trans_lt)?;
                self.set(&acc_id, &sh_account, &depth_balance_info).unwrap();
                Ok(Some(acc_id))
            }
            _ => Ok(None)
        }
    }

    pub fn account(&self, account_id: &AccountId) -> Result<Option<ShardAccount>> {
        match self.0.get_with_aug(account_id.clone(), &mut 0)? {
            (Some(mut slice), _aug) => Ok(Some(ShardAccount::construct_from(&mut slice)?)),
            _ => Ok(None)
        }
    }

    pub fn balance(&self, account_id: &AccountId) -> Result<Option<DepthBalanceInfo>> {
        self.0.get_with_aug(account_id.clone(), &mut 0).map(|result| result.1)
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
            split_depth: Number5::from_u32(split_depth as u32, 30)?,
            balance: balance.clone(),
        })
    }

    pub fn set_balance(&mut self, balance: CurrencyCollection) {
        self.balance = balance
    }
}

impl Augmentable for DepthBalanceInfo {
    fn calc(&mut self, other: &Self) -> Result<()> {
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
