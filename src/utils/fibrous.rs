use starknet::accounts::Account;
use starknet::core::codec::{Decode, Encode};
use starknet::core::types::{BlockId, BlockTag, Call, Felt, U256};
use starknet::macros::selector;

use super::starknet::{contract_address_felt, signer_account};

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct RouteParams {
    pub token_in: Felt,
    pub token_out: Felt,
    pub amount_in: u128,
    pub min_received: U256,
    pub destination: Felt,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SwapParams {
    pub token_in: Felt,
    pub token_out: Felt,
    pub rate: u32,
    pub protocol_id: u32,
    pub pool_address: Felt,
    pub extra_data: Vec<Felt>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallData {
    route_params: RouteParams,
    swap_params: SwapParams,
    contract_address: Felt,
}

type FibrousResponse = Result<
    starknet::core::types::InvokeTransactionResult,
    starknet::accounts::AccountError<
        starknet::accounts::single_owner::SignError<starknet::signers::local_wallet::SignError>,
    >,
>;

fn encode_swap_params(swap_params: &Vec<SwapParams>) -> Vec<Felt> {
    let swap_params_owned: Vec<SwapParams> = swap_params.to_owned();
    swap_params_owned
        .into_iter()
        .flat_map(|params| {
            let mut swap_data = vec![
                params.token_in,
                params.token_out,
                Felt::from(params.rate),
                Felt::from(params.protocol_id),
                params.pool_address,
                Felt::from(params.extra_data.len()),
            ];
            swap_data.extend(params.extra_data);
            swap_data
        })
        .collect()
}

pub async fn fibrous_swap(
    route_params: RouteParams,
    swap_params: Vec<SwapParams>,
    contract_address: Felt,
) -> FibrousResponse {
    let mut account = signer_account();
    let contract_address_main = contract_address_felt();

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let encoded_swap_params = encode_swap_params(&swap_params);

    let mut serialized: Vec<Felt> = vec![];
    route_params.encode(&mut serialized).unwrap();
    serialized.push(Felt::from(encoded_swap_params.len()));
    serialized = serialized.into_iter().chain(encoded_swap_params).collect();
    serialized.push(contract_address);

    let approve_call = Call {
        to: route_params.token_in,
        selector: selector!("approve"),
        calldata: vec![contract_address_main, route_params.amount_in.into()],
    };

    let swap_call = Call {
        to: contract_address_main,
        selector: selector!("fibrous_swap"),
        calldata: serialized,
    };

    account
        .execute_v3(vec![approve_call, swap_call])
        .send()
        .await
}
