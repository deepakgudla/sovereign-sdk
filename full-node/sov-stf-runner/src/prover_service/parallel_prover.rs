use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use super::{Hash, ProverService, ProverServiceError};
use crate::verifier::StateTransitionVerifier;
use crate::{ProofGenConfig, ProofSubmissionStatus, RollupProverConfig, StateTransitionData};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sov_rollup_interface::da::BlockHeaderTrait;
use sov_rollup_interface::da::DaSpec;
use sov_rollup_interface::services::da::DaService;
use sov_rollup_interface::stf::StateTransitionFunction;
use sov_rollup_interface::zk::{Proof, ZkvmHost};

enum ProverStatus<StateRoot, Witness, Da: DaSpec> {
    WitnessSubmitted(StateTransitionData<StateRoot, Witness, Da>),
    Proving,
    Proved(Proof),
    Err(anyhow::Error),
}

#[derive(Default)]
struct ProverState<StateRoot, Witness, Da: DaSpec> {
    prover_status: HashMap<Hash, ProverStatus<StateRoot, Witness, Da>>,
}

impl<StateRoot, Witness, Da: DaSpec> ProverState<StateRoot, Witness, Da> {
    fn remove(&mut self, hash: &Hash) -> Option<ProverStatus<StateRoot, Witness, Da>> {
        self.prover_status.remove(hash)
    }

    fn set_to_proving(&mut self, hash: Hash) -> Option<ProverStatus<StateRoot, Witness, Da>> {
        self.prover_status.insert(hash, ProverStatus::Proving)
    }

    fn set_to_proved(
        &mut self,
        hash: Hash,
        proof: Result<Proof, anyhow::Error>,
    ) -> Option<ProverStatus<StateRoot, Witness, Da>> {
        match proof {
            Ok(p) => self.prover_status.insert(hash, ProverStatus::Proved(p)),
            Err(e) => self.prover_status.insert(hash, ProverStatus::Err(e)),
        }
    }

    fn get_proover_status(&self, hash: Hash) -> Option<&ProverStatus<StateRoot, Witness, Da>> {
        self.prover_status.get(&hash)
    }
}

struct Prover<StateRoot, Witness, Da: DaService> {
    prover_state: Arc<Mutex<ProverState<StateRoot, Witness, Da::Spec>>>,
}

impl<StateRoot, Witness, Da> Prover<StateRoot, Witness, Da>
where
    Da: DaService,
    StateRoot: Serialize + DeserializeOwned + Clone + AsRef<[u8]> + Send + Sync + 'static,
    Witness: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn new() -> Self {
        Self {
            prover_state: Arc::new(Mutex::new(ProverState {
                prover_status: Default::default(),
            })),
        }
    }

    fn submit_witness(
        &self,
        state_transition_data: StateTransitionData<StateRoot, Witness, Da::Spec>,
    ) {
        let header_hash = state_transition_data.da_block_header.hash().into();
        let data = ProverStatus::WitnessSubmitted(state_transition_data);

        self.prover_state
            .lock()
            .unwrap()
            .prover_status
            .insert(header_hash, data);
    }

    fn start_proving<Vm, V>(
        &self,
        block_header_hash: Hash,
        config: Arc<ProofGenConfig<V, Da, Vm>>,
        mut vm: Vm,
        zk_storage: V::PreState,
    ) -> Result<(), ProverServiceError>
    where
        Vm: ZkvmHost + 'static,
        V: StateTransitionFunction<Vm::Guest, Da::Spec> + Send + Sync + 'static,
        V::PreState: Send + Sync + 'static,
    {
        let prover_state_clone = self.prover_state.clone();

        let mut prover_state = self.prover_state.lock().expect("Lock was poisoned");
        let prover_status = prover_state.remove(&block_header_hash).unwrap(); // TODO

        match prover_status {
            ProverStatus::WitnessSubmitted(state_tranistion_data) => {
                prover_state.set_to_proving(block_header_hash);
                vm.add_hint(state_tranistion_data);

                rayon::spawn(move || {
                    let mut prover_state = prover_state_clone.lock().expect("Lock was poisoned");

                    tracing::info_span!("guest_execution").in_scope(|| {
                        let proof = match config.deref() {
                            ProofGenConfig::Simulate(verifier) => verifier
                                .run_block(vm.simulate_with_hints(), zk_storage)
                                .map(|_| Proof::Empty)
                                .map_err(|e| {
                                    anyhow::anyhow!(
                                        "Guest execution must succeed but failed with {:?}",
                                        e
                                    )
                                }),
                            ProofGenConfig::Execute => vm.run(false),
                            ProofGenConfig::Prover => vm.run(true),
                        };

                        prover_state.set_to_proved(block_header_hash, proof);
                    })
                });

                Ok(())
            }
            ProverStatus::Proving => todo!(),
            ProverStatus::Proved(_) => todo!(),
            ProverStatus::Err(e) => Err(e.into()),
        }
    }

    fn get_proof_submission_status(&self, block_header_hash: Hash) -> ProofSubmissionStatus {
        let prover_state = self.prover_state.lock().unwrap();
        let status = prover_state.get_proover_status(block_header_hash);

        match status {
            Some(ProverStatus::Proving) => ProofSubmissionStatus::ProvingInProgress,
            Some(ProverStatus::Proved(_)) => ProofSubmissionStatus::Sucess,
            Some(ProverStatus::WitnessSubmitted(_)) => todo!(),
            Some(ProverStatus::Err(e)) => {
                ProofSubmissionStatus::Err(anyhow::anyhow!(e.to_string()))
            }
            None => todo!(),
        }
    }
}

/// TODO
pub struct ParallelProver<StateRoot, Witness, Da, Vm, V>
where
    StateRoot: Serialize + DeserializeOwned + Clone + AsRef<[u8]>,
    Witness: Serialize + DeserializeOwned,
    Da: DaService,
    Vm: ZkvmHost,
    V: StateTransitionFunction<Vm::Guest, Da::Spec> + Send + Sync,
{
    vm: Vm,
    prover_config: Option<Arc<ProofGenConfig<V, Da, Vm>>>,
    zk_storage: V::PreState,
    prover_state: Prover<StateRoot, Witness, Da>,
}

impl<StateRoot, Witness, Da, Vm, V> ParallelProver<StateRoot, Witness, Da, Vm, V>
where
    StateRoot: Serialize + DeserializeOwned + Clone + AsRef<[u8]> + Send + Sync + 'static,
    Witness: Serialize + DeserializeOwned + Send + Sync + 'static,
    Da: DaService,
    Vm: ZkvmHost,
    V: StateTransitionFunction<Vm::Guest, Da::Spec> + Send + Sync,
    V::PreState: Clone + Send + Sync,
{
    /// Creates a new prover.
    pub fn new(
        vm: Vm,
        zk_stf: V,
        da_verifier: Da::Verifier,
        config: Option<RollupProverConfig>,
        zk_storage: V::PreState,
    ) -> Self {
        let prover_config = config.map(|config| {
            let stf_verifier =
                StateTransitionVerifier::<V, Da::Verifier, Vm::Guest>::new(zk_stf, da_verifier);

            let config: ProofGenConfig<V, Da, Vm> = match config {
                RollupProverConfig::Simulate => ProofGenConfig::Simulate(stf_verifier),
                RollupProverConfig::Execute => ProofGenConfig::Execute,
                RollupProverConfig::Prove => ProofGenConfig::Prover,
            };

            Arc::new(config)
        });

        Self {
            vm,
            prover_config,
            prover_state: Prover::new(),
            zk_storage,
        }
    }
}

#[async_trait]
impl<StateRoot, Witness, Da, Vm, V> ProverService for ParallelProver<StateRoot, Witness, Da, Vm, V>
where
    StateRoot: Serialize + DeserializeOwned + Clone + AsRef<[u8]> + Send + Sync + 'static,
    Witness: Serialize + DeserializeOwned + Send + Sync + 'static,
    Da: DaService,
    Vm: ZkvmHost + 'static,
    V: StateTransitionFunction<Vm::Guest, Da::Spec> + Send + Sync + 'static,
    V::PreState: Clone + Send + Sync,
{
    type StateRoot = StateRoot;

    type Witness = Witness;

    type DaService = Da;

    async fn submit_witness(
        &self,
        state_transition_data: StateTransitionData<
            Self::StateRoot,
            Self::Witness,
            <Self::DaService as DaService>::Spec,
        >,
    ) {
        self.prover_state.submit_witness(state_transition_data);
    }

    async fn prove(&self, block_header_hash: Hash) -> Result<(), ProverServiceError> {
        if let Some(config) = self.prover_config.clone() {
            let vm = self.vm.clone();
            let zk_storage = self.zk_storage.clone();

            self.prover_state
                .start_proving(block_header_hash, config, vm, zk_storage)?;
        }
        Ok(())
    }

    async fn send_proof_to_da(&self, block_header_hash: Hash) -> ProofSubmissionStatus {
        self.prover_state
            .get_proof_submission_status(block_header_hash)
    }
}
