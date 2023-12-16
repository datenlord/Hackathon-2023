use reqwest::blocking::Client;
use serde_json::json;
use std::thread;
use std::time::{Instant, Duration};

fn main() {
    // Request online nodes from tracker
    let tracker_url = "http://127.0.0.1:38080/api/v1/info";
    // Default timeout is 30s
    let client = Client::new();

    let response = client
        .get(tracker_url)
        .timeout(Duration::from_secs(1000))
        .header("User-Agent", "Apifox/1.0.0 (https://apifox.com)")
        .header("Content-Type", "application/json")
        .json(&json!({}))
        .send();

    let online_nodes: Vec<String> = match response {
        Ok(res) => {
            let data: serde_json::Value = res.json().unwrap();
            let node_info = data["node_info"].as_array().unwrap();
            node_info
                .iter()
                .map(|node| node[1].as_str().unwrap().to_string())
                .collect()
        }
        Err(err) => {
            println!("Request failed: {:?}", err);
            Vec::new()
        }
    };

    let online_node_list = online_nodes;
    // ["127.0.0.1:48133", "127.0.0.1:48354"]
    
    let urls: Vec<String> = online_node_list.iter().map(|node| format!("http://{}/api/v1/start_download", node)).collect();
    println!("Online node list: {:?}", urls);
    
    let filename = "output_10MB.txt";
    let client = Client::new();
    let start_time = Instant::now(); // Start timer

    println!("Start request download...");
    let threads: Vec<_> = urls
        .into_iter()
        .map(|url| {
            let client = client.clone();
            let filename = filename.to_string();

            thread::spawn(move || {
                let payload = json!({
                    "filename": filename
                });

                let response = client
                    .post(&url)
                    .timeout(Duration::from_secs(1000))
                    .header("User-Agent", "Apifox/1.0.0 (https://apifox.com)")
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send();

                match response {
                    Ok(res) => {
                        println!("Request successful: {:?}", res.text().unwrap());
                    }
                    Err(err) => {
                        println!("Request failed: {:?}", err);
                    }
                }
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }

    let elapsed_time = start_time.elapsed(); // Calculate elapsed time
    println!("Finish request download...Total time taken: {:?}", elapsed_time);
}
