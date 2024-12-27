use clap::{arg, command, Parser};
use rust_rocksdb::DB;
use std::path::PathBuf;
use tonic::transport::Server;

use libplanet_kvstore::key_value_store_server::{KeyValueStore, KeyValueStoreServer};
use libplanet_kvstore::{
    DeleteValueRequest, DeleteValuesRequest, ExistsKeyRequest, ExistsKeyResponse, GetValueRequest,
    KeyValueStoreValue, ListKeysRequest, ListKeysResponse, SetValueRequest, SetValuesRequest,
    SetValuesResponse,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to primary rocksdb
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Port to listen gRPC requests.
    #[arg(short, long)]
    port: Option<u16>,

    /// UNUSED option for compatibility with Libplanet.Store.Remote.Executable.
    #[arg(long)]
    http_port: Option<u16>,
}

pub mod libplanet_kvstore {
    tonic::include_proto!("libplanet.rpc.v1");
}

pub struct KeyValueService {
    database: DB,
}

#[tonic::async_trait]
impl KeyValueStore for KeyValueService {
    async fn get_value(
        &self,
        request: tonic::Request<GetValueRequest>,
    ) -> std::result::Result<tonic::Response<KeyValueStoreValue>, tonic::Status> {
        let key = match &request.get_ref().key {
            Some(key) => &key.data,
            None => return Err(tonic::Status::invalid_argument("'key' must be non-null.")),
        };
        match self.database.get(key) {
            Ok(value) => match value {
                Some(value) => Ok(tonic::Response::new(KeyValueStoreValue { data: value })),
                None => Err(tonic::Status::not_found("Not found")),
            },
            Err(_) => Err(tonic::Status::unknown("Unknown error.")),
        }
    }

    async fn set_value(
        &self,
        _request: tonic::Request<SetValueRequest>,
    ) -> std::result::Result<tonic::Response<KeyValueStoreValue>, tonic::Status> {
        Err(tonic::Status::permission_denied(
            "'set_value' isn't supported by this implementations.",
        ))
    }

    async fn set_values(
        &self,
        _request: tonic::Request<SetValuesRequest>,
    ) -> std::result::Result<tonic::Response<SetValuesResponse>, tonic::Status> {
        Err(tonic::Status::permission_denied(
            "'set_values' isn't supported by this implementations.",
        ))
    }

    async fn delete_value(
        &self,
        _request: tonic::Request<DeleteValueRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::permission_denied(
            "'delete_value' isn't supported by this implementations.",
        ))
    }

    async fn delete_values(
        &self,
        _request: tonic::Request<DeleteValuesRequest>,
    ) -> std::result::Result<tonic::Response<()>, tonic::Status> {
        Err(tonic::Status::permission_denied(
            "'delete_values' isn't supported by this implementations.",
        ))
    }

    async fn exists_key(
        &self,
        request: tonic::Request<ExistsKeyRequest>,
    ) -> std::result::Result<tonic::Response<ExistsKeyResponse>, tonic::Status> {
        let key = match &request.get_ref().key {
            Some(key) => &key.data,
            None => return Err(tonic::Status::invalid_argument("'key' must be non-null.")),
        };
        Ok(tonic::Response::new(ExistsKeyResponse {
            exists: self.database.key_may_exist(key),
        }))
    }

    async fn list_keys(
        &self,
        _request: tonic::Request<ListKeysRequest>,
    ) -> std::result::Result<tonic::Response<ListKeysResponse>, tonic::Status> {
        Err(tonic::Status::permission_denied("'list_keys' isn't supported by this implementations because it is too heavy operation."))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let rocksdb = rust_rocksdb::DB::open_as_secondary(
        &rust_rocksdb::Options::default(),
        cli.path,
        std::env::temp_dir(),
    )?;

    let addr = format!("[::1]:{}", cli.port.unwrap_or(5000))
        .parse()
        .unwrap();
    let key_value_service = KeyValueService { database: rocksdb };

    println!("KeyValueService listening on {}", addr);

    Server::builder()
        .add_service(KeyValueStoreServer::new(key_value_service))
        .serve(addr)
        .await?;

    Ok(())
}
