use crate::{ChildCell, CommonMessage, Deserializable, Message, Serializable,
    SERDE_OPTS_COMMON_MESSAGE,
};

#[test]
fn test_serde_std() {
    let msg = CommonMessage::default();
    assert!(matches!(msg, CommonMessage::Std(_)));
    let cell = msg.serialize().unwrap();
    let msg2 = CommonMessage::construct_from_cell(cell).unwrap();
    assert!(matches!(msg2, CommonMessage::Std(_)));
}

#[test]
fn test_serde_mesh() {
    let msg = CommonMessage::default_mesh();
    assert!(matches!(msg, CommonMessage::Mesh(_)));
    assert!(matches!(msg.serialize(), Err(_)));
    let cell = msg.serialize_with_opts(SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert!(matches!(
        CommonMessage::construct_from_cell(cell.clone()),
        Err(_)
    ));
    let msg2 = CommonMessage::construct_from_cell_with_opts(cell, SERDE_OPTS_COMMON_MESSAGE).unwrap();
    assert!(matches!(msg2, CommonMessage::Mesh(_)));
}

#[test]
fn test_childcell_new_format() {
    let msgcell = ChildCell::with_struct_and_opts(
        &CommonMessage::Std(Message::default()),
        SERDE_OPTS_COMMON_MESSAGE,
    )
    .unwrap();
    let msg1 = msgcell.read_struct().unwrap();
    assert!(matches!(msg1, CommonMessage::Std(_)));
    assert!(matches!(
        CommonMessage::construct_from_cell(msgcell.cell()),
        Err(_)
    ));
    assert!(matches!(
        Message::construct_from_cell(msgcell.cell()),
        Err(_)
    ));
    assert_eq!(*msg1.get_std().unwrap(), Message::default());
    let msg3 = CommonMessage::construct_from_cell_with_opts(msgcell.cell(), SERDE_OPTS_COMMON_MESSAGE)
        .unwrap();
    assert_eq!(msg3, msg1);
}

#[test]
fn test_childcell_old_format() {
    let msgcell = ChildCell::with_struct(
        &CommonMessage::Std(Message::default()),
    )
    .unwrap();
    let msg1 = msgcell.read_struct().unwrap();
    assert!(matches!(msg1, CommonMessage::Std(_)));
    let msg2 = CommonMessage::construct_from_cell(msgcell.cell()).unwrap();
    let msg3 = Message::construct_from_cell(msgcell.cell()).unwrap();
    assert_eq!(*msg1.get_std().unwrap(), msg3);
    assert_eq!(*msg2.get_std().unwrap(), msg3);
    assert!(matches!(
        CommonMessage::construct_from_cell_with_opts(msgcell.cell(), SERDE_OPTS_COMMON_MESSAGE),
        Err(_)
    ));
}
