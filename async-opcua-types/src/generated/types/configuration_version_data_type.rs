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
///https://reference.opcfoundation.org/v105/Core/docs/Part14/6.2.3/#6.2.3.2.6
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConfigurationVersionDataType {
    pub major_version: opcua::types::VersionTime,
    pub minor_version: opcua::types::VersionTime,
}
impl opcua::types::MessageInfo for ConfigurationVersionDataType {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::ConfigurationVersionDataType_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::ConfigurationVersionDataType_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::ConfigurationVersionDataType_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::ConfigurationVersionDataType
    }
}
