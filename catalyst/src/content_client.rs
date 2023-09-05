use reqwest::multipart::Form;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::entity_files::SceneFile;
use crate::entity_information::EntityInformation;
use crate::snapshot::{EntitySnapshot, Snapshot};
use crate::status::ContentServerStatus;
use crate::*;
use dcl_common::{Parcel, Result};

/// Implements all the request to interact with [Catalyst Content Servers](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server).
///
#[derive(Default)]
pub struct ContentClient {}

#[derive(Serialize)]
struct ParcelPointer<'a> {
    pointers: &'a Vec<Parcel>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub challenge_text: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeployResponse {
    pub creation_timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntityData {
    pub pointer: String,
    pub entity_id: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FailedDeployment {
    pub failed_deployments_repo: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub reason: String,
    pub error_description: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ContentFileStatus {
    #[serde(rename = "cid")]
    pub id: ContentId,
    pub available: bool,
}

impl ContentClient {
    /// Returns a list of entity ids related to the given ContentId hash.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getListEntityIdsByHashId)
    pub async fn active_entities(server: &Server, content_id: &ContentId) -> Result<Vec<EntityId>> {
        let result = server
            .get(format!("/content/contents/{}/active-entities", content_id))
            .await?;
        Ok(result)
    }

    ///Used by the Server to figure out their identity on the DAO by themselves, so they will generate a random challenge text, and then query each server for it. If the text matches, then they have found themselves.
    ///[See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server/operation/getContentFile)
    pub async fn challenge(server: &Server) -> Result<Challenge> {
        let result = server.get("/content/challenge").await?;
        Ok(result)
    }

    /// Returns true if the content exists and false if it doesnt.
    ///[See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server/operation/headContentFile)    pub async fn challenge(server: &Server) -> Result<Challenge> {
    pub async fn content_file_exists(server: &Server, content: &ContentId) -> Result<bool> {
        let result = server
            .raw_head(format!("/content/contents/{}", content.hash()))
            .await?;

        Ok(result.status() == StatusCode::OK)
    }

    /// Returns the entity ids whose deployments are associated with the specified content hash.
    ///[See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server/operation/getEntityIdsByHashId)
    pub async fn entity_ids_by_hash(server: &Server, hash: &HashId) -> Result<Vec<ContentId>> {
        let result: Vec<ContentId> = server
            .get(format!("/content/contents/{}/entities", hash))
            .await?;
        Ok(result)
    }

    /// Deploys an entity in the content server.
    ///[See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server/operation/postEntity)
    pub async fn deploy_entity(server: &Server, form: Form) -> Result<DeployResponse> {
        let result = server.post_form("/content/entities", form).await?;
        Ok(result)
    }

    /// Returns the list of active entities which have at least one pointer that matches the prefix given
    ///[See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server/operation/getEntitiesByPointerPrefix)
    pub async fn entities_by_urn<T>(server: &Server, urn: T) -> Result<Vec<EntityData>>
    where
        T: AsRef<str>,
    {
        let result = server
            .get(format!(
                "/content/entities/active/collections/{}",
                urn.as_ref()
            ))
            .await?;
        Ok(result)
    }

    /// Retrieves a list of the failed deployments
    ///[See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#tag/Content-Server/operation/getFailedDeployments)
    pub async fn failed_deployments(server: &Server) -> Result<Vec<FailedDeployment>> {
        let result: Vec<FailedDeployment> =
            server.get(format!("/content/failed-deployments")).await?;
        Ok(result)
    }

    /// Returns the availability state for all the given ContentIds.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getIfFileExists)
    pub async fn content_files_exists(
        server: &Server,
        content: &Vec<ContentId>,
    ) -> Result<Vec<ContentFileStatus>> {
        let mut cids = String::new();

        for cid in content {
            if cid != &content[0] {
                cids.push('&');
            }
            cids.push_str("cid=");
            cids.push_str(cid.hash());
        }

        let result = server
            .get(format!("/content/available-content/?{}", cids))
            .await?;
        Ok(result)
    }

    /// Download the file referenced by `content_id` in the path given by `filename`.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getContentFile)
    pub async fn download<V>(server: &Server, content_id: ContentId, filename: V) -> Result<()>
    where
        V: AsRef<Path>,
    {
        let response = server
            .raw_get(format!("/content/contents/{}", content_id))
            .await?;

        if let Some(parent) = filename.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        let mut dest = File::create(filename)?;
        let content = response.bytes().await?;
        dest.write_all(&content)?;

        Ok(())
    }

    /// Get information about the given `entity`.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getEntityInformation)
    pub async fn entity_information(server: &Server, entity: &Entity) -> Result<EntityInformation> {
        let result = server
            .get(format!("/content/audit/{}/{}", entity.kind, entity.id))
            .await?;
        Ok(result)
    }

    /// Returns the scene content files for all the scenes that own the given `parcels`.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getListOfEntities)
    pub async fn scene_files_for_parcels(
        server: &Server,
        parcels: &Vec<Parcel>,
    ) -> Result<Vec<SceneFile>> {
        let pointers = ParcelPointer { pointers: parcels };
        let result: Vec<SceneFile> = server.post("/content/entities/active", &pointers).await?;
        Ok(result)
    }

    /// Returns a list of entities (in the form of `EntitySnapshot`) for the given `entity_type` and `snapshot`.
    pub async fn snapshot_entities<T>(
        server: &Server,
        entity_type: EntityType,
        snapshot: &Snapshot,
    ) -> Result<Vec<EntitySnapshot<T>>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let hash: &ContentId = match entity_type {
            EntityType::Scene => &snapshot.entities.scene.hash,
            EntityType::Profile => &snapshot.entities.profile.hash,
            EntityType::Wearable => &snapshot.entities.wearable.hash,
            EntityType::Emote => &snapshot.entities.emote.hash,
        };

        let response = server
            .raw_get(format!("/content/contents/{}", hash))
            .await?;

        let text = response.text().await?;

        let mut result: Vec<EntitySnapshot<T>> = vec![];

        for line in text.lines() {
            if line.find('{') == Some(0) {
                let snapshot: EntitySnapshot<T> = serde_json::from_str(line)?;
                result.push(snapshot);
            }
        }

        Ok(result)
    }

    /// Returns a snapshot that includes the content ids for the entities available in the snapshot.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getActiveEntities)
    pub async fn snapshot(server: &Server) -> Result<Snapshot> {
        let result = server.get("/content/snapshot").await?;
        Ok(result)
    }

    /// Returns information about the status of the server.
    /// [See on Catalyst API Docs](https://decentraland.github.io/catalyst-api-specs/#operation/getStatus)
    pub async fn status(server: &Server) -> Result<ContentServerStatus> {
        let result = server.get("/content/status").await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use dcl_common::Parcel;
    use httpmock::prelude::*;
    use httpmock::Method::HEAD;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn it_gets_scene_files_from_parcels() {
        let response = include_str!("../fixtures/scenes_from_parcels.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(POST)
                .path("/content/entities/active")
                .body_contains("{\"pointers\":[\"0,0\"]}");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let parcels = vec![Parcel(0, 0)];
        let result: Vec<SceneFile> =
            tokio_test::block_on(ContentClient::scene_files_for_parcels(&server, &parcels))
                .unwrap();

        m.assert();

        let expected: Vec<SceneFile> = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_implements_challenges() {
        let response = include_str!("../fixtures/challenge.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/content/challenge");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let result = tokio_test::block_on(ContentClient::challenge(&server)).unwrap();

        m.assert();

        let expected: Challenge = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_implements_content_file_exists() {
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(HEAD).path("/content/contents/a-cid");
            then.status(200);
        });

        let server = Server::new(server.url(""));
        let content_id = ContentId::new("a-cid".to_string());
        let result =
            tokio_test::block_on(ContentClient::content_file_exists(&server, &content_id)).unwrap();

        m.assert();
        assert!(result);

        let content_id = ContentId::new("invalid_cid".to_string());
        let result =
            tokio_test::block_on(ContentClient::content_file_exists(&server, &content_id)).unwrap();

        assert!(!result);
    }

    #[test]
    fn it_gets_entitiy_ids_by_hash() {
        let response = include_str!("../fixtures/entities_by_hash.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/content/contents/a-cid/entities");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let result = tokio_test::block_on(ContentClient::entity_ids_by_hash(
            &server,
            &"a-cid".to_string(),
        ))
        .unwrap();

        m.assert();

        let expected: Vec<ContentId> = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_deploy() {
        let response = include_str!("../fixtures/deploy_timestamp.json");

        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(POST).path("/content/entities");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let result =
            tokio_test::block_on(ContentClient::deploy_entity(&server, Form::default())).unwrap();

        m.assert();

        let expected: DeployResponse = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_gets_entities_by_urn() {
        let response = include_str!("../fixtures/entities_by_urn.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/content/entities/active/collections/a-urn");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let result =
            tokio_test::block_on(ContentClient::entities_by_urn(&server, "a-urn")).unwrap();

        m.assert();

        let expected: Vec<EntityData> = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_implements_failed_deployments() {
        let response = include_str!("../fixtures/failed_deployments.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/content/failed-deployments");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let result = tokio_test::block_on(ContentClient::failed_deployments(&server)).unwrap();

        m.assert();

        let expected: Vec<FailedDeployment> = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_implements_content_files_exist() {
        let response = include_str!("../fixtures/available_content.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/content/available-content/")
                .query_param("cid", "a-cid")
                .query_param("cid", "another-cid");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));
        let cids = vec![ContentId::new("a-cid"), ContentId::new("another-cid")];

        let result: Vec<ContentFileStatus> =
            tokio_test::block_on(ContentClient::content_files_exists(&server, &cids)).unwrap();

        m.assert();

        let expected = vec![
            ContentFileStatus {
                id: ContentId::new("a-cid"),
                available: true,
            },
            ContentFileStatus {
                id: ContentId::new("another-cid"),
                available: false,
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn it_gets_entity_information() {
        let response = include_str!("../fixtures/audit_scene_result.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/content/audit/scene/id");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));
        let entity = Entity::scene("id");
        let result: EntityInformation =
            tokio_test::block_on(ContentClient::entity_information(&server, &entity)).unwrap();

        m.assert();

        let expected: EntityInformation = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_gets_active_entities() {
        let response = "[\"entity-id\"]";
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET)
                .path("/content/contents/an-id/active-entities");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));
        let content_id = ContentId::new("an-id");
        let result =
            tokio_test::block_on(ContentClient::active_entities(&server, &content_id)).unwrap();

        m.assert();

        assert_eq!(result, vec!(EntityId::new("entity-id")));
    }

    #[test]
    fn it_gets_status() {
        let response = include_str!("../fixtures/content_server_status.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/content/status");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));
        let result: ContentServerStatus =
            tokio_test::block_on(ContentClient::status(&server)).unwrap();

        m.assert();

        let expected: ContentServerStatus = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_gets_snapshot() {
        let response = include_str!("../fixtures/snapshot.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path("/content/snapshot");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));
        let result: Snapshot = tokio_test::block_on(ContentClient::snapshot(&server)).unwrap();

        m.assert();

        let expected: Snapshot = serde_json::from_str(response).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn it_gets_scene_snapshot() {
        let response = include_str!("../fixtures/snapshot_entities_scene.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path(
                "/content/contents/bafybeiep3b54f6rzh5lgx647m4alfydi65smdz63y4gtpxnu2ero4trlsy",
            );
            then.status(200).body(response);
        });
        let server = Server::new(server.url(""));
        let snapshot: Snapshot =
            serde_json::from_str(include_str!("../fixtures/snapshot.json")).unwrap();

        let result: Vec<EntitySnapshot<Parcel>> = tokio_test::block_on(
            ContentClient::snapshot_entities(&server, EntityType::Scene, &snapshot),
        )
        .unwrap();

        m.assert();

        assert_eq!(result.len(), 23485);
    }

    #[test]
    fn it_gets_wearable_snapshot() {
        let response = include_str!("../fixtures/snapshot_entities_wearable.json");
        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.method(GET).path(
                "/content/contents/bafybeifk2e6dsuwqz24s5bwxvhvajinr7tb7n6jzwzvafd6q4pwuy3jmua",
            );
            then.status(200).body(response);
        });
        let server = Server::new(server.url(""));
        let snapshot: Snapshot =
            serde_json::from_str(include_str!("../fixtures/snapshot.json")).unwrap();

        let result: Vec<EntitySnapshot<Urn>> = tokio_test::block_on(
            ContentClient::snapshot_entities(&server, EntityType::Wearable, &snapshot),
        )
        .unwrap();

        m.assert();

        assert_eq!(result.len(), 17325);
    }

    #[test]
    fn it_downloads_file() {
        let response = "File Content";

        let server = MockServer::start();

        let m = server.mock(|when, then| {
            when.path("/content/contents/a-hash");
            then.status(200).body(response);
        });

        let server = Server::new(server.url(""));

        let tmp_dir = TempDir::new("content-client-test").unwrap();
        let filename = tmp_dir.path().join("test.txt");

        tokio_test::block_on(ContentClient::download(
            &server,
            ContentId::new("a-hash"),
            filename.clone(),
        ))
        .unwrap();

        m.assert();

        assert_eq!(fs::read_to_string(filename).unwrap(), "File Content");
    }
}
