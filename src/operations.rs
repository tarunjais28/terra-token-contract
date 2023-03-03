use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ListType {
    WhiteList,
    BurnList,
}

impl ListType {
    pub fn get_action(&self) -> &str {
        match self {
            ListType::WhiteList => "add_to_whitelist",
            ListType::BurnList => "add_to_burnlist",
        }
    }

    pub fn get_addr_type(&self) -> &str {
        match self {
            ListType::WhiteList => "whitelist_address",
            ListType::BurnList => "burnlist_address",
        }
    }
}
