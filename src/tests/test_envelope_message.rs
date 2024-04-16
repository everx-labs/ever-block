/*
* Copyright (C) 2019-2024 EverX. All Rights Reserved.
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

use crate::{
    write_read_and_assert,
    InternalMessageHeader, StateInit, TickTock, types::Number5, MsgAddressInt, ShardIdent, CurrencyCollection
};
use super::*;

fn check_serialization_intermediate_addr_regular(addr_orig: IntermediateAddressRegular){
    let mut b = BuilderData::new();

    addr_orig.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    let mut addr_restored = IntermediateAddressRegular::default();
    addr_restored.read_from(&mut s).unwrap();

    assert_eq!(addr_orig, addr_restored);
}

#[test]
fn test_serialize_intermediate_addr_regular(){
    check_serialization_intermediate_addr_regular(IntermediateAddressRegular::with_use_src_bits(0).unwrap());
    check_serialization_intermediate_addr_regular(IntermediateAddressRegular::with_use_src_bits(1).unwrap());
    check_serialization_intermediate_addr_regular(IntermediateAddressRegular::with_use_src_bits(54).unwrap());
    check_serialization_intermediate_addr_regular(IntermediateAddressRegular::with_use_src_bits(96).unwrap());
}

#[test]
fn test_intermediate_addr_regular_cons(){
    IntermediateAddressRegular::with_use_src_bits(97).expect_err("must not allow more than 96");
    IntermediateAddressRegular::with_use_dest_bits(97).expect_err("must not allow more than 96");
}


#[test]
fn test_intermediate_addr_regular_set(){
    let mut a = IntermediateAddressRegular::with_use_src_bits(0).unwrap();
    a.set_use_src_bits(97).expect_err("must not allow more than 96");
}

fn check_serialization_intermediate_addr_simple(addr_orig: IntermediateAddressSimple){
    let mut b = BuilderData::new();
    addr_orig.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    let mut addr_restored = IntermediateAddressSimple::default();
    addr_restored.read_from(&mut s).unwrap();

    assert_eq!(addr_orig, addr_restored);
}

#[test]
fn test_serialize_intermediate_addr_simple(){
    check_serialization_intermediate_addr_simple(IntermediateAddressSimple::with_addr(-1, 0x0102030405060708));
    check_serialization_intermediate_addr_simple(IntermediateAddressSimple::with_addr(0, 0xFF_FF_FF_FF_FF_FF_FF_FF));
    check_serialization_intermediate_addr_simple(IntermediateAddressSimple::with_addr(1, 0));
    check_serialization_intermediate_addr_simple(IntermediateAddressSimple::with_addr(127, 0xCD_CD_CD_CD_CD_CD_CD_CD));
}

fn check_serialization_intermediate_addr_ext(addr_orig: IntermediateAddressExt){
    let mut b = BuilderData::new();
    addr_orig.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    let mut addr_restored = IntermediateAddressExt::default();
    addr_restored.read_from(&mut s).unwrap();

    assert_eq!(addr_orig, addr_restored);
}

#[test]
fn test_serialize_intermediate_addr_ext(){
    check_serialization_intermediate_addr_ext(IntermediateAddressExt::with_addr(-1, 0x0102030405060708));
    check_serialization_intermediate_addr_ext(IntermediateAddressExt::with_addr(0, 0xFF_FF_FF_FF_FF_FF_FF_FF));
    check_serialization_intermediate_addr_ext(IntermediateAddressExt::with_addr(1, 0));
    check_serialization_intermediate_addr_ext(IntermediateAddressExt::with_addr(3462346, 0xCD_CD_CD_CD_CD_CD_CD_CD));
}

fn check_serialization_intermediate_addr(addr_orig: IntermediateAddress){
    let mut b = BuilderData::new();
    addr_orig.write_to(&mut b).unwrap();
    let mut s = SliceData::load_builder(b).unwrap();
    let mut addr_restored = IntermediateAddress::default();
    addr_restored.read_from(&mut s).unwrap();

    assert_eq!(addr_orig, addr_restored);
}

#[test]
fn test_serialize_inpermediate_address(){
    check_serialization_intermediate_addr(IntermediateAddress::Regular(IntermediateAddressRegular::with_use_src_bits(0).unwrap()));
    check_serialization_intermediate_addr(IntermediateAddress::Regular(IntermediateAddressRegular::with_use_src_bits(96).unwrap()));

    check_serialization_intermediate_addr(IntermediateAddress::Simple(IntermediateAddressSimple::with_addr(-1, 0x0102030405060708)));
    check_serialization_intermediate_addr(IntermediateAddress::Simple(IntermediateAddressSimple::with_addr(1, 0xFE_FE_FE_FE_FE_FE_FE_FE)));

    check_serialization_intermediate_addr(IntermediateAddress::Ext(IntermediateAddressExt::with_addr(-1, 0x0102030405060708)));
    check_serialization_intermediate_addr(IntermediateAddress::Ext(IntermediateAddressExt::with_addr(1, 0xCD_CD_CD_CD_CD_CD_CD_CD)));

}

fn gen_big_message() -> Message {

    let mut msg = Message::with_int_header(InternalMessageHeader::default());
    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let mut code = SliceData::new(
        vec![
            0b00111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
            0b11111111, 0b11110100
        ]
    );
    stinit.set_code(code.clone().into_cell());
    let mut code1 = SliceData::new(
        vec![
            0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 
            0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 
            0x15, 0x1a, 0x32, 0x03, 0x69, 0x4a, 0xff, 
            0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 
            0x8c, 0xc8, 0x10, 0xfb, 0x6b, 0x5b, 0x51
        ]
    );
    let mut code2 = SliceData::new(
        vec![
            0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 
            0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 
            0x15, 0x1a, 0x32, 0x03, 0x69, 0x4a, 0xff, 
            0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 
            0x8c, 0xc8, 0x10, 0xfb, 0x6b, 0x5b, 0x51
        ]
    );
    let code3 = SliceData::new(
        vec![
            0xad, 0xc9, 0xba, 0xfc, 0x56, 0x94, 0x11, 
            0x56, 0x58, 0xfa, 0x2b, 0xdf, 0xe4, 0x65, 
            0x15, 0x1a, 0x32, 0x03, 0x69, 0x4a, 0xff, 
            0xcd, 0x00, 0x8f, 0x36, 0x8b, 0xd2, 0xcc, 
            0x8c, 0xc8, 0x10, 0xfb, 0x6b, 0x5b, 0x51
        ]
    );
    code2.append_reference(code3);
    code1.append_reference(code2);
    code.append_reference(code1);

    stinit.set_code(code.clone().into_cell());

    let data = SliceData::new(
        vec![
            0b00111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 
            0b11111111, 0b11110100
        ]
    );
    stinit.set_data(data.into_cell());
    let library = SliceData::new(
        vec![
            0b00111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
            0b11111111, 0b11110100
        ]
    );
    stinit.set_library_code(library.into_cell(), false).unwrap();
    
    let mut body = SliceData::new(
        vec![
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,
            0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0x80
        ]
    ).into_builder();
    let mut body1 = SliceData::new(
        vec![
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,
            0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0xFE,0x80
        ]
    ).into_builder();
    let body2 = SliceData::new(
        vec![
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,
            0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0xA6,0x80
        ]
    ).into_builder();

    body1.checked_append_reference(body2.into_cell().unwrap()).unwrap();
    body.checked_append_reference(body1.into_cell().unwrap()).unwrap();

    msg.set_state_init(stinit);
    msg.set_body(SliceData::load_builder(body).unwrap());
    msg

}

#[test]
fn test_serialization_msg_envelope(){
    write_read_and_assert(MsgEnvelope::default());
    
    let mut msg = MsgEnvelope::with_message_and_fee(
        &gen_big_message(),
        12312.into()
    ).unwrap();

    write_read_and_assert(msg.clone());

    msg.set_cur_addr(IntermediateAddress::Simple(IntermediateAddressSimple::with_addr(-1, 0x0102030405060708)))
        .set_next_addr(IntermediateAddress::Simple(IntermediateAddressSimple::with_addr(-1, 0x0102030405060708)));

    write_read_and_assert(msg.clone());

    assert!(msg.collect_fee(123.into()));

    write_read_and_assert(msg.clone());
}

// prepare for testing purposes
fn prepare_test_env_message(src_prefix: u64, dst_prefix: u64, bits: u8, at: u32, lt: u64) -> Result<(Message, MsgEnvelope)> {
    let shard = ShardIdent::with_prefix_len(bits, 0, src_prefix)?;
    let src = UInt256::from_le_bytes(&src_prefix.to_be_bytes());
    let dst = UInt256::from_le_bytes(&dst_prefix.to_be_bytes());
    let src = MsgAddressInt::with_standart(None, 0, src.into())?;
    let dst = MsgAddressInt::with_standart(None, 0, dst.into())?;

    // let src_prefix = AccountIdPrefixFull::prefix(&src).unwrap();
    // let dst_prefix = AccountIdPrefixFull::prefix(&dst).unwrap();
    // let ia = IntermediateAddress::full_src();
    // let route_info = src_prefix.perform_hypercube_routing(&dst_prefix, &shard, ia)?.unwrap();
    // let cur_prefix  = src_prefix.interpolate_addr_intermediate(&dst_prefix, &route_info.0)?;
    // let next_prefix = src_prefix.interpolate_addr_intermediate(&dst_prefix, &route_info.1)?;

    let hdr = InternalMessageHeader::with_addresses(src, dst, CurrencyCollection::with_grams(1_000_000_000));
    let mut msg = Message::with_int_header(hdr);
    msg.set_at_and_lt(at, lt);

    let env = MsgEnvelope::hypercube_routing(&msg, &shard, 1_000_000.into())?;
    Ok((msg , env))
}

#[test]
fn test_prepare_msg_envelope() {
    let (msg, env) = prepare_test_env_message(0xd78b3fd904191a09, 0x9dd300cee029b9c7, 4, 0, 0).unwrap();
    let src = msg.src_ref().ok_or_else(|| error!("source address of message {:x} is invalid", env.message_hash())).unwrap();
    let src_prefix = AccountIdPrefixFull::prefix(src).unwrap();
    let dst = msg.dst_ref().ok_or_else(|| error!("destination address of message {:x} is invalid", env.message_hash())).unwrap();
    let dst_prefix = AccountIdPrefixFull::prefix(dst).unwrap();
    assert_eq!(src_prefix, AccountIdPrefixFull::workchain(0, 0xd78b3fd904191a09));
    assert_eq!(dst_prefix, AccountIdPrefixFull::workchain(0, 0x9dd300cee029b9c7));

    let (cur_prefix, next_prefix) = env.calc_cur_next_prefix().unwrap();
    assert_eq!(cur_prefix,  AccountIdPrefixFull::workchain(0, 0xd78b3fd904191a09));
    assert_eq!(next_prefix, AccountIdPrefixFull::workchain(0, 0x978b3fd904191a09));

    let src_shard = ShardIdent::with_tagged_prefix(0, 0xD800000000000000).unwrap();
    src_prefix.perform_hypercube_routing(&dst_prefix, &src_shard, IntermediateAddress::default()).unwrap();
}

#[test]
fn test_routing_with_hop() {
    let pfx_len = 12;
    let src = 0xd78b3fd904191a09;
    let dst = 0xd4d300cee029b9c7;
    let hop = 0xd48b3fd904191a09;
    let (msg, env) = prepare_test_env_message(src, dst, pfx_len, 0, 0).unwrap();
    let src_shard_id = ShardIdent::with_prefix_len(pfx_len, 0, src).unwrap();
    let dst_shard_id = ShardIdent::with_prefix_len(pfx_len, 0, dst).unwrap();
    let hop_shard_id = ShardIdent::with_prefix_len(pfx_len, 0, hop).unwrap();
    let src_addr = msg.src_ref().ok_or_else(|| error!("source address of message {:x} is invalid", env.message_hash())).unwrap();
    let src_prefix = AccountIdPrefixFull::prefix(src_addr).unwrap();
    let dst_addr = msg.dst_ref().ok_or_else(|| error!("destination address of message {:x} is invalid", env.message_hash())).unwrap();
    let dst_prefix = AccountIdPrefixFull::prefix(dst_addr).unwrap();
    assert!(src_shard_id.contains_full_prefix(&src_prefix));
    assert!(dst_shard_id.contains_full_prefix(&dst_prefix));
    
    assert_eq!(src_prefix, AccountIdPrefixFull::workchain(0, src));
    assert_eq!(dst_prefix, AccountIdPrefixFull::workchain(0, dst));

    let (cur_prefix, next_prefix) = env.calc_cur_next_prefix().unwrap();

    assert_eq!(src_prefix, cur_prefix);
    assert_ne!(dst_prefix, next_prefix);
    assert!(src_shard_id.contains_full_prefix(&cur_prefix));
    println!("shard: {}, prefix: {:x}", hop_shard_id, next_prefix.prefix);
    assert!(hop_shard_id.contains_full_prefix(&next_prefix));
    assert!(!dst_shard_id.contains_full_prefix(&next_prefix));

    assert_eq!(cur_prefix,  AccountIdPrefixFull::workchain(0, src));
    assert_eq!(next_prefix, AccountIdPrefixFull::workchain(0, hop));

    src_prefix.perform_hypercube_routing(&dst_prefix, &src_shard_id, IntermediateAddress::default()).unwrap();
    let route_info = next_prefix.perform_hypercube_routing(&dst_prefix, &hop_shard_id, IntermediateAddress::default()).unwrap();
    let prefix = next_prefix.interpolate_addr_intermediate(&dst_prefix, &route_info.0).unwrap();
    assert_eq!(prefix, next_prefix);
    let prefix = next_prefix.interpolate_addr_intermediate(&dst_prefix, &route_info.1).unwrap();
    println!("shard: {}, prefix: {:x}", dst_shard_id, prefix.prefix);
    assert!(dst_shard_id.contains_full_prefix(&prefix));
    println!("dst_prefix: {:x}, prefix: {:x}", dst_prefix.prefix, prefix.prefix);
    assert_ne!(prefix, dst_prefix);
}

#[test]
fn test_intermediate_addr_default() {
    assert_eq!(IntermediateAddress::default(), IntermediateAddress::full_src());
}
