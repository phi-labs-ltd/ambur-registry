use crate::state::RegistryItem;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub registry: Vec<Addr>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Register(RegisterMsg),
    Unregister(UnregisterMsg),
    SetAdmin(SetAdminMsg),
}

#[cw_serde]
pub struct RegisterMsg {
    pub cw721: Addr,
}

#[cw_serde]
pub struct UnregisterMsg {
    pub cw721: Addr,
}

#[cw_serde]
pub struct SetAdminMsg {
    pub admin: Addr,
}

#[cw_serde]
pub struct MigrateMsg {}

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
