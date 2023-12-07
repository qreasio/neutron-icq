use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, COUNT, LAST_REGISTERED_ADDR, OBJECTS, REGISTERED_QUERIES};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply,
    Response, StdError, SubMsg,
};
use neutron_sdk::{
    bindings::{
        msg::{MsgRegisterInterchainQueryResponse, NeutronMsg},
        query::NeutronQuery,
    },
    interchain_queries::v045::new_register_balance_query_msg,
    interchain_queries::v045::queries::query_balance,
    sudo::msg::SudoMsg,
    NeutronError, NeutronResult,
};

/// Reply ID used to tell this kind of reply call apart.
pub const REGISTER_BALANCES_ICQ_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> NeutronResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            asset_denom: msg.asset_denom,
            frequency: msg.frequency,
            connection_id: msg.connection_id,
        },
    )?;

    let count: u64 = 0;
    COUNT.save(deps.storage, &count)?;
    REGISTERED_QUERIES.save(deps.storage, &vec![])?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> NeutronResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::RegisterAddr { addr } => register_addr(deps, addr),
    }
}

/// Registers a balance ICQ for a given address.
pub fn register_addr(
    deps: DepsMut<NeutronQuery>,
    addr: String,
) -> NeutronResult<Response<NeutronMsg>> {
    // Save a given address to LAST_REGISTERED_ADDR to handle it later in Reply handler
    LAST_REGISTERED_ADDR.save(deps.storage, &Addr::unchecked(addr.clone()))?;

    // Construct an ICQ registration message based on contract's config and passed arguments
    let conf: Config = CONFIG.load(deps.storage)?;
    let msg =
        new_register_balance_query_msg(conf.connection_id, addr, conf.asset_denom, conf.frequency)?;

    // Send the ICQ registration message as a submessage to receive a reply callback
    Ok(Response::new().add_submessage(SubMsg {
        id: REGISTER_BALANCES_ICQ_REPLY_ID,
        msg: CosmosMsg::Custom(msg),
        gas_limit: None,
        reply_on: cosmwasm_std::ReplyOn::Success,
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> NeutronResult<Binary> {
    match msg {
        QueryMsg::Balance { query_id } => query_the_balance(deps, env, query_id),
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::Count {} => query_count(deps),
        QueryMsg::Queries {} => query_registered_queries(deps),
        QueryMsg::Objects { query_id } => query_objects(deps, query_id),
    }
}

pub fn query_registered_queries(deps: Deps<NeutronQuery>) -> NeutronResult<Binary> {
    let queries = REGISTERED_QUERIES.load(deps.storage)?;
    Ok(to_binary(&queries)?)
}

pub fn query_objects(deps: Deps<NeutronQuery>, query_id: u64) -> NeutronResult<Binary> {
    let objects = OBJECTS.load(deps.storage, query_id)?;
    Ok(to_binary(&objects)?)
}

pub fn query_count(deps: Deps<NeutronQuery>) -> NeutronResult<Binary> {
    let counter = COUNT.load(deps.storage)?;
    Ok(to_binary(&counter)?)
}

/// Returns encoded current contract's configuration.
pub fn query_config(deps: Deps<NeutronQuery>) -> NeutronResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&config)?)
}

#[entry_point]
pub fn sudo(deps: DepsMut<NeutronQuery>, env: Env, msg: SudoMsg) -> NeutronResult<Response> {
    match msg {
        SudoMsg::KVQueryResult { query_id } => sudo_kv_query_result(deps, env, query_id),
        _ => Ok(Response::default()),
    }
}

/// Returns balance from the registered query
pub fn query_the_balance(
    deps: Deps<NeutronQuery>,
    env: Env,
    query_id: u64,
) -> NeutronResult<Binary> {
    let conf = CONFIG.load(deps.storage)?;
    let balance_resp = query_balance(deps, env.clone(), query_id)?;

    let balance = balance_resp
        .balances
        .coins
        .iter()
        .find(|b| b.denom == conf.asset_denom);

    Ok(to_binary(&balance)?)
}

/// Contract's callback for KV query results. 
pub fn sudo_kv_query_result(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _query_id: u64,
) -> NeutronResult<Response> {
    let mut count: u64 = COUNT.load(deps.storage)?;
    count += 1;
    COUNT.save(deps.storage, &count)?;
    Ok(Response::default())
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> NeutronResult<Response> {
    match msg.id {
        // Bind a given ICQ ID with the last registered address.
        REGISTER_BALANCES_ICQ_REPLY_ID => {
            // decode the reply msg payload as MsgRegisterInterchainQueryResponse
            let resp: MsgRegisterInterchainQueryResponse = serde_json_wasm::from_slice(
                msg.result
                    .into_result()
                    .map_err(StdError::generic_err)?
                    .data
                    .ok_or_else(|| StdError::generic_err("no result"))?
                    .as_slice(),
            )
            .map_err(|e| StdError::generic_err(format!("failed to parse response: {:?}", e)))?;

            // load the pre-set address used in the ICQ we just registered
            let last_registered_addr = LAST_REGISTERED_ADDR.load(deps.storage)?;
            OBJECTS.save(deps.storage, resp.id, &last_registered_addr)?;

            let mut queries = REGISTERED_QUERIES.load(deps.storage)?;
            queries.push(resp.id);
            REGISTERED_QUERIES.save(deps.storage, &queries)?;

            Ok(Response::new())
        }

        _ => Err(NeutronError::Std(StdError::generic_err(format!(
            "unsupported reply message id {}",
            msg.id
        )))),
    }
}
