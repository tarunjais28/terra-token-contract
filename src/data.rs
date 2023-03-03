use super::*;
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Encode, Decode)]
pub struct ProposalData {
    pub amount: u128,
    pub recipient_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Encode, Decode)]
pub struct WithdrawData {
    pub token_address: String,
    pub recipient_address: String,
    pub amount: u128,
}
