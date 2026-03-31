pub mod greedy;
pub mod knapsack;
pub mod edf;
pub mod metrics;
pub mod functions;

pub mod forecast;
pub mod momentum;
pub mod debt;
pub mod identity;
pub mod recovery;
pub mod regret;

#[cfg(feature = "server")]
pub mod convex_client;
#[cfg(feature = "server")]
pub mod persistence;

#[cfg(test)]
mod tests;
