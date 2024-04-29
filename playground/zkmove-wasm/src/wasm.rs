// Copyright (c) zkMove Authors

extern crate wasm_bindgen;
use std::io::BufReader;

// use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
// use halo2_proofs::{circuit::Value, dev::MockProver};
// use halo2_proofs::halo2curves::{
//     pasta::Fp,
// };
// use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use halo2_proofs::plonk::{keygen_pk, keygen_vk};
// use halo2_proofs::poly::kzg::{
//     strategy::SingleStrategy,
// };
// use halo2_proofs::poly::{ipa, kzg};
// use halo2_proofs::poly::{commitment::Params};
// use halo2_proofs::halo2curves::pasta::EqAffine;
// use rand_core::OsRng;
// use std::fs::File;
// use std::io::{self, Write, Read, BufReader, BufWriter};
use wasm_bindgen::prelude::*;
// use js_sys::Uint8Array;
// use wasm_bindgen::JsValue;

use error::{RuntimeError, StatusCode};
use halo2_proofs::{
    dev::MockProver,
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::*,
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
};
use logger::prelude::*;
use move_binary_format::file_format::empty_script;
use move_binary_format::file_format::Bytecode as MoveBytecode;
use movelang::compiler::compile_source_files;
use rand::{rngs::StdRng, SeedableRng};
use std::marker::PhantomData;
use vm::runtime::Runtime;
use vm::state::StateStore;
use vm_circuit::circuit::VmCircuit;
use vm_circuit::witness::CircuitConfig;
use vm_circuit::{find_best_k, proof_vm_circuit_kzg, verify_vm_circuit_kzg};

#[cfg(all(target_family = "wasm", feature = "rayon"))]
pub use wasm_bindgen_rayon::init_thread_pool;

extern crate console_error_panic_hook;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
    pub fn log(s: &str);
}

#[wasm_bindgen(js_name = initPanicHook)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct VMWasm {
    circuit: Option<VmCircuit<Fr>>,
}

impl Default for VMWasm {
    fn default() -> Self {
        log!("vm default creating");
        let mut script = empty_script();
        script.code.code = vec![
            MoveBytecode::LdU64(1u64),
            MoveBytecode::LdU64(2u64),
            MoveBytecode::Add,
            MoveBytecode::Pop,
            MoveBytecode::Ret,
        ];
        let runtime = Runtime::new();
        let mut data_store = StateStore::new();
        let circuit_config = CircuitConfig::default()
            .stack_ops_num(Some(20))
            .locals_ops_num(Some(20));
        let trace = runtime
            .execute_script(script.clone(), vec![], None, None, &mut data_store)
            .expect("execute script failed.");
        let witness = runtime
            .process_execution_trace(vec![], Some(script), None, vec![], trace, circuit_config)
            .expect("process execution trace failed.");
        let vm_circuit = VmCircuit {
            witness,
            public_input: None,
            _maker: PhantomData,
        };

        log!("vm default done");
        VMWasm {
            circuit: Some(vm_circuit),
        }
    }
}

#[wasm_bindgen]
impl VMWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::type_complexity)]
    pub fn gen_witness(script_file: &str) -> VMWasm {
        log!("Run test {:?}", script_file);
        let targets = vec![script_file.to_string()];
        let (compiled_script, _compiled_modules) =
            compile_source_files(targets).expect("compile file failed");
        let script = compiled_script.expect("script is missing");
        let runtime = Runtime::new();
        let mut data_store = StateStore::new();
        let circuit_config = CircuitConfig::default()
            .stack_ops_num(Some(20))
            .locals_ops_num(Some(20));
        let trace = runtime
            .execute_script(script.clone(), vec![], None, None, &mut data_store)
            .expect("execute script failed.");
        let witness = runtime
            .process_execution_trace(vec![], Some(script), None, vec![], trace, circuit_config)
            .expect("process execution trace failed.");
        let vm_circuit = VmCircuit {
            witness,
            public_input: None,
            _maker: PhantomData,
        };

        log!("gen witness finished");
        VMWasm {
            circuit: Some(vm_circuit),
        }
    }
}

#[wasm_bindgen]
pub struct VMCircuitWasm {
    circuit: Option<VmCircuit<Fr>>,
    params: Option<ParamsKZG<Bn256>>,
    pk: Option<ProvingKey<G1Affine>>,
    vk: Option<VerifyingKey<G1Affine>>,
}

#[wasm_bindgen]
impl VMCircuitWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        VMCircuitWasm {
            circuit: None,
            params: None,
            pk: None,
            vk: None,
        }
    }

    pub fn init_params(&mut self) {
        let circuit = self.circuit.clone().expect("circuit is none");
        let k = find_best_k(&circuit);
        let rng = StdRng::from_entropy();
        let params = ParamsKZG::<Bn256>::setup(k, rng);
        self.params = Some(params);
    }

    #[wasm_bindgen(js_name = loadParams)]
    pub fn load_params(&mut self, params: &[u8]) {
        let params = ParamsKZG::<Bn256>::read(&mut BufReader::new(params)).expect("params is none");
        self.params = Some(params);
    }

    #[wasm_bindgen(js_name = setupCircuit)]
    pub fn setup_circuit(&mut self) {
        log!("setup circuit start");
        // TODO. file can't be access within WASM.
        //let vmwasm = VMWasm::gen_witness("data/scripts/add.move");
        let vmwasm = VMWasm::new();
        self.circuit = vmwasm.circuit;
        self.init_params();
        log!("setup circuit successfully!");
    }

    #[wasm_bindgen]
    pub fn mock(&self) -> Result<(), JsValue> {
        log!("Test circuit start");
        let circuit = self.circuit.clone().expect("circuit is none");
        let params = self.params.as_ref().expect("params is none");
        let k = params.k();

        let prover = MockProver::<Fr>::run(k, &circuit, vec![vec![Fr::zero()]])
            .map_err(|e| {
                debug!("Prover Error: {:?}", e);
                RuntimeError::new(StatusCode::ProofSystemError(e))
            })
            .expect("mock run failed");
        assert_eq!(prover.verify(), Ok(()));

        log!("Test circuit successfully!");
        Ok(())
    }

    #[wasm_bindgen(js_name = genVk)]
    pub fn gen_vk(&mut self) {
        log!("generate vk start");
        let params = self.params.as_ref().unwrap();
        let vk = keygen_vk(params, self.circuit.as_ref().unwrap()).expect("vk should not fail");
        self.vk = Some(vk);
        log!("generate vk successfully");
    }

    #[wasm_bindgen(js_name = genPk)]
    pub fn gen_pk(&mut self) {
        log!("generate pk start");
        let vk = self.vk.clone().unwrap();
        let params = self.params.as_ref().unwrap();
        let pk = keygen_pk(params, vk, self.circuit.as_ref().unwrap()).expect("pk should not fail");
        self.pk = Some(pk);
        log!("generate pk successfully");
    }

    #[wasm_bindgen(js_name = genProof)]
    pub fn gen_proof(&self) -> Vec<u8> {
        log!("proof generate start");
        let circuit = self.circuit.clone().unwrap();
        let params = self.params.clone().unwrap();
        let pk = self.pk.clone().unwrap();
        let proof =
            proof_vm_circuit_kzg(circuit, &[&[Fr::zero()]], &params, pk).expect("gen proof failed");
        log!("proof generate successfully");
        proof
    }

    pub fn verify(&self, proof: Vec<u8>) -> Result<(), JsValue> {
        log!("proof verify start");
        let circuit = self.circuit.clone().unwrap();
        let params = self.params.clone().unwrap();
        let pk = self.pk.clone().unwrap();
        let result = verify_vm_circuit_kzg(circuit, &[&[Fr::zero()]], &params, pk, proof);
        assert!(result.is_ok());
        log!("proof verify successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::wasm::VMCircuitWasm;

    #[wasm_bindgen_test]
    async fn test_js() {
        let mut vmcircuit = VMCircuitWasm::new();
        vmcircuit.setup_circuit();
        let _ = vmcircuit.mock();
        vmcircuit.gen_vk();
        vmcircuit.gen_pk();
        vmcircuit.gen_proof();
    }
}
