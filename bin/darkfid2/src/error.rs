/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2023 Dyne.org foundation
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use serde_json::Value;

use darkfi::rpc::jsonrpc::{ErrorCode::ServerError, JsonError, JsonResult};

/// Custom RPC errors available for darkfid.
/// Please sort them sensefully.
pub enum RpcError {
    // Transaction-related errors
    _TxSimulationFail = -32110,
    _TxBroadcastFail = -32111,

    // State-related errors,
    _NotSynced = -32120,
    UnknownSlot = -32121,

    // Parsing errors
    _ParseError = -32190,

    // Contract-related errors
    ContractZkasDbNotFound = -32200,
}

fn to_tuple(e: RpcError) -> (i64, String) {
    let msg = match e {
        // Transaction-related errors
        RpcError::_TxSimulationFail => "Failed simulating transaction state change",
        RpcError::_TxBroadcastFail => "Failed broadcasting transaction",
        // State-related errors
        RpcError::_NotSynced => "Blockchain is not synced",
        RpcError::UnknownSlot => "Did not find slot",
        // Parsing errors
        RpcError::_ParseError => "Parse error",
        // Contract-related errors
        RpcError::ContractZkasDbNotFound => "zkas database not found for given contract",
    };

    (e as i64, msg.to_string())
}

pub fn server_error(e: RpcError, id: Value, msg: Option<&str>) -> JsonResult {
    let (code, default_msg) = to_tuple(e);

    if let Some(message) = msg {
        return JsonError::new(ServerError(code), Some(message.to_string()), id).into()
    }

    JsonError::new(ServerError(code), Some(default_msg), id).into()
}
