use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::{dummy_fs::meta_map::MetaMap, sys::NodeID};

#[derive(Debug, Clone)]
pub struct NodesConfig {
    pub peers: HashMap<NodeID, NodeConfig>,
    pub this: (NodeID, NodeConfig),
    pub file_dir: PathBuf,
    pub s3_node: NodeID,
    pub router_node: NodeID,
    pub block_size: u32,
}

impl NodesConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub addr: SocketAddr,
    pub spec: HashSet<String>,
}

impl NodeConfig {
    pub fn is_fs(&self) -> bool {
        self.spec.contains("fs")
    }
    pub fn is_s3(&self) -> bool {
        self.spec.contains("s3")
    }
    pub fn is_router(&self) -> bool {
        self.spec.contains("router")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlNodeConfig {
    pub nodes: HashMap<NodeID, NodeConfig>,
    random_seed: String,
    block_size: u32,
    // pub this: NodeID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlUser {
    pub access: String,
    pub targets: Vec<String>,
}

pub fn read_yaml<T: DeserializeOwned>(file_path: impl AsRef<Path>) -> T {
    let file = std::fs::File::open(file_path).unwrap_or_else(|err| {
        panic!("open config file failed, err: {:?}", err);
    });
    serde_yaml::from_reader(file).unwrap_or_else(|e| {
        panic!("parse yaml config file failed, err: {:?}", e);
    })
}

pub fn read_yaml_users(file_path: impl AsRef<Path>) -> HashMap<NodeID, YamlUser> {
    let file_path = file_path.as_ref().join("user.yaml");
    read_yaml(file_path)
}

pub fn read_yaml_file_map(file_path: impl AsRef<Path>) -> MetaMap {
    let file_path = file_path.as_ref().join("file_map.yaml");
    MetaMap::new(read_yaml::<HashMap<String, u32>>(file_path))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YamlFileMap {
    pub access: String,
    pub targets: Vec<String>,
}

pub fn read_yaml_config<T: DeserializeOwned>(file_path: impl AsRef<Path>) -> T {
    let file_path = file_path.as_ref().join("node_config.yaml");
    read_yaml(file_path)
}

pub fn read_config(this_id: NodeID, file_path: impl AsRef<Path>) -> NodesConfig {
    let mut yaml_config: YamlNodeConfig = read_yaml_config(file_path.as_ref().to_path_buf());

    NodesConfig {
        s3_node: yaml_config
            .nodes
            .iter()
            .find(|(_nid, config)| config.is_s3())
            .map(|v| *v.0)
            .unwrap(),
        router_node: yaml_config
            .nodes
            .iter()
            .find(|(_nid, config)| config.is_router())
            .map(|v| *v.0)
            .unwrap(),

        this: (this_id, yaml_config.nodes.remove(&this_id).unwrap()),
        peers: yaml_config.nodes,
        file_dir: file_path.as_ref().to_path_buf(),
        block_size: yaml_config.block_size,
    }
}
