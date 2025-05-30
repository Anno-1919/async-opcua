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
///https://reference.opcfoundation.org/v105/Core/docs/Part4/5.10.4/#5.10.4.2
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueryNextResponse {
    pub response_header: opcua::types::response_header::ResponseHeader,
    pub query_data_sets: Option<Vec<super::query_data_set::QueryDataSet>>,
    pub revised_continuation_point: opcua::types::ContinuationPoint,
}
impl opcua::types::MessageInfo for QueryNextResponse {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::QueryNextResponse_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::QueryNextResponse_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::QueryNextResponse_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::QueryNextResponse
    }
}
