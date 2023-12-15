#![allow(unused)]

use s3::creds::Credentials;
use s3::region::Region;
use s3::error::S3Error;
use s3::{bucket::Bucket, BucketConfiguration};
use chrono::Utc;
use std::time::Instant;

async fn test_download(id:usize) -> Result<(), S3Error> {
    println!("Start download test {:?}", id);

    let bucket_name = "hackathon";
    let region = Region::Custom {
        region: "".to_owned(),
        endpoint: "http://127.0.0.1:39000".to_owned(),
    };
    let credentials = Credentials::new(
        Some("hackathon".to_owned().as_str()),
        Some("hackathon".to_owned().as_str()),
        None,
        None,
        None,
    )?;

    let mut bucket =
    Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

    if !bucket.exists().await? {
        bucket = Bucket::create_with_path_style(
            bucket_name,
            region,
            credentials,
            BucketConfiguration::default(),
        )
        .await?
        .bucket;
    }

    let filename = "output_10MB.txt";
    // let start_time = Instant::now();
    // Get meta data
    let (meta, code) = bucket.head_object(filename).await?;
    println!("meta: {:?}", meta);

    // let res = bucket.get_object(filename).await?;
    // println!("get object: {:?}", res);   

    // Get object range
    let file_size = meta.content_length.unwrap();
    let step = 128 * 1024; // 128KB
    let mut start = 0;
    let mut end = step;
    // let buf = Vec::new();
    while start < file_size {
        if end > file_size {
            end = file_size;
        }
        let res = bucket.get_object_range(filename, start.try_into().unwrap(), Some(end.try_into().unwrap())).await.unwrap();
        println!("get object range: {:?}", res);
        // buf.extend_from_slice(&res);
        start += step;
        end += step;
    }

    // 校验下载的文件大小
    // let end_time = Instant::now();
    // let duration = end_time - start_time;
    println!("End download test {:?}", id);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), S3Error>{
    let mut count = 0;

    for j in 0..1 {
        let start_time = Instant::now();

        println!("Start multi download test... start_time: {:?}", start_time);
        let mut tasks = Vec::new();
        for i in 0..1 {
            tasks.push(tokio::spawn(test_download(i)));
        }
        for task in tasks {
            task.await;
        }

        let end_time = Instant::now();
        let duration = end_time - start_time;
        println!("End multi download test... end_time: {:?}", end_time);
        println!("Task-{:?} multi download test duration: {:?}", j, duration);

        count += duration.as_millis();
    }

    println!("Average multi download test duration: {:?}", count/10);

    Ok(())
}
