use anyhow::Ok;
use async_trait::async_trait;
use demo_stf::genesis_config::StorageConfig;
use demo_stf::runtime::Runtime;
use sov_avail_adapter::verifier::Verifier;
use sov_modules_api::default_context::{DefaultContext, ZkDefaultContext};
use sov_avail_adapter::service::{DaProvider, DaServiceConfig};
use sov_avail_adapter::spec::DaLayerSpec;
use sov_db::ledger_db::LedgerDB;
use sov_modules_api::{Address, Spec};
use sov_modules_rollup_blueprint::{RollupBlueprint, WalletBlueprint};
use sov_modules_stf_blueprint::kernels::basic::BasicKernel;
use sov_modules_stf_blueprint::StfBlueprint;
use sov_rollup_interface::services::da::DaService;
use sov_rollup_interface::zk::ZkvmHost;
use sov_risc0_adapter::host::Risc0Host;
use sov_prover_storage_manager::ProverStorageManager;
use sov_state::{config, DefaultStorageSpec, Storage, ZkStorage};
use sov_stf_runner::{ParallelProverService, RollupConfig, RollupProverConfig};

use crate::{DEFAULT_POLLING_INTERVAL, DEFAULT_POLLING_TIMEOUT};
// use crate::{polling}
// Rollup with AvailDa
pub struct AvailDemoRollup {}

#[async_trait]
impl RollupBlueprint for AvailDemoRollup {
    type DaService = DaProvider;
    type DaSpec = DaLayerSpec;
    type DaConfig = DaServiceConfig;
    type Vm = Risc0Host<'static>;
    type ZkContext = ZkDefaultContext;
    type NativeContext = DefaultContext;
    type StorageManager = ProverStorageManager<DaLayerSpec, DefaultStorageSpec>;
    type ZkRuntime = Runtime<Self::ZkContext, Self::DaSpec>;
    type NativeRuntime = Runtime<Self::NativeContext, Self::DaSpec>;
    type NativeKernel = BasicKernel<Self::NativeContext, Self::DaSpec>;
    type ZkKernel = BasicKernel<Self::ZkContext, Self::DaSpec>;
    type ProverService = ParallelProverService<
        <<Self::NativeContext as Spec>::Storage as Storage>::Root,
        <<Self::NativeContext as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        Self::Vm,
        StfBlueprint<
            Self::ZkContext,
            Self::DaSpec,
            <Self::Vm as ZkvmHost>::Guest,
            Self::ZkRuntime,
            Self::ZkKernel,
        >,
    >;

    fn create_rpc_methods(
        &self, 
        storage: &<Self::NativeContext as sov_modules_api::Spec>::Storage, 
        ledger_db: &LedgerDB, 
        da_service: &Self::DaService,
    ) -> Result<rpc::RpcModule<()>, anyhow::Error> {
        let sequencer = Address::new([0; 32]);

        #[allow(unused_mut)]
        let mut rpc_methods = sov_modules_rollup_blueprint::register_rpc::<
            Self::NativeRuntime,
            Self::NativeContext,
            Self::DaService,
        >(storage, ledger_db, da_service, sequencer)?;

        Ok(rpc_methods) 
    }

    async fn create_da_service (
        &self,
        rollup_config: &RollupConfig<Self::DaConfig>,
    ) -> Self::DaService {
        DaProvider::new(
            // rollup_config.da.clone(),
            DaServiceConfig{
                light_client_url:String::from("sgs"),
                node_client_url:String::from("sfgs"),
                seed:String::from("ss"),
                polling_interval:DEFAULT_POLLING_INTERVAL,
                polling_timeout:DEFAULT_POLLING_TIMEOUT,
                app_id:1,
            },      
        )
        .await
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig, 
        rollup_config: &RollupConfig<Self::DaConfig>, 
        _da_service: &Self::DaService)
         -> Self::ProverService {
            let vm = Risc0Host::new(risc0::ROLLUP_ELF);
            let zk_stf = StfBlueprint::new();
            let zk_storage = ZkStorage::new();

            let da_verifier: verifier {}
        todo!()
    }

    fn create_storage_manager(
        &self,
        rollup_config: &sov_stf_runner::RollupConfig<Self::DaConfig>,
    ) -> Result<Self::StorageManager, anyhow::Error> {
        let storage_config = StorageConfig {
            path: rollup_config.storage.path.clone(),
        };
        ProverStorageManager::new(storage_config)
    }
}



impl WalletBlueprint for AvailDemoRollup {}

