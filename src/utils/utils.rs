use std::collections::{HashMap, VecDeque};

use bitcoin::Amount;
use bitcoincore_rpc::json::ListUnspentResultEntry;

use crate::modules::errors::UtilsError;

#[derive(Clone)]
pub enum UTXOStrategy {
    BranchAndBound,
    Fifo,
    LargestFirst,
    SmallestFirst
}

/// Extract xpubs from descriptors

pub fn extract_int_ext_xpubs(
    mut xpubs: HashMap<String,String>,
    descriptors_array: Vec<serde_json::Value>,
    i: usize
) -> Result<HashMap<String, String>, UtilsError> {
    // Find the correct descriptors for external and internal xpubs
    let external_xpub = descriptors_array
        .iter()
        .find(|desc| {
            desc["desc"].as_str().unwrap_or_default().starts_with("wpkh") && desc["desc"].as_str().unwrap_or_default().contains("/0/*")
        })
        .ok_or(UtilsError::ExternalXpubNotFound)?["desc"]
        .as_str().ok_or(UtilsError::ExternalXpubNotFound)?
        .to_string();

    let internal_xpub = descriptors_array
        .iter()
        .find(|desc| {
            desc["desc"].as_str().unwrap_or_default().starts_with("wpkh") && desc["desc"].as_str().unwrap_or_default().contains("/1/*")
        })
        .ok_or(UtilsError::InternalXpubNotFound)?["desc"]
        .as_str().ok_or(UtilsError::InternalXpubNotFound)?
        .to_string();

    // formatting notes: https://bitcoincoredocs.com/descriptors.html
    // split at "]" and take the last part
    let external_xpub_no_path = external_xpub.split("]").last().unwrap().to_string();
    let internal_xpub_no_path = internal_xpub.split("]").last().unwrap().to_string();

    // split at ")" and take the first part
    let external_xpub_no_path = external_xpub_no_path.split(")").next().unwrap().to_string();
    let internal_xpub_no_path = internal_xpub_no_path.split(")").next().unwrap().to_string();

    xpubs.insert(format!("internal_xpub_{}", i + 1), internal_xpub_no_path);
    xpubs.insert(format!("external_xpub_{}", i + 1), external_xpub_no_path);

    Ok(xpubs)
}

/// UTXO Selection Strategies

pub fn strat_handler(
    utxos: &[ListUnspentResultEntry],
    target_amount: Amount,
    fee_amount: Amount,
    utxo_strategy: UTXOStrategy
) -> Result<Vec<ListUnspentResultEntry>, UtilsError> {
    match utxo_strategy {
        UTXOStrategy::BranchAndBound => select_utxos_branch_and_bound(utxos, target_amount, fee_amount)
            .ok_or(UtilsError::InsufficientUTXOs),
        UTXOStrategy::Fifo => select_utxos_fifo(utxos, target_amount, fee_amount),
        UTXOStrategy::LargestFirst => select_utxos_largest_first(utxos, target_amount, fee_amount),
        UTXOStrategy::SmallestFirst => select_utxos_smallest_first(utxos, target_amount, fee_amount),
    }
}

fn select_utxos_branch_and_bound(
    utxos: &[ListUnspentResultEntry],
    target_amount: Amount,
    fee_amount: Amount,
) -> Option<Vec<ListUnspentResultEntry>> {
    let mut current_best_solution = None;
    let mut current_best_change = Amount::from_sat(u64::MAX);

    // the queue is a "vector double ended queue" that allows us to add and remove
    // elements from both ends of the vector
    let mut queue: VecDeque<(Vec<ListUnspentResultEntry>, Amount)> = VecDeque::new();

    // add the first element to the queue
    queue.push_back((Vec::new(), Amount::from_sat(0)));

    // This while loop uses a breadth-first search approach to explore all possible combinations of UTXOs.
    // It continually checks if the current combination is sufficient to cover the target amount plus fees
    // and updates the best solution found so far. If a combination is not sufficient, it expands the search
    // by adding more UTXOs to the combination and continues the process until all possibilities have been
    // explored. This ensures that the algorithm finds an optimal set of UTXOs with minimal leftover change.
    while let Some((current_selection, current_total)) = queue.pop_front() {
        if current_total >= target_amount + fee_amount {
            let change = current_total - target_amount - fee_amount;
            if change < current_best_change {
                current_best_change = change;
                current_best_solution = Some(current_selection.clone());
            }
        } else {
            for (_index, utxo) in utxos.iter().enumerate() {
                if !current_selection.contains(utxo) {
                    let mut new_selection = current_selection.clone();
                    new_selection.push(utxo.clone());
                    let new_total = current_total + utxo.amount;
                    queue.push_back((new_selection, new_total));
                }
            }
        }
    }

    current_best_solution
}

fn select_utxos_fifo(
    utxos: &[ListUnspentResultEntry],
    target_amount: Amount,
    fee_amount: Amount,
) -> Result<Vec<ListUnspentResultEntry>, UtilsError> {
    let sorted_utxos = utxos.to_vec();
    return select_utxos(sorted_utxos, target_amount, fee_amount);
}

fn select_utxos_largest_first(
    utxos: &[ListUnspentResultEntry],
    target_amount: Amount,
    fee_amount: Amount,
) -> Result<Vec<ListUnspentResultEntry>, UtilsError> {
    // Sort UTXOs by amount in descending order
    let mut sorted_utxos = utxos.to_vec();
    sorted_utxos.sort_by(|a, b| b.amount.cmp(&a.amount));

    return select_utxos(sorted_utxos, target_amount, fee_amount);
}

fn select_utxos_smallest_first(
    utxos: &[ListUnspentResultEntry],
    target_amount: Amount,
    fee_amount: Amount,
) -> Result<Vec<ListUnspentResultEntry>, UtilsError> {
    // Sort UTXOs by amount in descending order
    let mut sorted_utxos = utxos.to_vec();
    sorted_utxos.sort_by(|a, b| a.amount.cmp(&b.amount));

    return select_utxos(sorted_utxos, target_amount, fee_amount);
}

fn select_utxos(
    sorted_utxos: Vec<ListUnspentResultEntry>,
    target_amount: Amount,
    fee_amount: Amount
) -> Result<Vec<ListUnspentResultEntry>, UtilsError> {
    let mut selected_utxos = Vec::new();
    let mut total_amount = Amount::from_sat(0);

    for utxo in sorted_utxos.iter() {
        selected_utxos.push(utxo.clone());
        total_amount += utxo.amount;

        if total_amount >= target_amount + fee_amount {
            return Ok(selected_utxos);
        }
    }

    Err(UtilsError::InsufficientUTXOs)
}