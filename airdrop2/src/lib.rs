#[cfg(test)]

mod tests {
    use solana_sdk::signature::{Keypair, Signer};
    #[test]
fn keygen() {
    // Create a new keypair
    let kp = Keypair::new();
    println!("You've generated a new Solana wallet: {}\n", kp.pubkey());
    println!("To save your wallet, copy and paste the following into a JSON file:");
    println!("{:?}", kp.to_bytes());
    }
    #[test]
    fn claim_airdrop() {}
    #[test]
    fn transfer_sol() {}
}