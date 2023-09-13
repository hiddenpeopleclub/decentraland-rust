mod error;
use std::{
    collections::HashMap,
    error::Error,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use catalyst::{ContentFile, ContentId, DeployResponse, Entity, EntityId, EntityType, Metadata};
use cid::{multihash::MultihashDigest, Cid};
use dcl_common::{Parcel, Result};
use dcl_crypto::{AuthChain, AuthLink};
use error::SceneDeployError;

pub struct FileData {
    cid: String,
    bytes: Vec<u8>,
    mime_str: String,
}

pub async fn deploy(
    entity_id: EntityId,
    deploy_data: Vec<FileData>,
    auth_chain: AuthChain,
    server: catalyst::Server,
) -> Result<DeployResponse> {
    if let Some(entity) = find_entity(&deploy_data) {
        if let Some(Metadata::Scene(scene_3d)) = entity.metadata {
            let expected_parcels = parcels_vec_to_strings_vec(&scene_3d.scene.parcels);
            if expected_parcels != entity.pointers {
                return Err(Box::new(SceneDeployError::InvalidPointers {
                    parcels_found: entity.pointers,
                    expected_parcels,
                }) as Box<dyn Error>);
            }

            let scene_files = catalyst::ContentClient::scene_entities_for_parcels(
                &server,
                &scene_3d.scene.parcels,
            )
            .await?;
            if !scene_files.is_empty()
                && (scene_files.len() > 1 || scene_files[0].pointers != entity.pointers)
            {
                return Err(Box::new(SceneDeployError::InvalidPointers {
                    expected_parcels: scene_files[0].pointers.clone(),
                    parcels_found: entity.pointers,
                }) as Box<dyn Error>);
            }
        }
    } else {
        return Err(Box::new(SceneDeployError::MissingSceneEntity) as Box<dyn Error>);
    }

    let form =
        build_entity_form_data_for_deployment(entity_id.to_string(), deploy_data, auth_chain);
    catalyst::ContentClient::deploy_entity(&server, form).await
}

pub fn build_entity_form_data_for_deployment(
    entity_id: String,
    deploy_data: Vec<FileData>,
    auth_chain: AuthChain,
) -> reqwest::multipart::Form {
    let mut form = reqwest::multipart::Form::new();
    form = form.part(
        "entityId",
        reqwest::multipart::Part::text(entity_id.clone()),
    );

    for (index, auth_link) in (0..).zip(auth_chain.iter()) {
        form = form.part(
            format!("authChain[{}][type]", index),
            reqwest::multipart::Part::text((*auth_link.kind()).to_string()),
        );
        match auth_link {
            AuthLink::Signer { payload, signature } => {
                form = form.part(
                    format!("authChain[{}][payload]", index),
                    reqwest::multipart::Part::text(payload.to_string()),
                );
                form = form.part(
                    format!("authChain[{}][signature]", index),
                    reqwest::multipart::Part::text(signature.clone()),
                );
            }
            AuthLink::EcdsaPersonalEphemeral { payload, signature } => {
                form = form.part(
                    format!("authChain[{}][payload]", index),
                    reqwest::multipart::Part::text(payload.to_string()),
                );
                form = form.part(
                    format!("authChain[{}][signature]", index),
                    reqwest::multipart::Part::text(signature.to_string()),
                );
            }
            AuthLink::EcdsaPersonalSignedEntity { payload, signature } => {
                form = form.part(
                    format!("authChain[{}][payload]", index),
                    reqwest::multipart::Part::text(payload.to_string()),
                );
                form = form.part(
                    format!("authChain[{}][signature]", index),
                    reqwest::multipart::Part::text(signature.to_string()),
                );
            }
            AuthLink::EcdsaEip1654Ephemeral { payload, signature } => {
                form = form.part(
                    format!("authChain[{}][payload]", index),
                    reqwest::multipart::Part::text(payload.to_string()),
                );
                form = form.part(
                    format!("authChain[{}][signature]", index),
                    reqwest::multipart::Part::text(signature.to_string()),
                );
            }
            AuthLink::EcdsaEip1654SignedEntity { payload, signature } => {
                form = form.part(
                    format!("authChain[{}][payload]", index),
                    reqwest::multipart::Part::text(payload.to_string()),
                );
                form = form.part(
                    format!("authChain[{}][signature]", index),
                    reqwest::multipart::Part::text(signature.to_string()),
                );
            }
        }
    }

    for file in deploy_data {
        let part = reqwest::multipart::Part::bytes(file.bytes)
            .file_name(file.cid.clone())
            .mime_str(&file.mime_str)
            .unwrap();

        form = form.part(file.cid.clone(), part);
    }

    form
}

fn get_cid(content: &[u8]) -> String {
    let codec: u64 = 0x55;
    let h = cid::multihash::Code::Sha2_256.digest(content);
    Cid::new_v1(codec, h).to_string()
}

pub fn build_entity_scene(
    pointers: Vec<String>,
    files: HashMap<String, Vec<u8>>,
    entity: &Entity,
) -> (Vec<FileData>, EntityId) {
    let mut content = entity.content.clone();
    let mut files_data = Vec::default();

    for i in (0..content.len()).rev() {
        if content[i].filename.starts_with("./2dcl")
            || content[i].filename.starts_with("/2dcl")
            || content[i].filename.starts_with("2dcl")
        {
            content.remove(i);
        }
    }

    for (path, bytes) in files {
        let cid = get_cid(&bytes);
        let filename = PathBuf::from_str(&path).unwrap();
        content.push(ContentFile {
            filename,
            cid: ContentId::new(cid.clone()),
        });

        let mime_str = match path.ends_with(".png") {
            true => "image/png".to_string(),
            false => "application/octet-stream".to_string(),
        };
        files_data.push(FileData {
            cid,
            bytes,
            mime_str,
        });
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut entity = Entity {
        id: EntityId(String::default()),
        version: "v3".to_string(),
        kind: EntityType::Scene,
        pointers,
        timestamp,
        content,
        metadata: entity.metadata.clone(),
    };

    let entity_file = serde_json::to_string(&entity).unwrap();
    let entity_id = get_cid(entity_file.as_bytes());
    entity.id = EntityId(entity_id.clone());

    files_data.push(FileData {
        cid: entity_id.clone(),
        bytes: entity_file.as_bytes().to_vec(),
        mime_str: "application/octet-stream".to_string(),
    });
    (files_data, EntityId(entity_id))
}

fn find_entity(files_data: &Vec<FileData>) -> Option<Entity> {
    for file in files_data {
        if file.mime_str == *"application/octet-stream" {
            if let Ok(scene_file) = serde_json::from_slice::<Entity>(&file.bytes) {
                return Some(scene_file);
            };
        }
    }

    None
}

fn parcels_vec_to_strings_vec(parcels: &Vec<Parcel>) -> Vec<String> {
    let mut output = Vec::default();
    for parcel in parcels {
        output.push(format!("{},{}", parcel.0, parcel.1));
    }

    output
}
