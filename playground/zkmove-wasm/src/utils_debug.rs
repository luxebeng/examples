use crate::utils_input::Inputs;
use crate::utils_program::{ZkMoveProgram, DEBUG_ON};
use miden_vm::{math::StarkField, VmState, VmStateIterator, Word, DefaultHost, MemAdviceProvider};
use wasm_bindgen::prelude::*;

/// Stub struct and functions

// pub type Word = [u64; 4];
// pub struct VmStateIterator {
//     // chiplets: Chiplets,
//     // decoder: Decoder,
//     // stack: Stack,
//     // system: System,
//     // error: Option<ExecutionError>,
//     clk: u32,
//     asmop_idx: usize,
//     forward: bool,
//     // trace_len_summary: TraceLenSummary,
// }

// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct VmState {
//     pub clk: u32,
//     pub ctx: u32,
//     // pub op: Option<Operation>,
//     // pub asmop: Option<AsmOpInfo>,
//     // pub fmp: Felt,
//     // pub stack: Vec<Felt>,
//     pub memory: Vec<(u64, Word)>,
// }

// This is the main struct that will be exported to JS
// It will be used to execute debug commands against the VM
#[wasm_bindgen]
pub struct DebugExecutor {
    #[allow(dead_code)]
    vm_state_iter: VmStateIterator,
    vm_state: VmState,
}

// This is how the results will be exported to JS
#[wasm_bindgen(getter_with_clone)]
pub struct DebugOutput {
    pub clk: u32,
    pub op: Option<String>,
    pub instruction: Option<String>,
    pub num_of_operations: Option<u8>,
    pub operation_index: Option<u8>,
    pub stack: Vec<u64>,
    pub memory: Vec<u64>,
}

// This describes what the user can do with the DebugExecutor
#[wasm_bindgen]
pub enum DebugCommand {
    PlayAll,
    Play,
    RewindAll,
    Rewind,
    PrintState,
}

#[wasm_bindgen]
impl DebugExecutor {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new DebugExecutor for the specified program, inputs and advice provider.
    ///
    /// # Errors
    /// Returns an error if the command cannot be parsed.
    #[wasm_bindgen(constructor)]
    pub fn new(code_frontend: &str, inputs_frontend: &str) -> Result<DebugExecutor, String> {
        let mut program = ZkMoveProgram::new(code_frontend, DEBUG_ON);
        program.compile_program().unwrap();

        let mut inputs = Inputs::new();
        inputs.deserialize_inputs(inputs_frontend).unwrap();

        let host = DefaultHost::new(MemAdviceProvider::from(inputs.advice_provider));

        let mut vm_state_iter = miden_vm::execute_iter(
            &program.program.unwrap(),
            inputs.stack_inputs,
            host,
        );
        let vm_state = vm_state_iter
            .next()
            .ok_or("Failed to instantiate DebugExecutor - `VmStateIterator` is not yielding!")?
            .expect("initial state of vm must be healthy!");

        // let mut vm_state_iter = VmStateIterator {
        //     clk: 0,
        //     asmop_idx: 0,
        //     forward: false,  
        // };
        // let vm_state = VmState {
        //     clk: 0,
        //     ctx: 0,
        //     memory: Vec::new(),
        // };
        Ok(Self {
            vm_state_iter,
            vm_state,
        })
    }

    // MODIFIERS
    // --------------------------------------------------------------------------------------------

    /// executes a debug command against the vm in it's current state.
    pub fn execute(&mut self, command: DebugCommand, param: Option<u64>) -> DebugOutput {
        match command {
            DebugCommand::PlayAll => {
                while let Some(new_vm_state) = self.next_vm_state() {
                    self.vm_state = new_vm_state;
                    if self.should_break() {
                        break;
                    }
                }
                self.vm_state_to_output()
            }
            DebugCommand::Play => {
                for _cycle in 0..param.unwrap() {
                    match self.next_vm_state() {
                        Some(next_vm_state) => {
                            self.vm_state = next_vm_state;
                            if self.should_break() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                self.vm_state_to_output()
            }
            DebugCommand::RewindAll => {
                while let Some(new_vm_state) = self.prev_vm_state() {
                    self.vm_state = new_vm_state;
                    if self.should_break() {
                        break;
                    }
                }
                self.vm_state_to_output()
            }
            DebugCommand::Rewind => {
                for _cycle in 0..param.unwrap() {
                    match self.prev_vm_state() {
                        Some(new_vm_state) => {
                            self.vm_state = new_vm_state;
                            if self.should_break() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                self.vm_state_to_output()
            }
            DebugCommand::PrintState => self.vm_state_to_output(),
        }
    }

    /// iterates to the next clock cycle.
    fn next_vm_state(&mut self) -> Option<VmState> {
        // match self.vm_state_iter.next() {
        //     Some(Ok(vm_state)) => Some(vm_state),
        //     Some(Err(err)) => {
        //         println!("Execution error: {err:?}");
        //         None
        //     }
        //     None => {
        //         println!("Program execution complete.");
        //         None
        //     }
        // }
        None
    }

    /// iterates to the previous clock cycle.
    fn prev_vm_state(&mut self) -> Option<VmState> {
        // self.vm_state_iter.back()
         None
    }

    // ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// print general VM state information.
    fn vm_state_to_output(&self) -> DebugOutput {
        let memory: Vec<(u64, [u64; 4])> = self
            .vm_state
            .memory
            .iter()
            .map(|x| (x.0, word_to_ints(&x.1)))
            .collect();

        let output = DebugOutput {
            clk: self.vm_state.clk,
            //op: self.vm_state.op.map(|v| format!("{:?}", v)),
            //instruction: self.vm_state.asmop.clone().map(|v| format!("{:?}", v.op())),
            //num_of_operations: self.vm_state.asmop.clone().map(|v| v.num_cycles()),
            //operation_index: self.vm_state.asmop.clone().map(|v| v.cycle_idx()),
            //stack: self.vm_state.stack.iter().map(|x| x.as_int()).collect(),
            op: None,
            instruction: None,
            num_of_operations: None,
            operation_index: None,
            stack: Vec::new(),
            memory: transform_2d_to_1d(memory),
        };

        output
    }

    /// Returns `true` if the current state should break.
    fn should_break(&self) -> bool {
        // self.vm_state
        //     .asmop
        //     .as_ref()
        //     .map(|asm| asm.should_break())
        //     .unwrap_or(false)
        false
    }
}

// Helper functions

/// This converts a word to a tuple of 4 u64s.
fn word_to_ints(word: &Word) -> [u64; 4] {
    [
        word[0].as_int(),
        word[1].as_int(),
        word[2].as_int(),
        word[3].as_int(),
    ]
}

/// This converts the memory tuple Vec<(u64, [u64; 4])> to a single Vec<u64>.
/// This is necessary because wasm_bindgen does not support arrays of arrays.
fn transform_2d_to_1d(input: Vec<(u64, [u64; 4])>) -> Vec<u64> {
    let mut output = Vec::new();

    for (num, arr) in input {
        output.push(num);
        output.extend(arr.iter().cloned());
    }

    output
}
