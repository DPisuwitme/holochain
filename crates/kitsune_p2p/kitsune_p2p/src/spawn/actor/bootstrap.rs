use once_cell::sync::Lazy;
use crate::types::KitsuneSpace;
use crate::types::agent_store::AgentInfoSigned;
use std::convert::TryInto;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

const BOOTSTRAP_URL_ENV: &str = "P2P_BOOTSTRAP_URL";
const BOOTSTRAP_URL_DEFAULT: &str = "https://bootstrap.holo.host";
#[allow(dead_code)]
const BOOTSTRAP_URL_DEV: &str = "https://bootstrap-dev.holohost.workers.dev";

#[allow(dead_code)]
const RANDOM_LIMIT_DEFAULT: u32 = 256;

#[allow(clippy::declare_interior_mutable_const)]
const BOOTSTRAP_URL: Lazy<Option<String>> = Lazy::new(|| match std::env::var(BOOTSTRAP_URL_ENV) {
    Ok(v) => {
        // If the environment variable is set and empty then don't bootstrap at all.
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
    // If the environment variable is not set then fallback to the default.
    Err(_) => Some(BOOTSTRAP_URL_DEFAULT.to_string()),
});

const OP_HEADER: &str = "X-Op";
const OP_PUT: &str = "put";
const OP_NOW: &str = "now";
const OP_RANDOM: &str = "random";

fn select_url(url_override: Option<String>) -> Option<String> {
    match url_override {
        Some(url) => Some(url),
        #[allow(clippy::borrow_interior_mutable_const)]
        None => match Lazy::force(&BOOTSTRAP_URL) {
            Some(url) => Some(url.to_string()),
            None => None,
        },
    }
}

async fn do_api<I: serde::Serialize, O: serde::de::DeserializeOwned>(
    url_override: Option<String>,
    op: &str,
    input: I,
) -> crate::types::actor::KitsuneP2pResult<Option<O>> {
    let mut body_data = Vec::new();
    kitsune_p2p_types::codec::rmp_encode(&mut body_data, &input)?;
    match select_url(url_override) {
        Some(url) => {
            let res = CLIENT
                .post(&url)
                .body(body_data)
                .header(OP_HEADER, op)
                .header(reqwest::header::CONTENT_TYPE, "application/octet")
                .send()
                .await?;
            if res.status().is_success() {
                Ok(Some(
                    kitsune_p2p_types::codec::rmp_decode(
                        &mut res.bytes().await?.as_ref()
                    )?
                ))
            } else {
                return Err(crate::KitsuneP2pError::Bootstrap(std::sync::Arc::new(
                    res.text().await?,
                )));
            }
        }
        None => Ok(None),
    }
}

pub async fn put(
    url_override: Option<String>,
    agent_info_signed: crate::types::agent_store::AgentInfoSigned,
) -> crate::types::actor::KitsuneP2pResult<()> {
    match do_api(url_override, OP_PUT, agent_info_signed).await {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

#[allow(dead_code)]
pub async fn now(url_override: Option<String>) -> crate::types::actor::KitsuneP2pResult<u64> {
    match do_api(url_override, OP_NOW, ()).await {
        Ok(Some(v)) => Ok(v),
        Ok(None) => {
            let local_now = std::time::SystemTime::now();
            Ok(local_now.duration_since(std::time::UNIX_EPOCH)?.as_millis().try_into()?)
        },
        Err(e) => Err(e),
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RandomQuery {
    pub space: KitsuneSpace,
    pub limit: u32,
}

#[allow(dead_code)]
pub async fn random(
    url_override: Option<String>,
    query: RandomQuery,
) -> crate::types::actor::KitsuneP2pResult<Vec<AgentInfoSigned>> {
    match do_api(url_override, OP_RANDOM, query).await {
        Ok(Some(v)) => Ok(v),
        Ok(None) => {
            Ok(Vec::new())
        },
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {

    use crate::fixt::*;
    use crate::spawn::actor::space::AGENT_INFO_EXPIRES_AFTER_MS;
    use crate::types::agent_store::*;
    use crate::types::KitsuneAgent;
    use crate::types::KitsuneBinType;
    use crate::types::KitsuneSignature;
    use fixt::prelude::*;
    use lair_keystore_api::internal::sign_ed25519::sign_ed25519_keypair_new_from_entropy;
    use std::convert::TryInto;

    #[tokio::test(threaded_scheduler)]
    async fn test_bootstrap() {
        let keypair = sign_ed25519_keypair_new_from_entropy().await.unwrap();
        let space = fixt!(KitsuneSpace);
        let agent = KitsuneAgent::new((*keypair.pub_key.0).clone());
        let urls = fixt!(Urls);
        let now = std::time::SystemTime::now();
        let millis = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let agent_info = AgentInfo::new(
            space,
            agent.clone(),
            urls,
            (millis - 100).try_into().unwrap(),
            AGENT_INFO_EXPIRES_AFTER_MS,
        );
        let mut data = Vec::new();
        kitsune_p2p_types::codec::rmp_encode(&mut data, &agent_info).unwrap();
        let signature = keypair
            .sign(std::sync::Arc::new(data.clone()))
            .await
            .unwrap();
        let agent_info_signed =
            AgentInfoSigned::try_new(agent, KitsuneSignature((*signature.0).clone()), data)
                .unwrap();

        // Simply hitting the endpoint should be OK.
        super::put(
            Some(super::BOOTSTRAP_URL_DEV.to_string()),
            agent_info_signed,
        )
        .await
        .unwrap();

        // We should get back an error if we don't have a good signature.
        assert!(super::put(
            Some(super::BOOTSTRAP_URL_DEV.to_string()),
            fixt!(AgentInfoSigned)
        )
        .await
        .is_err());
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_now() {
        let local_now = std::time::SystemTime::now();
        let local_millis: u64 = local_now.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis().try_into().unwrap();

        // We should be able to get a milliseconds timestamp back.
        let remote_now = super::now(Some(super::BOOTSTRAP_URL_DEV.to_string()))
            .await
            .unwrap();
        let threshold = 1000;

        assert!(
            (remote_now - local_millis) < threshold
        );
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_random() {
        let space = fixt!(KitsuneSpace);
        let now = super::now(Some(super::BOOTSTRAP_URL_DEV.to_string())).await.unwrap();

        let alice = sign_ed25519_keypair_new_from_entropy().await.unwrap();
        let bob = sign_ed25519_keypair_new_from_entropy().await.unwrap();

        let mut expected: Vec<AgentInfoSigned> = Vec::new();
        for agent in vec![alice.clone(), bob.clone()] {
            let kitsune_agent = KitsuneAgent::new((*agent.pub_key.0).clone());
            let agent_info = AgentInfo::new(
                space.clone(),
                kitsune_agent.clone(),
                fixt!(Urls),
                now,
                AGENT_INFO_EXPIRES_AFTER_MS,
            );
            let mut data = Vec::new();
            kitsune_p2p_types::codec::rmp_encode(&mut data, &agent_info).unwrap();
            let signature = agent
                .sign(std::sync::Arc::new(data.clone()))
                .await
                .unwrap();
            let agent_info_signed =
                AgentInfoSigned::try_new(kitsune_agent, KitsuneSignature((*signature.0).clone()), data)
                    .unwrap();

            super::put(
                Some(super::BOOTSTRAP_URL_DEV.to_string()),
                agent_info_signed.clone(),
            )
            .await
            .unwrap();

            expected.push(agent_info_signed);
        }

        let mut random = super::random(
            Some(super::BOOTSTRAP_URL_DEV.to_string()),
            super::RandomQuery {
                space: space.clone(),
                limit: super::RANDOM_LIMIT_DEFAULT,
            },
        )
        .await
        .unwrap();

        expected.sort();
        random.sort();

        assert!(random.len() == 2);
        assert!(random == expected);

        let random_single = super::random(
            Some(super::BOOTSTRAP_URL_DEV.to_string()),
            super::RandomQuery {
                space: space.clone(),
                limit: 1,
            },
        )
        .await
        .unwrap();

        assert!(random.len() == 1);
        assert!(expected[0] == random_single[0] || expected[1] == random_single[0]);
    }
}