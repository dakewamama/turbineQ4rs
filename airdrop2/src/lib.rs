#[cfg(test)]

mod tests {
    use bs58;
    use std::io::{self, BufRead};
    use solana_client::rpc_client::RpcClient;
    const RPC_URL: &str = "https://api.devnet.solana.com";
    use solana_system_interface::instruction::transfer;
    use solana_system_interface::program as system_program;
    use solana_sdk::{
        hash::hash,
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer, read_keypair_file},
        transaction::Transaction,
        instruction::{Instruction, AccountMeta},
    };
    use std::str::FromStr;
    #[test]
    fn keygen() {
    // Create a new keypair
    let kp = Keypair::new();
    println!("You've generated a new Solana wallet: {}\n", kp.pubkey());
    println!("To save your wallet, copy and paste the following into a JSON file:");
    println!("{:?}", kp.to_bytes());
    }
    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as a base58 string:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file format is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }
    #[test]
    fn wallet_to_base58() {
    println!("Input your private key as a JSON byte array (e.g. [12,34,...]):");
    let stdin = io::stdin();
    let wallet = stdin
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().parse::<u8>().unwrap())
        .collect::<Vec<u8>>();
    println!("Your Base58-encoded private key is:");
    let base58 = bs58::encode(wallet).into_string();
    println!("{:?}", base58);
}
    #[test]
    fn claim_airdrop() {
        // Import our keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        
        // connection to Solana devnet
        let client = RpcClient::new(RPC_URL);
        
        // Claim
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(sig) => {
                println!("Success! Check your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", sig);
            }
            Err(err) => {
                println!("Airdrop failed: {}", err);
            }
        }
    }
    #[test]
    fn transfer_sol() {
        // Load your devnet keypair
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        
        println!("Loaded keypair with pubkey: {}", keypair.pubkey());
        
        
        let to_pubkey = Pubkey::from_str("29DypSfJxtzvKid7MYF9VcPnCPgg7jRtQf9MU3ukLDew").unwrap();
        
        // Connect to devnet
        let rpc_client = RpcClient::new(RPC_URL);
        
        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        // let transaction = Transaction::new_signed_with_payer(
        //     &[transfer(&keypair.pubkey(), &to_pubkey, 100_000_000)],
        //     Some(&keypair.pubkey()),
        //     &vec![&keypair],
        //     recent_blockhash,
        // );
        
        
        // Get current balance
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");
        
        println!("Current balance: {} lamports ({} SOL)", balance, balance as f64 / 1_000_000_000.0);
        
        
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );
        
        // Get transaction fee
        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee");
        
        println!("Transaction fee: {} lamports", fee);
        println!("Transferring: {} lamports ({} SOL)", balance - fee, (balance - fee) as f64 / 1_000_000_000.0);
        
        // Create final transaction with balance minus fee
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );
        
        // Send transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
    #[test]
    fn submit_completion() {
        // Load your Turbin3 keypair
        let signer = read_keypair_file("turbin3-wallet.json").expect("Couldn't find wallet file");
        
        // Create RPC client
        let rpc_client = RpcClient::new(RPC_URL);
        
        // Define program and account public keys
        let mint = Keypair::new();
        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();
        let mpl_core_program = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();
        
        // Get the PDA 
        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);
        
        println!("PDA: {}", prereq_pda);
        
        // Get the authority from the collection account
        let collection_account = rpc_client
            .get_account_data(&collection)
            .expect("Failed to get collection account");
        
        // The update authority is stored in bytes 1-33 (skip 1-byte discriminator)
        let authority_bytes = &collection_account[1..33];
        let authority = Pubkey::try_from(authority_bytes).expect("Invalid authority pubkey");
        
        println!("Authority: {}", authority);
        
        // Prepare the instruction data (discriminator for submit_rs)
        let data = vec![77, 124, 82, 163, 21, 133, 181, 206];
        
        // Define the accounts metadata
        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),           
            AccountMeta::new(prereq_pda, false),               
            AccountMeta::new(mint.pubkey(), true),             
            AccountMeta::new(collection, false),               
            AccountMeta::new_readonly(authority, false),       
            AccountMeta::new_readonly(mpl_core_program, false), 
            AccountMeta::new_readonly(system_program, false),  
        ];
        
        // Get the recent blockhash
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");
        
        // Build the instruction
        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };
        
        // Create and sign the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&signer.pubkey()),
            &[&signer, &mint],
            blockhash,
        );
        
        // Send and confirm the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");
        
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

}