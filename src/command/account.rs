// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

// Copyright 2019 Conflux Foundation. All rights reserved.
// Conflux is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

extern crate ethcore_accounts;

use super::helpers::{keys_path, password_from_file, password_prompt};
use clap;
use ethcore_accounts::{AccountProvider, AccountProviderSettings};
use ethstore::{
    accounts_dir::RootDiskDirectory, import_account, import_accounts, EthStore,
};
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum AccountCmd {
    New(NewAccount),
    List(ListAccounts),
    Import(ImportAccounts),
}

#[derive(Debug, PartialEq)]
pub struct ListAccounts {
    pub path: String,
}

impl ListAccounts {
    pub fn new(_matches: &clap::ArgMatches) -> Self {
        Self { path: keys_path() }
    }
}

#[derive(Debug, PartialEq)]
pub struct NewAccount {
    pub iterations: u32,
    pub path: String,
    pub password_file: Option<String>,
}

impl NewAccount {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let iterations: u32 = matches
            .value_of("key-iterations")
            .unwrap_or("0")
            .parse()
            .unwrap();
        let password_file = matches.value_of("password").map(|x| x.to_string());
        Self {
            iterations,
            path: keys_path(),
            password_file,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ImportAccounts {
    pub from: Vec<String>,
    pub to: String,
}

impl ImportAccounts {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let from: Vec<_> = matches
            .values_of("import-path")
            .expect("CLI argument is required; qed")
            .map(|s| s.to_string())
            .collect();
        Self {
            from,
            to: keys_path(),
        }
    }
}

pub fn execute(cmd: AccountCmd) -> Result<String, String> {
    match cmd {
        AccountCmd::New(new_cmd) => new(new_cmd),
        AccountCmd::List(list_cmd) => list(list_cmd),
        AccountCmd::Import(import_cmd) => import(import_cmd),
    }
}

fn keys_dir(path: String) -> Result<RootDiskDirectory, String> {
    let mut path = PathBuf::from(&path);
    // TODO: make a global constant
    path.push("conflux".to_string());
    RootDiskDirectory::create(path)
        .map_err(|e| format!("Could not open keys directory: {}", e))
}

fn secret_store(
    dir: Box<RootDiskDirectory>, iterations: Option<u32>,
) -> Result<EthStore, String> {
    match iterations {
        Some(i) => EthStore::open_with_iterations(dir, i),
        _ => EthStore::open(dir),
    }
    .map_err(|e| format!("Could not open keys store: {}", e))
}

fn new(new_cmd: NewAccount) -> Result<String, String> {
    let password = match new_cmd.password_file {
        Some(file) => password_from_file(file)?,
        None => password_prompt()?,
    };

    let dir = Box::new(keys_dir(new_cmd.path)?);
    let secret_store = Box::new(secret_store(dir, Some(new_cmd.iterations))?);
    let acc_provider =
        AccountProvider::new(secret_store, AccountProviderSettings::default());
    let new_account = acc_provider
        .new_account(&password)
        .map_err(|e| format!("Could not create new account: {}", e))?;
    Ok(format!("0x{:x}", new_account))
}

fn list(list_cmd: ListAccounts) -> Result<String, String> {
    let dir = Box::new(keys_dir(list_cmd.path)?);
    let secret_store = Box::new(secret_store(dir, None)?);
    let acc_provider =
        AccountProvider::new(secret_store, AccountProviderSettings::default());
    let accounts = acc_provider.accounts().map_err(|e| format!("{}", e))?;
    let result = accounts
        .into_iter()
        .map(|a| format!("0x{:x}", a))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(result)
}

fn import(import_cmd: ImportAccounts) -> Result<String, String> {
    let to = keys_dir(import_cmd.to)?;
    let mut imported = 0;

    for path in &import_cmd.from {
        let path = PathBuf::from(path);
        if path.is_dir() {
            let from = RootDiskDirectory::at(&path);
            imported += import_accounts(&from, &to)
                .map_err(|e| {
                    format!("Importing accounts from {:?} failed: {}", path, e)
                })?
                .len();
        } else if path.is_file() {
            import_account(&path, &to).map_err(|e| {
                format!("Importing account from {:?} failed: {}", path, e)
            })?;
            imported += 1;
        }
    }

    Ok(format!("{} account(s) imported", imported))
}