use colored::*;

use crate::near_jsonrpc_client::NearJsonRpcClient;
use crate::primitives::{Account, AccountBalancesAtBlock, Block};
use crate::utils;

pub(crate) async fn collect_account_data(
    client: &NearJsonRpcClient,
    account: &mut Account,
    block: Block,
) -> AccountBalancesAtBlock {
    let account_in_pool = match client
        .get_account_in_pool(
            account.clone().account_id,
            account
                .get_pool_account_id(&client)
                .await
                .expect("Unable to get the pool"),
            block.header.height,
        )
        .await
    {
        Ok(account) => account,
        Err(err) => {
            panic!("Error: {}", err);
        }
    };
    let locked_amount: u128 = if let Some(amount) = &account.locked_amount {
        if let Ok(amount) = amount.parse() {
            amount
        } else {
            0
        }
    } else {
        match client
            .get_locked_amount(account.clone().account_id, block.header.height)
            .await
        {
            Ok(amount) => amount,
            Err(_err) => 0,
        }
    };
    let native_balance = match client
        .get_native_balance(account.clone().account_id, block.header.height)
        .await
    {
        Ok(amount) => amount,
        Err(err) => {
            panic!("Reqwest Error: {}", err);
        }
    };
    let liquid_balance = match client
        .get_liquid_owners_balance(account.clone().account_id, block.header.height)
        .await
    {
        Ok(amount) => amount,
        Err(_err) => native_balance,
    };
    let reward = account_in_pool
        .get_staked_balance()
        .saturating_add(account_in_pool.get_unstaked_balance())
        .saturating_add(if locked_amount > 0 { native_balance } else { 0 })
        .saturating_sub(locked_amount);

    AccountBalancesAtBlock {
        account_in_pool,
        native_balance,
        liquid_balance,
        reward,
    }
}

pub(crate) fn reward_diff(current_reward: u128, prev_reward: u128) -> String {
    if current_reward > prev_reward {
        return format!("+{:.2}", utils::human(current_reward - prev_reward))
            .blue()
            .to_string();
    } else {
        return format!("-{:.2}", utils::human(prev_reward - current_reward))
            .red()
            .to_string();
    }
}

pub(crate) fn current_reward(current_reward: u128) -> String {
    return format!("{:.2}", utils::human(current_reward))
        .green()
        .to_string();
}
