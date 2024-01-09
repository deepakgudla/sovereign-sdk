use log::info;
// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use risc0::{MOCK_REC_ELF, MOCK_REC_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

#[test]
fn test_rec() {
    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    env_logger::init();

    // An executor environment describes the configurations for the zkVM
    // including program inputs.
    // An default ExecutorEnv can be created like so:
    // `let env = ExecutorEnv::builder().build().unwrap();`
    // However, this `env` does not have any inputs.
    //
    // To add add guest input to the executor environment, use
    // ExecutorEnvBuilder::write().
    // To access this method, you'll need to use ExecutorEnv::builder(), which
    // creates an ExecutorEnvBuilder. When you're done adding input, call
    // ExecutorEnvBuilder::build().

    // For example:
    let input: u32 = 15 * 2 ^ 27 + 2;
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    println!("Proving receipt for input: {}", input);

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove_elf(env, MOCK_REC_ELF).unwrap();

    // TODO: Implement code for retrieving receipt journal here.
    println!("Receipt journal: {:?}", receipt.journal);

    // For example:
    let output: u32 = receipt.journal.decode().unwrap();

    // Optional: Verify receipt to confirm that recipients will also be able to
    // verify your receipt
    receipt.verify(MOCK_REC_ID).unwrap();

    // Print, notice, after committing to a journal, the private input became public
    info!(
        "Hello, world! I generated a proof of guest execution! {} is a public output from journal ",
        output
    );

    assert_eq!(output, 10002);
}