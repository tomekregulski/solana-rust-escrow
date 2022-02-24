pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// GO BACK AND LEARN ABOUT SLICE

// Flow of the file structure:
// 1. Someone calls the entrypoint
// 2. The entrypoint forwards the arguments to the processor
// 3. The processor asks instruction.rs to decode the   
//      instruction_data argument from the entrypoint 
//      function.
// 4. Using the decoded data, the processor will now decide 
//      which processing function to use to process the 
//      request.
// 5. The processor may use state.rs to encode state into or 
//      decode the state of an account which has been passed 
//      into the entrypoint.

// General workflow (can vary depending on the data sent to instruction):
// Entrypoint -> Processor -> Instruction (API) -> Processor -> (State) -> Output / Error

// Flow of first half of program:
// 1. create empty account owned by token program
// 2. initialize empty account as INITIALIZER's X token account
// 3. transfer X tokens from INITIALIZER's main X token account to their temporary X token account
// 4. create empty account owned by escrow program
// 5. initialize empty account as escrow state and transfer temporary X token account ownership to PDA

// Instructions may depend on previous instructions inside the same transaction. Transactions are atomic, so if any of the instructions fail, the entire transaction fails.