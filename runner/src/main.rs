use crate::utils::get_partner_peer_numbers;
use arcis::{
    ArcisField,
    utils::crypto::{
        key::{X25519PrivateKey, X25519PublicKey},
        rescue_cipher::RescueCipher,
    },
};
use log::info;
use mpc_execution_harness::{
    mpc_execution::{
        Cluster, DeterministicRandom, MPCExecutionBuilder, MPCInputs, MpcInputsConfig, NodeInfo,
        PrivNodeInfo,
    },
    utils::load_circuit_from_path,
};
use primitives::types::PeerNumber;
use rand_chacha::rand_core::SeedableRng;
use rustls::crypto::{CryptoProvider, aws_lc_rs};
use std::sync::Once;
use tokio::runtime::Builder;

mod utils;

static INIT: Once = Once::new();

fn setup() {
    INIT.call_once(|| {
        let _ = pretty_env_logger::try_init();
        // Install the default crypto provider
        CryptoProvider::install_default(aws_lc_rs::default_provider()).unwrap();
    });
}

pub fn main() {
    setup();

    // Create custom runtime - standard tokio::test rt only uses one thread for some reason,
    // which makes this super slow (since we're simulating multiple nodes on one computer)
    let rt = Builder::new_multi_thread()
        .worker_threads(3)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let task1 = tokio::spawn(async {
            run_add_together_circuit(0).await;
        });

        let task2 = tokio::spawn(async {
            run_add_together_circuit(1).await;
        });

        let (t1, t2) = tokio::join!(task1, task2);
        t1.unwrap();
        t2.unwrap();
    });
}

async fn run_add_together_circuit(local_peer_number: PeerNumber) {
    let mut rng = rand_chacha::ChaCha12Rng::from_seed([42; 32]);
    info!("Crypto initialized");

    let circuit = load_circuit_from_path(
        "../build/add_together.arcis".into(),
        // format!(
        //     "{}/build/add_together.arcis",
        //     std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        // )
        // .into(),
    );

    info!("Add together circuit loaded");

    let my_peer_number = local_peer_number;
    let partner_peer_numbers = get_partner_peer_numbers::<1>(my_peer_number);
    let partner_peer_number = partner_peer_numbers[0];
    let node_info = PrivNodeInfo::new_from_const(my_peer_number, 9100);
    let partner_node_info = NodeInfo::new_from_const(partner_peer_number, 9100);
    let mut cluster = Cluster::<1>::new(node_info, [partner_node_info]).await;

    let cluster_pubkey = cluster.gen_key_shares().await.unwrap();

    let client_priv_key = X25519PrivateKey::<bool>::random_det(&mut rng);
    let client_pub_key: X25519PublicKey<ArcisField> =
        X25519PublicKey::get_public_key(client_priv_key);

    let cipher = RescueCipher::new_with_client_from_key_pair(client_priv_key, cluster_pubkey);
    let plaintext = vec![ArcisField::from(1), ArcisField::from(2)];
    let nonce = ArcisField::from(20);

    let ciphertext = cipher.encrypt(plaintext, nonce);

    let base_field_inputs = vec![client_pub_key.inner(), nonce, ciphertext[0], ciphertext[1]];

    let inputs = MPCInputs::from_cluster(
        &cluster,
        base_field_inputs,
        MpcInputsConfig {
            uses_shared_encryption: true,
            uses_mxe_encryption: false,
        },
    );

    let mpc_execution = MPCExecutionBuilder::default()
        .cluster(&mut cluster)
        .inputs(inputs)
        .circuit(circuit)
        .build(124)
        .await;

    // Start the protocol
    let res = mpc_execution.run().await.unwrap();

    let output_nonce = res[1];
    let output_cipher = res[2];

    let plaintext = cipher.decrypt(vec![output_cipher], output_nonce)[0];

    assert_eq!(plaintext, ArcisField::from(3));
    info!("plaintext is {:?}", plaintext);
}
