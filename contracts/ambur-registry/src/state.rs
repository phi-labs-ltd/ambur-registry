use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct RegistryItem {
    pub minter: Addr,
    pub cw721: Addr,
}

impl RegistryItem {
    pub fn new(cw721: Addr, minter: Addr) -> Self {
        Self { cw721, minter }
    }
}

pub static ADMIN: Item<Addr> = Item::new("admin");
// Key is Collection and item is minter
pub static REGISTRY: Map<Addr, Addr> = Map::new("collections");
