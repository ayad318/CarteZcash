use json::JsonValue;
use ethereum_types::U256;

use zebra_chain::amount::Amount;
use zebra_chain::serialization::ZcashDeserialize;
use zebra_chain::transaction::Transaction;
use zebra_chain::transparent::Address;

#[derive(Debug)]
pub enum Request {
    AdvanceState(AdvanceStateRequest),
    InspectState,
}

impl TryFrom<JsonValue> for Request {
    type Error = anyhow::Error;

    fn try_from(req: JsonValue) -> Result<Self, Self::Error> {
        match req["request_type"].as_str() {
            Some("advance_state") => {
                let advance_state = AdvanceStateRequest::try_from(req)?;
                Ok(Request::AdvanceState(advance_state))
            }
            Some("inspect_state") => Ok(Request::InspectState),
            _ => anyhow::bail!("Invalid request type"),
        }
    }
}

/// Requests that can be received from the L1
/// will be either EtherTransfer {"request_type":"advance_state","data":{"metadata":{"msg_sender":"0xffdbe43d4c855bf7e0f105c400a50857f53ab044","epoch_index":0,"input_index":0,"block_number":11,"timestamp":1710913093},"payload":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000000000000000000000000001314fb37062980000"}}
///      or generic data message {"request_type":"advance_state","data":{"metadata":{"msg_sender":"0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266","epoch_index":0,"input_index":1,"block_number":122,"timestamp":1710913648},"payload":"0xffff"}}
#[derive(Debug)]
pub enum AdvanceStateRequest {
    Deposit { amount: Amount, to: Address },
    Transact { txn: Transaction },
}

const ETH_DEPOSIT_ADDR: &str = "0xffdbe43d4c855bf7e0f105c400a50857f53ab044";
const INBOX_CONTRACT_ADDR: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

impl TryFrom<JsonValue> for AdvanceStateRequest {
    type Error = anyhow::Error;

    fn try_from(req: JsonValue) -> Result<Self, Self::Error> {
        if req["request_type"] != "advance_state" {
            anyhow::bail!("Invalid request type");
        }
        match req["data"]["metadata"]["msg_sender"].as_str() {
            Some(ETH_DEPOSIT_ADDR) => {
                /*  encoding as determined by the Cartesi Eth deposit contract
                abi.encodePacked(
                    sender, //              20B
                    value, //               32B
                    execLayerData //        arbitrary size
                );
                */
                let hex = req["data"]["payload"].as_str().unwrap();
                let bytes = hex::decode(hex.trim_start_matches("0x"))?;

                let _sender = &bytes[0..20];
                let value = U256::from_big_endian(&bytes[20..52]).checked_div(U256::try_from(10_000_000_000_u64).unwrap()).unwrap(); // 1 ZEC is 100_000_000 units while 1 ETH is 10^18. So we divide by 10^10 so that 1 ETH is 1 ZEC
                println!("Value: {:?}", value);
                let dest_t_address = Address::from_pub_key_hash(zebra_chain::parameters::Network::Mainnet, bytes[52..].try_into().unwrap());
                
                Ok(AdvanceStateRequest::Deposit {
                    amount: Amount::try_from(value.as_u64())?, // FIX: This is going to panic if too much eth is sent
                    to: dest_t_address,
                })
            }
            Some(INBOX_CONTRACT_ADDR) => {
                let hex = req["data"]["payload"].as_str().unwrap();
                let bytes = hex::decode(hex.trim_start_matches("0x"))?;
                let txn = zebra_chain::transaction::Transaction::zcash_deserialize(
                    &mut bytes.as_slice(),
                )?;
                Ok(AdvanceStateRequest::Transact { txn })
            }
            _ => anyhow::bail!("unrecognised sender"),
        }
    }
}