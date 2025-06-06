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
///https://reference.opcfoundation.org/v105/Core/docs/Part4/5.10.3/#5.10.3.1
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueryFirstResponse {
    pub response_header: opcua::types::response_header::ResponseHeader,
    pub query_data_sets: Option<Vec<super::query_data_set::QueryDataSet>>,
    pub continuation_point: opcua::types::ContinuationPoint,
    pub parsing_results: Option<Vec<super::parsing_result::ParsingResult>>,
    pub diagnostic_infos: Option<Vec<opcua::types::diagnostic_info::DiagnosticInfo>>,
    pub filter_result: super::content_filter_result::ContentFilterResult,
}
impl opcua::types::MessageInfo for QueryFirstResponse {
    fn type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::QueryFirstResponse_Encoding_DefaultBinary
    }
    fn json_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::QueryFirstResponse_Encoding_DefaultJson
    }
    fn xml_type_id(&self) -> opcua::types::ObjectId {
        opcua::types::ObjectId::QueryFirstResponse_Encoding_DefaultXml
    }
    fn data_type_id(&self) -> opcua::types::DataTypeId {
        opcua::types::DataTypeId::QueryFirstResponse
    }
}
