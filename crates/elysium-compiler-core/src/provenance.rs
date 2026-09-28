use crate::source::{is_digest, Environment};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Provenance {
    revision: u32,
    environment: String,
    pub(crate) runtime: String,
    pub(crate) session: String,
    selection: String,
}

impl Provenance {
    pub(crate) fn verify(value: &Value, environment: &Value, request: &Value) -> Result<Self> {
        let proof: Self = serde_json::from_value(value.clone())
            .context("missing or invalid capture provenance")?;
        // This is the Source envelope, not an ad-hoc partial environment. Unknown
        // fields must not silently disappear when comparing runtime identities.
        let _: Environment = serde_json::from_value(environment.clone())?;
        ensure!(
            proof.revision == 1 && is_digest(&proof.session),
            "invalid capture session"
        );
        ensure!(
            proof.environment == hash(&serde_json::to_vec(environment)?),
            "capture environment mismatch"
        );
        let mut runtime = environment.clone();
        runtime
            .as_object_mut()
            .context("missing environment")?
            .remove("probes");
        let settings = runtime["settings"]
            .as_object_mut()
            .context("missing runtime settings")?;
        settings.remove("profile");
        settings.remove("handlers");
        settings.remove("scope");
        ensure!(
            proof.runtime == hash(&serde_json::to_vec(&runtime)?),
            "capture runtime mismatch"
        );
        let mut selection = request.clone();
        let fields = selection
            .as_object_mut()
            .context("missing capture request")?;
        fields.remove("key");
        fields.remove("name");
        ensure!(
            proof.selection == hash(&serde_json::to_vec(&selection)?),
            "capture selection mismatch"
        );
        ensure!(
            environment["settings"]["profile"] == request["profile"]
                && environment["probes"] == request["probes"],
            "capture request differs from its environment"
        );
        let handlers = request["handlers"]
            .as_array()
            .context("missing capture handlers")?
            .iter()
            .map(|value| value.as_str().context("invalid capture handler"))
            .collect::<Result<Vec<_>>>()?
            .join(",");
        ensure!(
            environment["settings"]["handlers"] == handlers,
            "capture handlers differ from environment"
        );
        ensure!(
            environment["settings"]["scope"] == request["scope"],
            "capture scope differs from environment"
        );
        if !request["scope"].is_null() {
            ensure!(
                request["scope"] == "recipes"
                    && request["check"].is_null()
                    && request["world"]
                        .as_str()
                        .is_some_and(|value| !value.is_empty())
                    && !handlers.is_empty()
                    && matches!(request["profile"].as_str(), Some("full" | "data")),
                "invalid recipe capture scope"
            );
        }
        Ok(proof)
    }
}
