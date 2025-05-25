#![allow(unexpected_cfgs)]

use borsh::BorshDeserialize;
use jaguar::{JaguarDeserialize, JaguarDeserializer, JaguarSerializer};
use pinocchio::syscalls::sol_remaining_compute_units;
use pinocchio::{default_panic_handler, program_entrypoint, ProgramResult};
use pinocchio::default_allocator;
use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
};
use pinocchio_log::log;

program_entrypoint!(process_instruction);
default_allocator!();
default_panic_handler!();

#[derive(JaguarDeserialize, Debug, Clone, PartialEq)]
struct SampleData { // TOTAL: 32 + 32 + 1 + 20 + 2 + 4 + 8 + 16 = 103
    authority: [u8; 32], // 32
    admin: [u8; 32], // 32
    bump: u8, // 1
    memo: String, // 20
    flags: (u16, u32), // 2 + 4
    offset: u64, // 8
    amount: u128, // 16
}

#[derive(BorshDeserialize, Debug, Clone, PartialEq)]
struct OtherData { // TOTAL: 32 + 32 + 1 + 20 + 2 + 4 + 8 + 16 = 103
    authority: [u8; 32], // 32
    admin: [u8; 32], // 32
    bump: u8, // 1
    memo: String, // 20
    flags: (u16, u32), // 2 + 4
    offset: u64, // 8
    amount: u128, // 16
}

fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    unsafe { sol_remaining_compute_units() };

    log!("[INSTRUCTION_DATA]: data={}", _data);
    let _lib = _data.first().unwrap();
    log!("[INSTRUCTION_DISCRIM]: discrim={}", *_lib);

    let data_clone = _data;

    match *_lib {
        0 => {
            compute_fn!("JAGUAR_DESERIALIZE" => {
                let mut de = JaguarDeserializer::new(&data_clone[1..]);
                let _ = SampleData::deserialize(&mut de).unwrap();
            });
        }
        1 => {
            compute_fn!("BORSH_DESERIALIZE" => {
                let _ = OtherData::try_from_slice(&mut &data_clone[1..]).unwrap();
            });
        }
        2 => {
            let c = data_clone;
            compute_fn!("JAGUAR_DESERIALIZE" => {
                let mut de = JaguarDeserializer::new(&data_clone[1..]);
                let _ = SampleData::deserialize(&mut de).unwrap();
            });
    
            compute_fn!("BORSH_DESERIALIZE" => {
                let _ = OtherData::try_from_slice(&mut &c[1..]).unwrap();
            });
        }
        _ => {
            compute_fn!("JAGUAR_DESERIALIZE" => {
                let mut de = JaguarDeserializer::new(&_data);
                let _ = de.read_u32_vec().unwrap();
            });
            compute_fn!("JAGUAR_SERIALIZE" => {
                let mut ser = JaguarSerializer::new();
                ser.write_u32_slice(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
                let _serialized = ser.finish();
            });
        }
    }

    Ok(())
}

#[macro_export]
macro_rules! compute_fn {
  (
    $msg:expr => $($tt:tt)*
  ) => {
        ::pinocchio::msg!(concat!($msg, " {"));
        ::pinocchio::log::sol_log_compute_units();
        let _res = { $($tt)* };
        ::pinocchio::log::sol_log_compute_units();
        ::pinocchio::msg!(concat!(" } // ", $msg));
  };
}

#[cfg(test)]
mod tests {
    use jaguar::{JaguarDeserialize, JaguarSerialize, JaguarSerializer};
    use solana_program_test::ProgramTest;
    use solana_sdk::{
        account::AccountSharedData,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    };

    #[derive(JaguarSerialize, JaguarDeserialize, Debug, Clone, PartialEq)]
    struct JaguarSampleData {
        authority: [u8; 32],
        admin: [u8; 32],
        bump: u8,
        memo: String,
        flags: (u16, u32),
        offset: u64,
        amount: u128,
    }

    #[tokio::test]
    async fn test_jaguar_cu_usage() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::default();

        program_test.prefer_bpf(true);
        program_test.add_program("jaguar_cu", program_id, None);
        let mut ctx = program_test.start_with_context().await;

        let payer = Keypair::new();
        ctx.set_account(
            &payer.pubkey(),
            &AccountSharedData::new(1_000_000_000, 0, &system_program::ID),
        );

        let data = JaguarSampleData {
            authority: payer.pubkey().to_bytes(),
            admin: payer.pubkey().to_bytes(),
            bump: 254,
            memo: "this is a test".to_string(),
            flags: (128, 25600),
            offset: 392_000,
            amount: 100_000_000_000_000,
        };

        let mut ser = JaguarSerializer::new();
        data.serialize(&mut ser).unwrap();
        let data = ser.finish();

        let discrim = vec![0 as u8];
        let ix_data = vec![discrim, data].concat();

        let ix_data_len = ix_data.len();
        println!("ix_data_len: {}", ix_data_len);

        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data: ix_data,
        };

        let blockhash = ctx.get_new_latest_blockhash().await.unwrap();
        let invoke_transaction =
            Transaction::new_signed_with_payer(&[instruction], Some(&payer.pubkey()), &[&payer], blockhash);

        let result = ctx.banks_client
            .process_transaction_with_metadata(invoke_transaction)
            .await
            .unwrap();

        let cu_usage = result.metadata.clone().unwrap().compute_units_consumed;
        println!("cu_usage: {}", cu_usage);
        let logs = result.metadata.unwrap().log_messages;
        println!("logs: {:?}", logs);
    }
}

#[cfg(test)]
mod borsh_tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program_test::ProgramTest;
    use solana_sdk::{
        account::AccountSharedData,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    };

    #[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
    struct BorshSampleData {
        authority: [u8; 32],
        admin: [u8; 32],
        bump: u8,
        memo: String,
        flags: (u16, u32),
        offset: u64,
        amount: u128,
    }

    #[tokio::test]
    async fn test_borsh_cu_usage() {
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::default();
        program_test.prefer_bpf(true);
        program_test.add_program("jaguar_cu", program_id, None);
        let mut ctx = program_test.start_with_context().await;

        let payer = Keypair::new();
        ctx.set_account(
            &payer.pubkey(),
            &AccountSharedData::new(1_000_000_000, 0, &system_program::ID),
        );

        let data = BorshSampleData {
            authority: payer.pubkey().to_bytes(),
            admin: payer.pubkey().to_bytes(),
            bump: 254,
            memo: "this is a test".to_string(),
            flags: (128, 25600),
            offset: 392_000,
            amount: 100_000_000_000_000,
        };

        let mut ser = Vec::new();
        BorshSerialize::serialize(&mut &data, &mut ser).unwrap();

        let discrim = vec![1 as u8];
        let ix_data = vec![discrim, ser].concat();

        let ix_data_len = ix_data.len();
        println!("ix_data_len: {}", ix_data_len);

        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data: ix_data,
        };

        let blockhash = ctx.get_new_latest_blockhash().await.unwrap();
        let invoke_transaction =
            Transaction::new_signed_with_payer(&[instruction], Some(&payer.pubkey()), &[&payer], blockhash);

        let result = ctx.banks_client
            .process_transaction_with_metadata(invoke_transaction)
            .await
            .unwrap();

        let cu_usage = result.metadata.clone().unwrap().compute_units_consumed;
        println!("cu_usage: {}", cu_usage);
        let logs = result.metadata.unwrap().log_messages;
        println!("logs: {:?}", logs);
    }
}