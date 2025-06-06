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
///https://reference.opcfoundation.org/v105/Core/docs/Part4/5.13.2/#5.13.2.2
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MonitoredItemCreateRequest {
    pub item_to_monitor: super::read_value_id::ReadValueId,
    pub monitoring_mode: super::enums::MonitoringMode,
    pub requested_parameters: super::monitoring_parameters::MonitoringParameters,
}
impl opcua::types::MessageInfo for MonitoredItemCreateRequest {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::MonitoredItemCreateRequest_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::MonitoredItemCreateRequest_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::MonitoredItemCreateRequest_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::MonitoredItemCreateRequest
    }
}
