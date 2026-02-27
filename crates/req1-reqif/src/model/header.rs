use serde::{Deserialize, Serialize};

use super::common::ReqifDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "REQ-IF-HEADER")]
pub struct ReqIfHeader {
    #[serde(rename = "@IDENTIFIER")]
    pub identifier: String,

    #[serde(rename = "COMMENT", skip_serializing_if = "Option::is_none", default)]
    pub comment: Option<String>,

    #[serde(rename = "CREATION-TIME", skip_serializing_if = "Option::is_none", default)]
    pub creation_time: Option<ReqifDateTime>,

    #[serde(rename = "REPOSITORY-ID", skip_serializing_if = "Option::is_none", default)]
    pub repository_id: Option<String>,

    #[serde(rename = "REQ-IF-TOOL-ID", skip_serializing_if = "Option::is_none", default)]
    pub req_if_tool_id: Option<String>,

    #[serde(rename = "REQ-IF-VERSION", skip_serializing_if = "Option::is_none", default)]
    pub req_if_version: Option<String>,

    #[serde(rename = "SOURCE-TOOL-ID", skip_serializing_if = "Option::is_none", default)]
    pub source_tool_id: Option<String>,

    #[serde(rename = "TITLE", skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
}
