use async_graphql::SimpleObject;
use chrono::Local;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::graphql::scalars::BigIntScalar;

#[derive(SimpleObject, Clone)]
pub struct BigIntTimeSerie {
    pub time: chrono::DateTime<Local>,
    pub value: BigIntScalar,
}

#[derive(SimpleObject, Clone)]
pub struct NumberTimeSerie {
    pub time: chrono::DateTime<Local>,
    pub value: i32,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct DiskUsage {
    pub used: BigIntScalar,
    pub used_last_month: BigIntScalar,
    pub used_range: Vec<BigIntTimeSerie>,
    pub free: BigIntScalar,
    pub free_last_month: BigIntScalar,
    pub free_range: Vec<BigIntTimeSerie>,
    pub total: BigIntScalar,
    pub total_last_month: BigIntScalar,
    pub total_range: Vec<BigIntTimeSerie>,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct PoolUsage {
    pub longest_chain: i32,
    pub longest_chain_range: Vec<NumberTimeSerie>,
    pub longest_chain_last_month: Option<i32>,

    pub nb_chunk: i32,
    pub nb_chunk_range: Vec<NumberTimeSerie>,
    pub nb_chunk_last_month: Option<i32>,

    pub nb_ref: i32,
    pub nb_ref_range: Vec<NumberTimeSerie>,
    pub nb_ref_last_month: Option<i32>,

    pub size: BigIntScalar,
    pub size_range: Vec<BigIntTimeSerie>,
    pub size_last_month: BigIntScalar,

    pub compressed_size: BigIntScalar,
    pub compressed_size_range: Vec<BigIntTimeSerie>,
    pub compressed_size_last_month: BigIntScalar,

    pub unused_size: BigIntScalar,
    pub unused_size_range: Vec<BigIntTimeSerie>,
    pub unused_size_last_month: BigIntScalar,
}

#[derive(SimpleObject, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct HostStatistics {
    pub host: String,

    pub longest_chain: i32,
    pub longest_chain_range: Vec<NumberTimeSerie>,
    pub longest_chain_last_month: Option<i32>,

    pub nb_chunk: i32,
    pub nb_chunk_range: Vec<NumberTimeSerie>,
    pub nb_chunk_last_month: Option<i32>,

    pub nb_ref: i32,
    pub nb_ref_range: Vec<NumberTimeSerie>,
    pub nb_ref_last_month: Option<i32>,

    pub size: BigIntScalar,
    pub size_range: Vec<BigIntTimeSerie>,
    pub size_last_month: BigIntScalar,

    pub compressed_size: BigIntScalar,
    pub compressed_size_range: Vec<BigIntTimeSerie>,
    pub compressed_size_last_month: BigIntScalar,
}

#[derive(Clone, Debug)]
pub struct GqlStatistics;

#[derive(SimpleObject, Clone)]
pub struct ServerInformations {
    pub hostname: String,
    pub uptime: u64,
    #[graphql(name = "woodstockVersion")]
    pub woodstock_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    pub hosts_count: usize,
    pub backups_count: usize,
    pub total_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
    pub last_backup: Option<u64>,
}
