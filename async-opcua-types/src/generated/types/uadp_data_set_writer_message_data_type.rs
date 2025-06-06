// This file was autogenerated from schemas/1.05/Opc.Ua.NodeSet2.Services.xml by async-opcua-codegen
//
// DO NOT EDIT THIS FILE

// OPCUA for Rust
// SPDX-License-Identifier: MPL-2.0
// Copyright (C) 2017-2024 Adam Lock, Einar Omang
#[allow(unused)]
mod opcua {
    pub(super) use crate as types;
}
#[opcua::types::ua_encodable]
///https://reference.opcfoundation.org/v105/Core/docs/Part14/6.3.1/#6.3.1.3.6
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UadpDataSetWriterMessageDataType {
    pub data_set_message_content_mask: super::enums::UadpDataSetMessageContentMask,
    pub configured_size: u16,
    pub network_message_number: u16,
    pub data_set_offset: u16,
}
impl opcua::types::MessageInfo for UadpDataSetWriterMessageDataType {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UadpDataSetWriterMessageDataType_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UadpDataSetWriterMessageDataType_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UadpDataSetWriterMessageDataType_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::UadpDataSetWriterMessageDataType
    }
}
