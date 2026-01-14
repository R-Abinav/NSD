mod utils;
mod ip_bst;
mod ip_bin_trie;

use std::time::Instant;
use utils::{ip_to_u32, u32_to_ip};
use ip_bst::BSTNode;
use ip_bin_trie::TrieNode;

fn main() {
    println!("ip lookup\n");

    //base table
    let mut routes = vec![
        ("192.168.0.0", 16, "Router_A"),
        ("192.168.1.0", 24, "Router_B"),
        ("192.168.1.128", 25, "Router_C"),
        ("10.0.0.0", 8, "Router_D"),
        ("172.16.0.0", 12, "Router_E"),
    ];

    let mut generated_routes: Vec<(String, u8, String)> = vec![];
    for i in 0..100 {
        generated_routes.push((
            format!("10.{}.0.0", i),
            16,
            format!("Router_{}", i + 100),
        ));
        generated_routes.push((
            format!("172.{}.0.0", (i % 240) + 16),
            20,
            format!("Router_{}", i + 200),
        ));
        generated_routes.push((
            format!("192.168.{}.0", i % 256),
            24,
            format!("Router_{}", i + 300),
        ));
    }
    
    println!("Total routes: {} (base) + {} (generated) = {}", routes.len(), generated_routes.len(), routes.len() + generated_routes.len());

    let mut bst_root = BSTNode::new(
        ip_to_u32(routes[0].0),
        routes[0].1,
        routes[0].2.to_string(),
    );

    let mut trie_root = TrieNode::new();

    for (i, (prefix, len, hop)) in routes.iter().enumerate() {
        let prefix_ip = ip_to_u32(prefix);
        if i < 5 {
            println!("{}/{} -> {}", prefix, len, hop);
        }

        if i == 0 {
            //first route init bst root
        } else {
            bst_root.insert(prefix_ip, *len, hop.to_string());
        }
        trie_root.insert(prefix_ip, *len, hop.to_string());
    }

    for (prefix, len, hop) in &generated_routes {
        let prefix_ip = ip_to_u32(prefix);
        bst_root.insert(prefix_ip, *len, hop.to_string());
        trie_root.insert(prefix_ip, *len, hop.to_string());
    }
    

    //test lookups
    println!("\nLookup Tests:");

    let test_ips = vec![
        "192.168.1.5",
        "192.168.1.200",
        "10.5.10.1",
        "172.16.5.5",
        "8.8.8.8",
    ];

    for ip_str in &test_ips {
        let ip = ip_to_u32(ip_str);

        //bst
        let mut bst_result = None;
        let mut best_len = -1;
        bst_root.lookup(ip, &mut bst_result, &mut best_len);

        //trie
        let trie_result = trie_root.lookup(ip);

        println!("\nLooking up: {}", ip_str);
        println!("BST  Result: {}", bst_result.unwrap_or("No route".to_string()));
        println!("Trie Result: {}", trie_result.unwrap_or("No route".to_string()));
    }

    println!("\nPerformance test - bst vs binary trie");
    println!("Performing 100,000 lookups on {} routes...\n", routes.len() + generated_routes.len());

    let lookup_ip = ip_to_u32("192.168.1.5");

    //bst
    let start = Instant::now();
    for _ in 0..100_000 {
        let mut result = None;
        let mut best_len = -1;
        bst_root.lookup(lookup_ip, &mut result, &mut best_len);
    }
    let bst_time = start.elapsed();

    //trie
    let start = Instant::now();
    for _ in 0..100_000 {
        let _ = trie_root.lookup(lookup_ip);
    }
    let trie_time = start.elapsed();

    println!("BST  Time: {:.3} ms", bst_time.as_secs_f64() * 1000.0);
    println!("Trie Time: {:.3} ms", trie_time.as_secs_f64() * 1000.0);

    let speedup = bst_time.as_secs_f64() / trie_time.as_secs_f64();
    println!(
        "\nSpeedup: {:.2}x {}",
        speedup,
        if trie_time < bst_time {
            "(Trie is faster)"
        } else {
            "(BST is faster)"
        }
    );
}