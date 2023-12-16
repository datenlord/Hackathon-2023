# Baseline

多台设备直接读取同一个文件，测试读取速度。测试机器4核8G，测试文件大小为10MB。

### 测试结果

**测试环境使用tc工具进行限速，单个节点10MB大约需要10S下载完成**
**通过给网络带宽限速访问小文件的形式测试来模拟高带宽，海量存储的情况**

##### **Baseline测试**（baseline）

1. 单个文件使用分片形式直接下载到内存中，耗时为：9.730874292s

```bash
Start multi download test... start_time: Instant { tv_sec: 1195191, tv_nsec: 100834166 }
Start download test 0
meta: HeadObjectResult { accept_ranges: Some("bytes"), cache_control: None, content_disposition: None, content_encoding: None, content_language: None, content_length: Some(10485760), content_type: Some("text/plain"), delete_marker: None, e_tag: Some("\"8b8378787c0925f42ccb829f6cc2fb97\""), expiration: None, expires: None, last_modified: Some("Tue, 12 Dec 2023 08:05:17 GMT"), metadata: Some({}), missing_meta: None, object_lock_legal_hold_status: None, object_lock_mode: None, object_lock_retain_until_date: None, parts_count: None, replication_status: None, request_charged: None, restore: None, sse_customer_algorithm: None, sse_customer_key_md5: None, ssekms_key_id: None, server_side_encryption: None, storage_class: None, version_id: None, website_redirect_location: None }
End download test 0
End multi download test... end_time: Instant { tv_sec: 1195200, tv_nsec: 831708458 }
Task-0 multi download test duration: 9.730874292s
Average multi download test duration: 973
```

2. 目前是使用多线程的方式并行进行下载，下载时间为:
（10个线程同时读取这个10MB的文件，测试10次取平均值）：
```bash
...
Average multi download test duration: 117.793s
```

#### **P2P测试**（p2p-with-tracker）

>（10MB下载）,测试S3服务器使用Minio S3， 带宽约为1MB/s，可以使用tc工具进行限速
> 
> 配置文件可修改，默认分片大小为128KB

修改config.toml，启动tracker服务，默认端口38080：
```bash
cargo run
```

修改config.toml，指定启动模式为node，启动node服务，默认端口是在48000-49000中的随机端口，可以在启动日志中查看：
```bash
cargo run -- -m node

# INFO p2p_with_tracker::server: 🚀🚀🚀 Server [node:ff0e66a6-6153-4926-842a-3a5f5d360d66] has launched on http://127.0.0.1:48133
```

修改test.sh文件，指定node服务的端口，以及下载的文件名，启动下载测试：
```bash
time curl --location --request POST 'http://127.0.0.1:48412/api/v1/start_download' \
--header 'User-Agent: Apifox/1.0.0 (https://apifox.com)' \
--header 'Content-Type: application/json' \
--data-raw '{
    "filename": "output_10MB.txt"
}'
```
> time命令可以测试命令的执行时间。


1. 使用p2p的方式进行下载，1个node节点同时读取这个10MB的文件，下载时间为，比直接下载略长（**主要是额外的通信的耗时**）：
```bash
cpu 12.450 total
```

2. 尝试使用p2p方式进行下载，10个node节点同时读取这个10MB的文件，下载时间为（**后续节点能够选择从高速的相邻节点进行下载**）：
```bash
cpu 37.540 total
```

因此，在10个节点的情况下，读取该文件的时间提升效果非常明显。

3. 