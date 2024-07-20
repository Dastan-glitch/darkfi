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
use darkfi::net;
use darkfi_serial::{AsyncDecodable, VarInt};
use smol::io::AsyncReadExt;
use std::sync::Arc;
use url::Url;

const ENDPOINT: &str = "tcp+tls://lilith1.dark.fi:5262";

async fn ping() {
    println!("Pinging {ENDPOINT}");
    let endpoint = Url::parse(ENDPOINT).unwrap();
    let dialer = net::transport::Dialer::new(endpoint, None).await.unwrap();
    let timeout = std::time::Duration::from_secs(5);

    println!("Connecting...");
    let Ok(mut stream) = dialer.dial(Some(timeout)).await else {
        println!("Connection failed");
        return
    };
    println!("Connected!");

    let mut magic = [0u8; 4];
    stream.read_exact(&mut magic).await.unwrap();
    println!("read magic bytes {:?}", magic);

    let command = String::decode_async(&mut stream).await.unwrap();
    println!("read command {command}");

    let payload_len = VarInt::decode_async(&mut stream).await.unwrap().0;
    println!("payload len = {payload_len}");

    let version = net::message::VersionMessage::decode_async(&mut stream).await.unwrap();
    println!("version: {version:?}");
}

fn main() {
    let (signal, shutdown) = smol::channel::unbounded::<()>();

    let ex = Arc::new(smol::Executor::new());
    let _task = ex.spawn(async {
        ping().await;
        let _ = signal.send(()).await;
    });

    let _ = smol::future::block_on(ex.run(shutdown.recv()));
}