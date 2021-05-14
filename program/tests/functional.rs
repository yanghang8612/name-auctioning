#![cfg(feature = "test-bpf")]
use solana_program::pubkey::Pubkey;
use solana_program_test::{processor, ProgramTest};

#[tokio::test]
async fn test() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "name-auctioning",
        program_id,
        processor!(name_auctioning::entrypoint::process_instruction),
    );
    program_test.add_program(
        "spl-naming-service", 
        spl_name_service::id(), 
        processor!(spl_name_service::processor::Processor::process_instruction)
    );
    program_test.add_program(
        "spl-auction",
        spl_auction::id(),
        processor!(spl_auction::processor::process_instruction)
    );

    let ctx = program_test.start_with_context().await;

    
}
