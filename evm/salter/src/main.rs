use tiny_keccak::Hasher as _;

fn main() {
    let mut args = std::env::args().skip(1);

    let abi_path = args.next().expect("abi path required");
    let ctor_args = args.next().map(|ctor_hex| {
        let c = &ctor_hex;
        hex::decode(c.strip_prefix("0x").unwrap_or(c)).expect("invalid ctor data")
    });

    let abi: serde_json::Value =
        serde_json::from_slice(&std::fs::read(abi_path).expect("unable to open abi")).unwrap();
    let bytecode = abi
        .as_object()
        .and_then(|abi| abi.get("bytecode"))
        .and_then(|bytecode| bytecode.as_object())
        .and_then(|bytecode| bytecode.get("object"))
        .and_then(|object| object.as_str())
        .and_then(|object| hex::decode(&object[2..]).ok())
        .expect("unparseable abi");

    let deployer = {
        let deployer_bytes = hex::decode("4e59b44847b379578588920ca78fbf26c0b4956c").unwrap();
        let mut deployer = [0u8; 20];
        deployer.copy_from_slice(&deployer_bytes);
        deployer
    };

    let bytecode_hash = {
        let mut bytecode_hash = [0u8; 32];
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(&bytecode);
        if let Some(ctor_args) = ctor_args {
            hasher.update(&ctor_args);
        }
        hasher.finalize(&mut bytecode_hash);
        bytecode_hash
    };

    let (tx, rx) = std::sync::mpsc::channel();

    for _ in 1..num_cpus::get() {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let mut best_weight = 20;
            let mut addr = [0u8; 32];
            let mut salt = [0u8; 32];
            let mut rng = fastrand::Rng::new();
            loop {
                rng.fill(&mut salt);
                calculate_create2_addr(&deployer, &salt, &bytecode_hash, &mut addr);
                let weight = addr.iter().skip(12).filter(|b| **b != 0).count();
                if weight < best_weight {
                    best_weight = weight;
                    tx.send((addr, salt, weight)).unwrap();
                }
            }
        });
    }

    let mut best_weight = 20;
    loop {
        let (addr, salt, weight) = rx.recv().unwrap();
        if weight >= best_weight {
            continue;
        }
        best_weight = weight;
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "address": hex::encode(&addr[12..]),
                "salt": format!("0x{}", hex::encode(salt)),
                "weight": best_weight,
            }))
            .unwrap()
        );
    }
}

fn calculate_create2_addr(
    deployer: &[u8; 20],
    salt: &[u8; 32],
    bytecode_hash: &[u8; 32],
    addr: &mut [u8; 32],
) {
    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(&[0xff]);
    hasher.update(deployer);
    hasher.update(salt);
    hasher.update(bytecode_hash);
    hasher.finalize(addr);
}
