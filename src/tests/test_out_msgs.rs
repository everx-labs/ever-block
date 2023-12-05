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

use super::*;
use crate::{
    AccountStatus, HashUpdate, InMsgExternal, InternalMessageHeader, MsgAddressInt, 
    StateInit, TickTock, TransactionDescr, write_read_and_assert,
    types::{Grams, Number5}
};
#[allow(unused_imports)] // TBD when types fixed
use std::str::FromStr;

fn get_message_with_addrs(src: AccountId, dst: AccountId) -> Message
{
    let mut msg = Message::with_int_header(
        InternalMessageHeader::with_addresses(
            MsgAddressInt::with_standart( None, 0, src).unwrap(),
            MsgAddressInt::with_standart( None, 0, dst).unwrap(),
            CurrencyCollection::default())
    );
    
    let mut stinit = StateInit::default();
    stinit.set_split_depth(Number5::new(23).unwrap());
    stinit.set_special(TickTock::with_values(false, true));
    let code = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_code(code.into_cell());
    let data = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_data(data.into_cell());
    let library = SliceData::new(vec![0b00111111, 0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110100]);
    stinit.set_library_code(library.into_cell(), false).unwrap();
    
    msg.set_state_init(stinit);

    msg
}

fn get_message() -> Message {
    get_message_with_addrs(
        AccountId::from([0; 32]),
        AccountId::from([1; 32]),
    )
}

fn transaction() -> Transaction {

    let mut tr = Transaction::with_address_and_status(
        AccountId::from([1; 32]),
        AccountStatus::AccStateActive,
    );

    let s_in_msg = CommonMessage::Std(get_message());
    let s_out_msg1 = CommonMessage::Std(get_message());
    let s_out_msg2 = CommonMessage::Std(get_message());
    let s_out_msg3 = CommonMessage::Std(get_message());

    let s_status_update = HashUpdate::default();
    let s_tr_desc = TransactionDescr::default();

    tr.set_logical_time(123423);
    tr.set_end_status(AccountStatus::AccStateFrozen);
    tr.set_total_fees(CurrencyCollection::with_grams(653));
    tr.write_in_msg(Some(&s_in_msg)).unwrap();
    tr.add_out_message(&s_out_msg1).unwrap();
    tr.add_out_message(&s_out_msg2).unwrap();
    tr.add_out_message(&s_out_msg3).unwrap();
    tr.write_state_update(&s_status_update).unwrap();
    tr.write_description(&s_tr_desc).unwrap();
    tr
}

fn get_out_ext_msg() -> OutMsg {
    let tr_cell = ChildCell::with_struct(&transaction()).unwrap();
    let msg_cell = ChildCell::with_struct(
        &CommonMessage::Std(get_message())
    ).unwrap();
    OutMsg::external(msg_cell, tr_cell)
}

#[test]
fn test_out_msg_external_serialization()
{
    let mut b = BuilderData::new();
    let out_msg = get_out_ext_msg();

    out_msg.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    println!("out_msg slice = {}", s);

    let mut out_msg_restored = OutMsg::None;
    out_msg_restored.read_from(&mut s).unwrap();

    assert_eq!(out_msg, out_msg_restored);
}

#[test]
fn test_out_msg_immediately_serialization() {
    let mut b = BuilderData::new();
    let out_msg = OutMsg::immediate(
        ChildCell::with_struct(&MsgEnvelope::default()).unwrap(),
        ChildCell::with_struct(&transaction()).unwrap(),
        ChildCell::with_struct(&InMsg::External(InMsgExternal::default())).unwrap(),
    );

    out_msg.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    println!("out_msg slice = {}", s);

    let mut out_msg_restored = OutMsg::None;
    out_msg_restored.read_from(&mut s).unwrap();

    assert_eq!(out_msg, out_msg_restored);
}

#[test]
fn test_out_msg_new_serialization() {
    let tr_cell = ChildCell::with_struct(&transaction()).unwrap();
    let env_cell = ChildCell::with_struct(&MsgEnvelope::default()).unwrap();
    let out_msg = OutMsg::new(env_cell, tr_cell);
    write_read_and_assert(out_msg);
}

#[test]
fn test_out_msg_transit_serialization()
{
    let mut b = BuilderData::new();
    let out_msg = OutMsg::transit(
        ChildCell::with_struct(&MsgEnvelope::default()).unwrap(),
        ChildCell::with_struct(&InMsg::External(InMsgExternal::default())).unwrap(),
        false,
    );

    out_msg.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    println!("out_msg slice = {}", s);

    let mut out_msg_restored = OutMsg::None;
    out_msg_restored.read_from(&mut s).unwrap();

    assert_eq!(out_msg, out_msg_restored);
}

#[test]
fn test_out_msg_dequeue_serialization()
{
    let mut b = BuilderData::new();
    let out_msg = OutMsg::Dequeue(
        OutMsgDequeue::with_cells(
            ChildCell::with_struct(&MsgEnvelope::default()).unwrap(),
            243563457456709,
    ));

    out_msg.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    println!("out_msg slice = {}", s);

    let mut out_msg_restored = OutMsg::None;
    out_msg_restored.read_from(&mut s).unwrap();

    assert_eq!(out_msg, out_msg_restored);
}

#[test]
fn test_out_msg_dequeue_short_serialization()
{
    let mut b = BuilderData::new();
    let out_msg = OutMsg::DequeueShort(
        OutMsgDequeueShort {
            msg_env_hash: UInt256::from_str("b44798875f5c390ea9d405b653abb213fb25c108ddd316ccfbb10df2558d6e6c").unwrap(),
            next_workchain: -1,
            next_addr_pfx: 238798479,
            import_block_lt: 1000234234,
        }
    );

    out_msg.write_to(&mut b).unwrap();

    let mut s = SliceData::load_builder(b).unwrap();

    println!("out_msg slice = {}", s);

    let mut out_msg_restored = OutMsg::None;
    out_msg_restored.read_from(&mut s).unwrap();

    assert_eq!(out_msg, out_msg_restored);
}

#[test]
fn test_serialization_out_msg_descr()
{
    let mut desc = OutMsgDescr::default();
    
    for _ in 0..10 {
        desc.insert(&get_out_ext_msg()).unwrap();
    }

    write_read_and_assert(desc);
}

#[test]
fn test_serialization_out_msg_queue()
{
    let mut queue = OutMsgQueue::default();
    
    for n in 0..100 {
        let msg = get_message();
        let out_msg_env = MsgEnvelope::with_message_and_fee(&msg, Grams::one()).unwrap();
        queue.insert(0, n, &out_msg_env, 11).unwrap();
    }

    println!("{:?}", queue);
    write_read_and_assert(queue);
}

fn create_account_id(n: u8) -> AccountId{
    AccountId::from([0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,0,
                    0,0,0,0,0,0,0,n])
}

#[test]
fn test_work_with_out_msg_desc(){
    let tr = transaction();
    let tr_cell = ChildCell::with_struct(&tr).unwrap();
    let mut msg_desc = OutMsgDescr::default();

    // test OutMsg::External
    let msg = CommonMessage::Std(
        get_message_with_addrs(create_account_id(1), create_account_id(2))
    );
    let out_msg_ext = OutMsg::external(
        ChildCell::with_struct(&msg).unwrap(), 
        tr_cell.clone(),
    );

    msg_desc.insert(&out_msg_ext).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 1);

    let msg = CommonMessage::Std(
        get_message_with_addrs(create_account_id(2), create_account_id(1))
    );
    let out_msg_ext = OutMsg::external(
        ChildCell::with_struct(&msg).unwrap(),
        tr_cell.clone(),
    );

    msg_desc.insert(&out_msg_ext).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 2);

    // msg_desc.remove(out_msg_ext);

    // assert_eq!(msg_desc.len().unwrap(), 1);

    // test OutMsg::Immediate
    let msg = CommonMessage::Std(
        get_message_with_addrs(create_account_id(3), create_account_id(4))
    );
    let msg_in = InMsg::external(
        ChildCell::with_struct(&msg).unwrap(),
        tr_cell.clone(),
    );

    let env = MsgEnvelope::with_message_and_fee(
        &msg.get_std().unwrap(),
        Grams::one()
    ).unwrap();
    let out_msg = OutMsgImmediate::with_cells(
        ChildCell::with_struct(&env).unwrap(),
        tr_cell.clone(),
        ChildCell::with_struct(&msg_in).unwrap()
    );
    let out_msg_imm = OutMsg::Immediate(out_msg);

    msg_desc.insert(&out_msg_imm).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 3);


    // test OutMsg::OutMsgNew
    let msg = get_message_with_addrs(create_account_id(4), create_account_id(5));
    let env = MsgEnvelope::with_message_and_fee(&msg, Grams::one()).unwrap();

    let out_msg_new = OutMsg::new(
        ChildCell::with_struct(&env).unwrap(),
        tr_cell.clone(),
    );

    msg_desc.insert(&out_msg_new).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 4);

    // test OutMsg::OutMsgTransit
    let msg = CommonMessage::Std(
        get_message_with_addrs(create_account_id(5), create_account_id(6))
    );
    let msg_in = InMsg::external(
        ChildCell::with_struct(&msg).unwrap(),
        tr_cell.clone(),
    );

    let out_msg_transit = OutMsg::Transit(
        OutMsgTransit::with_cells(
            ChildCell::with_struct(
                &MsgEnvelope::with_message_and_fee(&msg.get_std().unwrap(), Grams::one()).unwrap()
            ).unwrap(),
            ChildCell::with_struct(&msg_in).unwrap()
        ),
    );

    msg_desc.insert(&out_msg_transit).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 5);

    // test OutMsg::OutMsgDequeue
    let msg = get_message_with_addrs(create_account_id(6), create_account_id(7));
    let env = MsgEnvelope::with_message_and_fee(&msg, Grams::one()).unwrap();
    let out_msg = OutMsgDequeue::with_cells(
        ChildCell::with_struct(&env).unwrap(), 
        32523,
    );
    let out_msg_dequeue = OutMsg::Dequeue(out_msg);

    msg_desc.insert(&out_msg_dequeue).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 6);

    // test OutMsg::OutMsgDequeueShort
    let out_msg_dequeue_short = OutMsg::DequeueShort(
        OutMsgDequeueShort {
            msg_env_hash: UInt256::from_str("b44798875f5c390ea9d405b653abb213fb25c108ddd316ccfbb10df2558d6e6c").unwrap(),
            next_workchain: -100,
            next_addr_pfx: 6,
            import_block_lt: 1234567890,
        }
    );

    let msg = get_message_with_addrs(create_account_id(7), create_account_id(8));
    let hash = msg.serialize().unwrap().repr_hash();
    msg_desc.insert_with_key(hash, &out_msg_dequeue_short).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 7);

    // test OutMsg::OutMsgTransitRequeued
    let msg = get_message_with_addrs(create_account_id(8), create_account_id(9));
    let msg_in = InMsg::external(
        ChildCell::with_cell(msg.serialize().unwrap()), 
        tr_cell.clone(),
    );
    let out_msg_transit = OutMsg::TransitRequeued(
        OutMsgTransitRequeued::with_cells(
            ChildCell::with_struct(&MsgEnvelope::with_message_and_fee(&msg, Grams::one()).unwrap()).unwrap(),
            ChildCell::with_struct(&msg_in).unwrap(),
        )
    );

    msg_desc.insert(&out_msg_transit).unwrap();
    assert_eq!(msg_desc.len().unwrap(), 8);
}


#[test]
fn test_out_msg_queue_and_info()
{
    let mut queue = OutMsgQueue::default();
    
    // test OutMsg::External
    let msg = get_message_with_addrs(create_account_id(1), create_account_id(2));
    let out_msg_env = MsgEnvelope::with_message_and_fee(&msg, Grams::one()).unwrap();

    queue.insert(0, 1, &out_msg_env, 11).unwrap();
    assert_eq!(queue.len().unwrap(), 1);

    write_read_and_assert(queue.clone());

    let omq_info = OutMsgQueueInfo::with_params(
        queue, ProcessedInfo::default(), IhrPendingInfo::default()
    );

    write_read_and_assert(omq_info);
}

#[test]
fn test_enqueued_msg() {
    
    let em1 = EnqueuedMsg::new();
    let em2 = EnqueuedMsg::default();
    assert_eq!(em1, em2);
    write_read_and_assert(em1);

    let em1 = EnqueuedMsg::with_param(
        234523452345, 
        &MsgEnvelope::with_message_and_fee(&Message::default(), 27348376.into()).unwrap()
    ).unwrap();
    let em2 = EnqueuedMsg::with_param(
        234523452346, 
        &MsgEnvelope::with_message_and_fee(&Message::default(), 27348377.into()).unwrap()
    ).unwrap();
    assert_ne!(em1, em2);

    write_read_and_assert(em1);
    write_read_and_assert(em2);
}

#[test]
fn test_outmsgdescr_common_msg() {
    let mut msg_descr = OutMsgDescr::with_serde_opts(SERDE_OPTS_COMMON_MESSAGE);
    assert_eq!(msg_descr.serde_opts(), SERDE_OPTS_COMMON_MESSAGE);
    let cmn_std_msg = CommonMessage::Std(Message::default());
    let enveloped = MsgEnvelope::with_common_msg_support(
        &cmn_std_msg,
        1.into(),
    ).unwrap();
    let mut tr = Transaction::with_common_msg_support(
        cmn_std_msg.get_std().unwrap().int_dst_account_id().unwrap()
    );
    tr.set_logical_time(1);
    tr.write_in_msg(Some(&cmn_std_msg)).unwrap();
    tr.orig_status = AccountStatus::AccStateActive;

    let out_msg = OutMsg::new(
        ChildCell::with_struct_and_opts(
                &enveloped,
                SERDE_OPTS_COMMON_MESSAGE,
            ).unwrap(),
        ChildCell::with_struct_and_opts(
            &tr,
            SERDE_OPTS_COMMON_MESSAGE
        ).unwrap(),
    );
    msg_descr.insert(&out_msg).unwrap();

    assert_eq!(msg_descr.serde_opts(), SERDE_OPTS_COMMON_MESSAGE);
    let cell = msg_descr.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert!(matches!(msg_descr.serialize(), Err(_)));

    let descr = OutMsgDescr::construct_from_cell_with_opts(cell.clone(), SERDE_OPTS_COMMON_MESSAGE).unwrap();
    let mut msg = None;
    let _ = descr.iterate_objects(|x| {
        let enveloped = x.read_out_message()?.unwrap();
        msg = Some(enveloped.read_common_message()?);
        Ok(true)
    }).unwrap();
    let msg = msg.unwrap();
    assert_eq!(msg.get_std().unwrap(), &Message::default());

    let descr = OutMsgDescr::construct_from_cell(cell).unwrap();
    assert_eq!(descr.serde_opts(), SERDE_OPTS_EMPTY);
    assert!(matches!(descr.get(&out_msg.read_message_hash().unwrap()), Err(_)));
}

#[test]
fn test_outmsgdescr_with_cmnmsg_serialize_without_opts() {
    let msg_descr = OutMsgDescr::with_serde_opts(SERDE_OPTS_COMMON_MESSAGE);
    assert!(matches!(msg_descr.serialize(), Err(_)));
    msg_descr.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert!(matches!(msg_descr.serialize_with_opts(SERDE_OPTS_EMPTY), Err(_)));
}

#[test]
fn test_outmsg_serde_with_cmnmsg_success() {
    for opts in [SERDE_OPTS_COMMON_MESSAGE, SERDE_OPTS_EMPTY] {
        let msg = CommonMessage::default();
        let orig_status = AccountStatus::AccStateActive;
        let acc_id = AccountId::from([1; 32]);
        let tr = match opts {
            SERDE_OPTS_COMMON_MESSAGE => Transaction::with_common_msg_support(acc_id.clone()),
            SERDE_OPTS_EMPTY => Transaction::with_address_and_status(acc_id.clone(), orig_status),
            _ => unreachable!(),
        };
        let enveloped = match opts {
            SERDE_OPTS_COMMON_MESSAGE => MsgEnvelope::with_common_msg_support(&msg, 10.into()).unwrap(),
            SERDE_OPTS_EMPTY => MsgEnvelope::with_message_and_fee(&msg.get_std().unwrap(), 10.into()).unwrap(),
            _ => unreachable!(),
        };
        let msg_cell = ChildCell::with_struct_and_opts(&msg, opts).unwrap();
        let tr_cell =  ChildCell::with_struct_and_opts(&tr, opts).unwrap();
        let env_cell = ChildCell::with_struct_and_opts(&enveloped, opts).unwrap();
        let reimport_msg = InMsg::external(msg_cell.clone(), tr_cell.clone());
        let reimport_msg_cell = ChildCell::with_struct_and_opts(&reimport_msg, opts).unwrap();

        let outmsg_variants = [
            OutMsg::external(msg_cell.clone(), tr_cell.clone()),
            OutMsg::new(env_cell.clone(), tr_cell.clone()),
            OutMsg::immediate(env_cell.clone(), tr_cell.clone(), reimport_msg_cell.clone()),
            OutMsg::transit(env_cell.clone(), reimport_msg_cell.clone(), true),
            OutMsg::transit(env_cell.clone(), reimport_msg_cell.clone(), false),
            OutMsg::dequeue_long(env_cell.clone(), 12345),
            OutMsg::dequeue_short(enveloped.message_hash(), &AccountIdPrefixFull::default(), 12345),
            OutMsg::dequeue_immediate(env_cell, reimport_msg_cell),
        ];
        for ref outmsg in outmsg_variants {
            let cell = outmsg.serialize_with_opts(opts).unwrap();
            let outmsg2 = OutMsg::construct_from_cell_with_opts(cell, opts).unwrap();
            assert_eq!(outmsg, &outmsg2);
            
            match outmsg {
                OutMsg::External(_) | OutMsg::DequeueShort(_) => (),
                _ => {
                    let msg_env2 = outmsg2.read_out_message().unwrap().unwrap();
                    let msg2 = msg_env2.read_message().unwrap();
                    assert_eq!(&msg2, msg.get_std().unwrap());

                    let msg3 = outmsg2.read_message().unwrap().unwrap();
                    assert_eq!(&msg3, msg.get_std().unwrap());
                }
            };
            match outmsg {
                OutMsg::External(_) | OutMsg::Immediate(_) | OutMsg::New(_) => {
                    let tr2 = outmsg2.read_transaction().unwrap().unwrap();
                    assert_eq!(tr2, tr);
                },
                _ => (),
            };

            match outmsg {
                OutMsg::Immediate(_) | OutMsg::Transit(_) | OutMsg::DequeueImmediate(_) | OutMsg::TransitRequeued(_) => {
                    let inmsg = outmsg2.read_reimport_message().unwrap().unwrap();
                    assert_eq!(inmsg, reimport_msg);
                },
                _ => (),
            };
        }
    }
}