// Copyright (c) 2018-2020 MobileCoin Inc.

//! A tool for inspecting binary keyfiles
//! Reads .bin file on stdin, or a path to .bin file, emits description on stdout

use mc_transaction_core::account_keys::AccountKey;
use mc_transaction_std::identity::RootIdentity;

fn main() {
    let root_id: RootIdentity = {
        let args: Vec<String> = std::env::args().collect();
        match args.get(1) {
            None => mc_util_keyfile::read_keyfile_data(&mut std::io::stdin())
                .unwrap_or_else(|_| panic!("Failed when reading from stdin")),
            Some(arg) => mc_util_keyfile::read_keyfile(arg)
                .unwrap_or_else(|_| panic!("Failed when reading from {}", arg)),
        }
    };
    let acct_key = AccountKey::from(&root_id);
    println!("{:?}\n{:?}", root_id, acct_key,);
}
