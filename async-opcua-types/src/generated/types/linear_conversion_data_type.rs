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
///https://reference.opcfoundation.org/v105/Core/docs/Part8/6.6.2
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LinearConversionDataType {
    pub initial_addend: f32,
    pub multiplicand: f32,
    pub divisor: f32,
    pub final_addend: f32,
}
impl opcua::types::MessageInfo for LinearConversionDataType {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::LinearConversionDataType_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::LinearConversionDataType_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::LinearConversionDataType_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::LinearConversionDataType
    }
}
