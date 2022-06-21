use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::prelude::*;
use std::str;
use std::str::FromStr;
use std::{thread, time::Duration};
use web3::helpers as w3h;
use web3::types::{BlockId, BlockNumber, TransactionId, H160};
#[derive(Serialize, Deserialize, Debug)]
struct TX {
    block: String,
    tx_hash: String,
    from: String,
    to: String,
    value: String,
}

/*
impl std::fmt::Display for TX {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
*/

struct HexSlice<'a>(&'a [u8]);

impl<'a> HexSlice<'a> {
    fn new<T>(data: &'a T) -> HexSlice<'a>
    where
        T: ?Sized + AsRef<[u8]> + 'a,
    {
        HexSlice(data.as_ref())
    }
}

// You can choose to implement multiple traits, like Lower and UpperHex
impl fmt::Display for HexSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            // Decide if you want to pad the value or have spaces inbetween, etc.
            write!(f, "{:X} ", byte)?;
        }
        Ok(())
    }
}

trait HexDisplayExt {
    fn hex_display(&self) -> HexSlice<'_>;
}

impl<T> HexDisplayExt for T
where
    T: ?Sized + AsRef<[u8]>,
{
    fn hex_display(&self) -> HexSlice<'_> {
        HexSlice::new(self)
    }
}

#[tokio::main]
async fn main() -> web3::Result<()> {
    // wss://rpc.mainnet.pulsechain.com/ws/v1/

    let rpc_url = "https://api.avax.network/ext/bc/C/rpc";
    let web3httpclient = web3::transports::Http::new(rpc_url).unwrap();
    let web3s = web3::Web3::new(web3httpclient);
    let block_counter: u64 = 9027422; // v1 deploy block
    let block_counter_end: u64 = 9027500; // last v1 tx
    let block_to_check = block_counter_end - block_counter;

    let contract_address = "0x40064CE057Fb99a5c8e34F61365cC5996E59aB57"; // PXT V1
    let pb = ProgressBar::new(block_to_check);

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("my-file.txt")
        .unwrap();

    //let mut txs: Vec<TX> = vec![];

    for current_block_index in block_counter..block_counter_end {
        //println!("current_block_index {}", current_block_index);

        let latest_block = web3s
            .eth()
            .block(BlockId::Number(BlockNumber::Number(
                web3::types::U64::from(current_block_index),
            )))
            .await
            .unwrap_or_default()
            .unwrap_or_default();

        for transaction_hash in latest_block.transactions {
            let tx = match web3s
                .eth()
                .transaction(TransactionId::Hash(transaction_hash))
                .await
            {
                Ok(Some(tx)) => tx,
                _ => {
                    println!("An error occurred");
                    continue;
                }
            };

            let from_addr = tx.from.unwrap_or(H160::zero());
            let to_addr = tx.to.unwrap_or(H160::zero());
            let contract_address_h160: H160 = H160::from_str(contract_address).unwrap();

            //println!("From: {:?}", from_addr);
            //println!("To: {:?}", to_addr);
            //println!("");

            if to_addr == contract_address_h160 {
                let a = tx.input.0.hex_display();

                println!("{}", a.to_string());

                //println!("tx: {:?}", str::from_utf8(&a).unwrap());

                println!("FOUND -----------");
                println!("Tx: {:?}", w3h::to_string(&transaction_hash));
                println!("From: {:?}", from_addr);

                let ctx = TX {
                    block: current_block_index.to_string(),
                    tx_hash: w3h::to_string(&transaction_hash),
                    from: w3h::to_string(&from_addr),
                    to: w3h::to_string(&to_addr),
                    value: tx.value.to_string(),
                };

                let serialized_ctx = serde_json::to_string(&ctx).unwrap();
                //println!("{:?}", ctx);
                //println!("{}", serialized_ctx);
                //txs.push(ctx);

                /*
                // Save the JSON structure into the other file.
                std::fs::write(
                    "result.txt",
                    serde_json::to_string_pretty(&ctx).unwrap(),
                )
                .unwrap();
                */

                if let Err(e) = writeln!(file, "{}", serialized_ctx) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
        }
        //pb.inc(1);
        println!("{}", current_block_index);
        thread::sleep(Duration::from_millis(5));
    }
    pb.finish_with_message("done");
    Ok(())
}
