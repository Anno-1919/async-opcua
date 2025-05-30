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
///https://reference.opcfoundation.org/v105/Core/docs/Part5/12.36
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UABinaryFileDataType {
    pub namespaces: Option<Vec<opcua::types::string::UAString>>,
    pub structure_data_types: Option<Vec<super::structure_description::StructureDescription>>,
    pub enum_data_types: Option<Vec<super::enum_description::EnumDescription>>,
    pub simple_data_types: Option<Vec<super::simple_type_description::SimpleTypeDescription>>,
    pub schema_location: opcua::types::string::UAString,
    pub file_header: Option<Vec<super::key_value_pair::KeyValuePair>>,
    pub body: opcua::types::variant::Variant,
}
impl opcua::types::MessageInfo for UABinaryFileDataType {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UABinaryFileDataType_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UABinaryFileDataType_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::UABinaryFileDataType_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::UABinaryFileDataType
    }
}
