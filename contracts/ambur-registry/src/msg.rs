use crate::state::RegistryItem;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub registry: Vec<RegistryItem>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register { cw721: Addr, minter: Addr },
    Unregister { cw721: Addr },
    SetAdmin { admin: Addr },
}

#[cw_serde]
pub struct Page {
    pub registered: Vec<RegistryItem>,
    pub page: u32,
    pub total: u32,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Config {},
    #[returns(Page)]
    Registry {
        page: Option<u32>,
        limit: Option<u32>,
    },
    #[returns(bool)]
    Registered { cw721: Addr },
}
