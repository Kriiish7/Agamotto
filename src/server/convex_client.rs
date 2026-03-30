use std::collections::BTreeMap;
use std::env;

use convex::{ConvexClient, Value};

pub struct AgamottoConvex {
    client: ConvexClient,
}

impl AgamottoConvex {
    pub async fn new() -> Result<Self, String> {
        dotenvy::from_filename(".env.local").ok();
        dotenvy::dotenv().ok();

        let url = env::var("CONVEX_URL").map_err(|_| "CONVEX_URL not set".to_string())?;
        let client = ConvexClient::new(&url)
            .await
            .map_err(|e| format!("Convex connection failed: {e}"))?;

        Ok(Self { client })
    }

    pub async fn query(
        &mut self,
        function: &str,
        args: BTreeMap<String, Value>,
    ) -> Result<Value, String> {
        let result = self
            .client
            .query(function, args)
            .await
            .map_err(|e| format!("Query {function} failed: {e}"))?;
        Ok(result)
    }

    pub async fn mutation(
        &mut self,
        function: &str,
        args: BTreeMap<String, Value>,
    ) -> Result<Value, String> {
        let result = self
            .client
            .mutation(function, args)
            .await
            .map_err(|e| format!("Mutation {function} failed: {e}"))?;
        Ok(result)
    }
}
