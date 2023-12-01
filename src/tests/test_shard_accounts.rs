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
    generate_test_account_by_init_code_hash, write_read_and_assert,
};
use super::*;

#[test]
fn test_serialization_shard_account() {
    let mut shard_acc = ShardAccounts::default();
    
    for n in 5..6 {
        let acc = generate_test_account_by_init_code_hash(false);
        shard_acc.insert(n, &acc, UInt256::default(), 0).unwrap();
    }
    write_read_and_assert(shard_acc);
}
