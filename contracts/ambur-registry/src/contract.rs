#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Order, QueryRequest,
    Reply, Response, StdResult, SubMsgResult, WasmQuery,
};
use cw2::{get_contract_version, set_contract_version};
use cw721_base::msg::QueryMsg as Cw721QueryMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, Page, QueryMsg};
use crate::state::{RegistryItem, ADMIN, REGISTRY};

pub type Extension = Option<Empty>;

// version info for migration info
const CONTRACT_NAME: &str = "ambur-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

    let mut pre_registered: Vec<RegistryItem> = vec![];

    for cw721 in msg.registry {
        let query_msg: cw721_base::QueryMsg<Extension> = Cw721QueryMsg::Minter {};
        let query_req = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cw721.clone().into(),
            msg: to_json_binary(&query_msg).unwrap(),
        });
        let minter: Addr = deps.querier.query(&query_req)?;

        REGISTRY.save(deps.storage, cw721.clone(), &minter)?;

        pre_registered.push(RegistryItem { cw721, minter });
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let json_pre_registered: String =
        serde_json_wasm::to_string(&pre_registered).unwrap_or_default();

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("registered", json_pre_registered))
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

    match msg {
        ExecuteMsg::Register(msg) => {
            let query_msg: cw721_base::QueryMsg<Extension> = Cw721QueryMsg::Minter {};
            let query_req = QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: msg.cw721.clone().into(),
                msg: to_json_binary(&query_msg).unwrap(),
            });
            let minter: Addr = deps.querier.query(&query_req)?;

            REGISTRY.save(deps.storage, msg.cw721.clone(), &minter)?;

            Ok(Response::new()
                .add_attribute("action", "register")
                .add_attribute("cw721", msg.cw721.to_string())
                .add_attribute("minter", minter.to_string()))
        }
        ExecuteMsg::Unregister(msg) => {
            let minter: Addr = REGISTRY
                .load(deps.storage, msg.cw721.clone())
                .unwrap_or(Addr::unchecked(""));
            REGISTRY.remove(deps.storage, msg.cw721.clone());

            Ok(Response::new()
                .add_attribute("action", "unregister")
                .add_attribute("cw721", msg.cw721.to_string())
                .add_attribute("minter", minter.to_string()))
        }
        ExecuteMsg::SetAdmin(msg) => {
            ADMIN.save(deps.storage, &msg.admin)?;

            Ok(Response::new()
                .add_attribute("action", "set_admin")
                .add_attribute("admin", msg.admin.to_string()))
        }
    }
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(_) => Ok(Response::default()),
        SubMsgResult::Err(_) => Err(ContractError::Unauthorized {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let original_version = get_contract_version(deps.storage)?;
    let name = CONTRACT_NAME.to_string();
    let version = CONTRACT_VERSION.to_string();
    if original_version.contract != name {
        return Err(ContractError::InvalidInput {});
    }
    if original_version.version >= version {
        return Err(ContractError::InvalidInput {});
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
