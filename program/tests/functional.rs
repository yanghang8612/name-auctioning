use std::str::FromStr;

use name_auctioning::{
    instructions::{create, init, resell, reset_auction},
    processor::{AUCTION_PROGRAM_ID, BONFIDA_VAULT, TOKEN_MINT},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, hash::hashv, instruction::Instruction,
    msg, program_error::PrintProgramError, program_option::COption, program_pack::Pack,
    pubkey::Pubkey,
};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
};
use spl_auction::PREFIX;
use spl_name_service::{
    instruction::NameRegistryInstruction,
    state::{get_seeds_and_key, NameRecordHeader, HASH_PREFIX},
};
use spl_token::state::Mint;

#[tokio::test]
async fn test() {
    let program_id = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let bonfida_vault_owner = Keypair::new();
    let mut program_test = ProgramTest::new(
        "name_auctioning",
        program_id,
        // processor!(name_auctioning::entrypoint::process_instruction),
        None,
    );
    let auction_program_id = Pubkey::from_str(AUCTION_PROGRAM_ID).unwrap();
    program_test.add_program(
        "spl_name_service",
        spl_name_service::id(),
        processor!(spl_name_service::processor::Processor::process_instruction),
    );
    program_test.add_program(
        "spl_auction",
        auction_program_id,
        // processor!(spl_auction::processor::process_instruction),
        processor!(spl_auction::processor::process_instruction),
    );

    let mut mint_data = vec![0u8; Mint::LEN];
    Mint {
        mint_authority: COption::Some(mint_authority.pubkey()),
        supply: 1_000_000_000,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    }
    .pack_into_slice(&mut mint_data);

    program_test.add_account(
        Pubkey::from_str(TOKEN_MINT).unwrap(),
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: spl_token::id(),
            executable: false,
            ..Account::default()
        },
    );

    let mut vault_data = vec![0u8; spl_token::state::Account::LEN];
    spl_token::state::Account {
        mint: Pubkey::from_str(TOKEN_MINT).unwrap(),
        owner: bonfida_vault_owner.pubkey(),
        amount: 0,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut vault_data);

    program_test.add_account(
        Pubkey::from_str(BONFIDA_VAULT).unwrap(),
        Account {
            lamports: 1_000_000,
            data: vault_data,
            owner: spl_token::id(),
            executable: false,
            ..Account::default()
        },
    );

    let (derived_central_state_key, state_nonce) =
        Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);

    // program_test.add_account(
    //     derived_state_key,
    //     Account {
    //         lamports: 1_000_000,
    //         data: vec![state_nonce],
    //         owner: program_id,
    //         executable: false,
    //         ..Account::default()
    //     }
    // );

    let mut ctx = program_test.start_with_context().await;

    let init_instruction = init(
        program_id,
        derived_central_state_key,
        ctx.payer.pubkey(),
        state_nonce,
    );

    sign_send_instruction(&mut ctx, init_instruction, vec![])
        .await
        .unwrap();

    let root_name = ".sol";

    let hashed_root_name: Vec<u8> = hashv(&[(HASH_PREFIX.to_owned() + root_name).as_bytes()])
        .0
        .to_vec();
    let (root_name_account_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed_root_name.clone(),
        None,
        None,
    );

    let create_name_instruction = spl_name_service::instruction::create(
        spl_name_service::id(),
        NameRegistryInstruction::Create {
            hashed_name: hashed_root_name,
            lamports: 1_000_000,
            space: 1_000,
        },
        root_name_account_key,
        ctx.payer.pubkey(),
        derived_central_state_key,
        None,
        None,
        None,
    )
    .unwrap();
    sign_send_instruction(&mut ctx, create_name_instruction, vec![])
        .await
        .unwrap();

    let name_record_header = NameRecordHeader::unpack_from_slice(
        &ctx.banks_client
            .get_account(root_name_account_key)
            .await
            .unwrap()
            .unwrap()
            .data,
    )
    .unwrap();
    println!("Name Record Header: {:?}", name_record_header);

    let test_name = "test";

    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + test_name).as_bytes()])
        .0
        .to_vec();

    println!("Hashed name length {:?}", hashed_name.len());

    let (name_account, key) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed_name.clone(),
        None,
        Some(&root_name_account_key),
    );

    let auction_seeds = &[
        PREFIX.as_bytes(),
        &auction_program_id.to_bytes(),
        name_account.as_ref(),
    ];
    let (auction_account, _) = Pubkey::find_program_address(auction_seeds, &&auction_program_id);

    // let rent = ctx.banks_client.get_rent().await.unwrap();

    // let allocate_auction_account_instruction = system_instruction::create_account(
    //     &ctx.payer.pubkey(),
    //     &auction_account.pubkey(),
    //     rent.minimum_balance(BASE_AUCTION_DATA_SIZE),
    //     BASE_AUCTION_DATA_SIZE as u64,
    //     &&auction_program_id,
    // );

    // sign_send_instruction(
    //     &mut ctx,
    //     allocate_auction_account_instruction,
    //     vec![&auction_account],
    // )
    // .await
    // .unwrap();

    println!("{:?}", key.len());
    let (derived_state_key, _) =
        Pubkey::find_program_address(&[&name_account.to_bytes()], &program_id);

    println!("Program Id: {:?}", program_id);
    println!("Root Name Account: {:?}", root_name_account_key);
    println!("Name Account: {:?}", name_account);
    println!("Auction Account: {:?}", auction_account);
    println!("State Account: {:?}", derived_state_key);
    println!("Payer account: {:?}", ctx.payer.pubkey());
    println!("Quote mint: {:?}", TOKEN_MINT);

    let hashed_reverse_lookup =
        hashv(&[(HASH_PREFIX.to_owned() + &name_account.to_string()).as_bytes()])
            .0
            .to_vec();

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed_reverse_lookup.clone(),
        Some(&derived_central_state_key),
        None,
    );

    let create_naming_auction_instruction = create(
        program_id,
        auction_program_id,
        root_name_account_key,
        name_account,
        reverse_lookup_account_key,
        auction_account,
        derived_central_state_key,
        derived_state_key,
        ctx.payer.pubkey(),
        Pubkey::from_str(TOKEN_MINT).unwrap(),
        test_name.to_owned(),
    );

    sign_send_instruction(&mut ctx, create_naming_auction_instruction, vec![])
        .await
        .unwrap();

    let reset_auction_instruction = reset_auction(
        program_id,
        auction_program_id,
        ctx.payer.pubkey(),
        auction_account,
        name_account,
        derived_state_key,
    );
    sign_send_instruction(&mut ctx, reset_auction_instruction, vec![])
        .await
        .unwrap();
}

// #[tokio::test]
// async fn test_resell() {
//     let program_id = Pubkey::new_unique();
//     let mint_authority = Keypair::new();
//     let bonfida_vault_owner = Keypair::new();
//     let mut program_test = ProgramTest::new(
//         "name_auctioning",
//         program_id,
//         // processor!(name_auctioning::entrypoint::process_instruction),
//         None,
//     );
//     let auction_program_id = Pubkey::from_str(AUCTION_PROGRAM_ID).unwrap();
//     program_test.add_program("spl_name_service", spl_name_service::id(), None);
//     program_test.add_program(
//         "spl_auction",
//         auction_program_id,
//         // processor!(spl_auction::processor::process_instruction),
//         None,
//     );

//     let mut mint_data = vec![0u8; Mint::LEN];
//     Mint {
//         mint_authority: COption::Some(mint_authority.pubkey()),
//         supply: 1_000_000_000,
//         decimals: 6,
//         is_initialized: true,
//         freeze_authority: COption::None,
//     }
//     .pack_into_slice(&mut mint_data);

//     program_test.add_account(
//         Pubkey::from_str(TOKEN_MINT).unwrap(),
//         Account {
//             lamports: 1_000_000,
//             data: mint_data,
//             owner: spl_token::id(),
//             executable: false,
//             ..Account::default()
//         },
//     );

//     let mut vault_data = vec![0u8; spl_token::state::Account::LEN];
//     spl_token::state::Account {
//         mint: Pubkey::from_str(TOKEN_MINT).unwrap(),
//         owner: bonfida_vault_owner.pubkey(),
//         amount: 0,
//         delegate: COption::None,
//         state: spl_token::state::AccountState::Initialized,
//         is_native: COption::None,
//         delegated_amount: 0,
//         close_authority: COption::None,
//     }
//     .pack_into_slice(&mut vault_data);

//     program_test.add_account(
//         Pubkey::from_str(BONFIDA_VAULT).unwrap(),
//         Account {
//             lamports: 1_000_000,
//             data: vault_data,
//             owner: spl_token::id(),
//             executable: false,
//             ..Account::default()
//         },
//     );

//     let (derived_central_state_key, state_nonce) =
//         Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);

//     // program_test.add_account(
//     //     derived_state_key,
//     //     Account {
//     //         lamports: 1_000_000,
//     //         data: vec![state_nonce],
//     //         owner: program_id,
//     //         executable: false,
//     //         ..Account::default()
//     //     }
//     // );

//     let mut ctx = program_test.start_with_context().await;

//     let init_instruction = init(
//         program_id,
//         derived_central_state_key,
//         ctx.payer.pubkey(),
//         state_nonce,
//     );

//     sign_send_instruction(&mut ctx, init_instruction, vec![])
//         .await
//         .unwrap();

//     let root_name = ".sol";

//     let hashed_root_name: Vec<u8> = hashv(&[(HASH_PREFIX.to_owned() + root_name).as_bytes()])
//         .0
//         .to_vec();
//     let (root_name_account_key, _) = get_seeds_and_key(
//         &spl_name_service::id(),
//         hashed_root_name.clone(),
//         None,
//         None,
//     );

//     let create_name_instruction = spl_name_service::instruction::create(
//         spl_name_service::id(),
//         NameRegistryInstruction::Create {
//             hashed_name: hashed_root_name,
//             lamports: 1_000_000,
//             space: 1_000,
//         },
//         root_name_account_key,
//         ctx.payer.pubkey(),
//         derived_central_state_key,
//         None,
//         None,
//         None,
//     )
//     .unwrap();
//     sign_send_instruction(&mut ctx, create_name_instruction, vec![])
//         .await
//         .unwrap();

//     let name_record_header = NameRecordHeader::unpack_from_slice(
//         &ctx.banks_client
//             .get_account(root_name_account_key)
//             .await
//             .unwrap()
//             .unwrap()
//             .data,
//     )
//     .unwrap();
//     println!("Name Record Header: {:?}", name_record_header);

//     let test_name = "test";

//     let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + test_name).as_bytes()])
//         .0
//         .to_vec();

//     println!("Hashed name length {:?}", hashed_name.len());

//     let (name_account, key) = get_seeds_and_key(
//         &spl_name_service::id(),
//         hashed_name.clone(),
//         None,
//         Some(&root_name_account_key),
//     );

//     let auction_seeds = &[
//         PREFIX.as_bytes(),
//         &auction_program_id.to_bytes(),
//         name_account.as_ref(),
//     ];
//     let (auction_account, _) = Pubkey::find_program_address(auction_seeds, &&auction_program_id);

//     println!("{:?}", key.len());
//     let (derived_state_key, _) =
//         Pubkey::find_program_address(&[&name_account.to_bytes()], &program_id);

//     println!("Program Id: {:?}", program_id);
//     println!("Root Name Account: {:?}", root_name_account_key);
//     println!("Name Account: {:?}", name_account);
//     println!("Auction Account: {:?}", auction_account);
//     println!("State Account: {:?}", derived_state_key);
//     println!("Payer account: {:?}", ctx.payer.pubkey());
//     println!("Quote mint: {:?}", TOKEN_MINT);

//     let hashed_reverse_lookup =
//         hashv(&[(HASH_PREFIX.to_owned() + &name_account.to_string()).as_bytes()])
//             .0
//             .to_vec();

//     let name = "megosiani";
//     let hashed_name: Vec<u8> = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
//         .0
//         .to_vec();
//     let (name_account_key, _) =
//         get_seeds_and_key(&spl_name_service::id(), hashed_name.clone(), None, None);
//     let create_name_instruction = spl_name_service::instruction::create(
//         spl_name_service::id(),
//         NameRegistryInstruction::Create {
//             hashed_name: hashed_name,
//             lamports: 1_000_000,
//             space: 1_000,
//         },
//         name_account_key,
//         ctx.payer.pubkey(),
//         ctx.payer.pubkey(),
//         None,
//         Some(root_name_account_key),
//         None,
//     )
//     .unwrap();
//     sign_send_instruction(&mut ctx, create_name_instruction, vec![])
//         .await
//         .unwrap();

//     let resell_naming_auction_instruction = resell(
//         program_id,
//         auction_program_id,
//         root_name_account_key,
//         name_account,
//         reverse_lookup_account_key,
//         auction_account,
//         derived_central_state_key,
//         derived_state_key,
//         ctx.payer.pubkey(),
//         Pubkey::from_str(TOKEN_MINT).unwrap(),
//         test_name.to_owned(),
//     );

//     sign_send_instruction(&mut ctx, resell_naming_auction_instruction, vec![])
//         .await
//         .unwrap();
// }

// Utils
pub async fn sign_send_instruction(
    ctx: &mut ProgramTestContext,
    instruction: Instruction,
    signers: Vec<&Keypair>,
) -> Result<(), TransportError> {
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&ctx.payer.pubkey()));
    let mut payer_signers = vec![&ctx.payer];
    for s in signers {
        payer_signers.push(s);
    }
    transaction.partial_sign(&payer_signers, ctx.last_blockhash);
    ctx.banks_client.process_transaction(transaction).await
}
