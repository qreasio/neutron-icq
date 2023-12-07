use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Config of the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Count how many times the sudo_kv_query_result is called
pub const COUNT: Item<u64> = Item::new("count");

/// The last address passed to the balances ICQ registration message. Used in the reply handler.
pub const LAST_REGISTERED_ADDR: Item<Addr> = Item::new("last_registered_addr");

/// Contains all registered query ids
pub const REGISTERED_QUERIES: Item<Vec<u64>> = Item::new("registered_queries");

/// Contains addresses of all observed objects mapped to respective Interchain query ID.
pub const OBJECTS: Map<u64, Addr> = Map::new("objects");

/// Contains contract's configurable parameters.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
pub struct Config {
    /// Owner is capable of declaring new observed objects.
    pub owner: Addr,
    /// The asset which is taken into account when deciding whether one is rich or not.
    pub asset_denom: String,
    /// How often the balance is updated
    pub frequency: u64,
    /// Connection ID to identify a query associated IBC light client which will be used in
    /// crypto-proving the query results.
    pub connection_id: String,
}
