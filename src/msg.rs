use cosmwasm_schema::cw_serde;

/// InstantiateMsg contains initial configuration parameters for a new contract instance.
#[cw_serde]
pub struct InstantiateMsg {
    /// How often the balance is updated
    pub frequency: u64,
    /// Connection ID to identify a query associated IBC light client which will be used in
    /// crypto-proving the query results.
    pub connection_id: String,
    // Denom
    pub asset_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterAddr { addr: String },
}

#[cw_serde]
pub enum QueryMsg {
    /// Query balance from specific query id
    Balance {
        query_id: u64,
    },
    /// Query config
    Config {},
    /// Query count how many times sudo_kv_query_result is called
    Count {},
    /// Query address registered by specific query
    Objects {
        query_id: u64,
    },
    /// Query all registered query ids
    Queries {},
}
