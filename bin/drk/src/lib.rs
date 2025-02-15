/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2024 Dyne.org foundation
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

use std::{fs, sync::Arc};

use rusqlite::types::Value;
use url::Url;

use darkfi::{rpc::client::RpcClient, util::path::expand_path, Error, Result};

/// Error codes
pub mod error;
use error::{WalletDbError, WalletDbResult};

/// darkfid JSON-RPC related methods
pub mod rpc;

/// Payment methods
pub mod transfer;

/// Swap methods
pub mod swap;

/// Token methods
pub mod token;

/// CLI utility functions
pub mod cli_util;

/// Wallet functionality related to Money
pub mod money;

/// Wallet functionality related to Dao
pub mod dao;

/// Wallet functionality related to Deployooor
pub mod deploy;

/// Wallet functionality related to transactions history
pub mod txs_history;

/// Wallet database operations handler
pub mod walletdb;
use walletdb::{WalletDb, WalletPtr};

// Wallet SQL table constant names. These have to represent the `wallet.sql`
// SQL schema.
const WALLET_INFO_TABLE: &str = "wallet_info";
const WALLET_INFO_COL_LAST_SCANNED_BLOCK_HEIGHT: &str = "last_scanned_block_height";
const WALLET_INFO_COL_LAST_SCANNED_BLOCK_HASH: &str = "last_scanned_block_hash";

/// CLI-util structure
pub struct Drk {
    /// Wallet database operations handler
    pub wallet: WalletPtr,
    /// JSON-RPC client to execute requests to darkfid daemon
    pub rpc_client: Option<RpcClient>,
    /// Flag indicating if fun stuff are enabled
    pub fun: bool,
}

impl Drk {
    pub async fn new(
        wallet_path: String,
        wallet_pass: String,
        endpoint: Option<Url>,
        ex: Arc<smol::Executor<'static>>,
        fun: bool,
    ) -> Result<Self> {
        // Initialize wallet
        let wallet_path = expand_path(&wallet_path)?;
        if !wallet_path.exists() {
            if let Some(parent) = wallet_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        let Ok(wallet) = WalletDb::new(Some(wallet_path), Some(&wallet_pass)) else {
            return Err(Error::DatabaseError(format!("{}", WalletDbError::InitializationFailed)));
        };

        // Initialize rpc client
        let rpc_client = if let Some(endpoint) = endpoint {
            Some(RpcClient::new(endpoint, ex).await?)
        } else {
            None
        };

        Ok(Self { wallet, rpc_client, fun })
    }

    /// Initialize wallet with tables for `Drk`.
    pub async fn initialize_wallet(&self) -> WalletDbResult<()> {
        // Initialize wallet schema
        self.wallet.exec_batch_sql(include_str!("../wallet.sql"))?;

        // We maintain the last scanned block as part of the wallet
        // info table.
        if self.last_scanned_block().await.is_err() {
            let query = format!(
                "INSERT INTO {} ({}, {}) VALUES (?1, ?2);",
                WALLET_INFO_TABLE,
                WALLET_INFO_COL_LAST_SCANNED_BLOCK_HEIGHT,
                WALLET_INFO_COL_LAST_SCANNED_BLOCK_HASH
            );
            self.wallet.exec_sql(&query, rusqlite::params![0, "-"])?;
        }

        Ok(())
    }

    /// Update the last scanned block height and hash in the wallet.
    pub fn update_last_scanned_block(&self, height: u32, hash: &str) -> WalletDbResult<()> {
        let query = format!(
            "UPDATE {} SET {} = ?1, {} = ?2;",
            WALLET_INFO_TABLE,
            WALLET_INFO_COL_LAST_SCANNED_BLOCK_HEIGHT,
            WALLET_INFO_COL_LAST_SCANNED_BLOCK_HASH
        );
        self.wallet.exec_sql(&query, rusqlite::params![height, hash])
    }

    /// Get the last scanned block height and hash from the wallet.
    pub async fn last_scanned_block(&self) -> WalletDbResult<(u32, String)> {
        let ret = self.wallet.query_single(WALLET_INFO_TABLE, &[], &[])?;

        let Value::Integer(height) = ret[0] else {
            return Err(WalletDbError::ParseColumnValueError);
        };
        let Ok(height) = u32::try_from(height) else {
            return Err(WalletDbError::ParseColumnValueError);
        };

        let Value::Text(ref hash) = ret[1] else {
            return Err(WalletDbError::ParseColumnValueError);
        };

        Ok((height, hash.clone()))
    }
}
