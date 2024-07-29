#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, Page, QueryMsg};
use crate::state::{RegistryItem, ADMIN, REGISTRY};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ambur-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

static LIMIT: u32 = 50;
static MAX_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ADMIN.save(deps.storage, msg.admin.as_ref().unwrap_or(&info.sender))?;

    for item in msg.registry {
        REGISTRY.save(deps.storage, item.cw721, &item.minter)?;
    }

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // All msgs are permissioned so we do the check here
    if info.sender != ADMIN.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    Ok(match msg {
        ExecuteMsg::Register { cw721, minter } => {
            REGISTRY.save(deps.storage, cw721, &minter)?;
            Response::new().add_attribute("action", "register")
        }
        ExecuteMsg::Unregister { cw721 } => {
            REGISTRY.remove(deps.storage, cw721);
            Response::new().add_attribute("action", "unregister")
        }
        ExecuteMsg::SetAdmin { admin } => {
            ADMIN.save(deps.storage, &admin)?;
            Response::new().add_attribute("action", "set_admin")
        }
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&ADMIN.load(deps.storage)?),
        QueryMsg::Registry { limit, page } => {
            let limit = limit.unwrap_or(LIMIT).min(MAX_LIMIT);
            let page = page.unwrap_or(0);

            let items: Vec<(Addr, Addr)> = REGISTRY
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<(Addr, Addr)>>>()?;
            let start = limit * page;
            let end = (start + limit).min(items.len() as u32);

            to_json_binary(&if start >= end {
                Page {
                    registered: vec![],
                    page,
                    total: 0,
                }
            } else {
                let registered: Vec<RegistryItem> = items[start as usize..end as usize]
                    .iter()
                    .map(|(cw721, minter)| RegistryItem::new(cw721.clone(), minter.clone()))
                    .collect();
                let total = registered.len() as u32;

                Page {
                    registered,
                    page,
                    total,
                }
            })
        }
        QueryMsg::Registered { cw721 } => to_json_binary(&REGISTRY.has(deps.storage, cw721)),
    }
}
