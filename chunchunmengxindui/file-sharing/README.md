## 参赛队伍信息
队伍名：纯纯萌新队
队伍人数：1人

## 文档快速一览
- 项目简介 中介绍了项目的大致业务框架和技术框架
- 项目快速使用 中介绍了项目启动的操作步骤
- 测试报告 中提供了 文件传输准确性和1GB文件传输所需时间的两种测试方案和结论
- 项目不足 中介绍了一些项目上的问题

## 项目简介
使用Rust语言实现一个局域网内的P2P文件传输程序，其中节点发现、内容发现、节点间通信使用Libp2p库作为底层实现。大致的功能框架是：先启用一个引导节点作为后续所有节点需要通信的对象，然后在此引导节点上记录后续所有来访普通节点的节点id与网络地址。之后的普通节点正常初始化、向引导节点通信、然后根据硬编码到其内部的文件请求任务进行文件请求，在文件请求时是通过Kademila算法从p2p网络中去发现那些提供该文件的节点的节点ID，再通过从引导节点获取其已知的DHT路由表来得到那些提供者的网络地址，然后就可以通过网络地址Request到对应节点获取文件。获取文件后，该节点也会作为文件的提供者存在于网络中。

### 项目技术概览
项目中大量使用了异步channel，如果按照管道程序来划分，可以将项目分为client、event_loop、file_mananger三层。层次之间通过管道生产和消费任务。
client是节点的主要业务工具，可以发起监听命令、dialing通信命令、获取指定文件的提供者、向网络请求文件内容、向网络响应文件内容等
event_loop中处理两个任务来源传递过来的任务，其中一个是command channel，负责接受client调用方法所传递过来的任务；另一个是由libp2p提供实现的swarm接受到的其他节点的通信任务，swarm在libp2p中定义是一个节点的通信管理器，负责管理listening和dialing，从而管理多路复用、安全通信和协议协商.
file_mananger则是纯粹业务层实现，通过event channel获取任务，然后进行文件缓存、文件提供、从S3中获取等方法。

## 项目快速使用
### 第一步：启动引导节点
```
cargo run --  --listen-address-with-port /ip4/192.168.3.21/tcp/13000 bootstrap
```
参数解释：
--listen-address-with-port参数是想要监听本机的哪个端口，其中192.168.3.21需要替换成节点对应的局域网IP,端口13000也可以自定义更换
bootstrap参数是表示按照引导节点模式启动

启动后须知：
当引导节点启动后会在控制台打印类似如下的语句：
```
Local node is listening on "/ip4/192.168.3.21/tcp/13000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X"
```
其中/ip4/192.168.3.21/tcp/13000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X就是引导节点的网络地址，在之后启动普通节点中需要作为参数传递进去

### 第二步：启动普通节点
普通节点是作为文件的获取者和提供者而存在的
```
cargo run --  --bootstrap-peer /ip4/192.168.3.21/tcp/13000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.3.21/tcp/14000 common-node 
```
参数解释：
--bootstrap-peer /ip4/192.168.3.21/tcp/13000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X 就是将引导节点的网络地址传递进去
--listen-address-with-port /ip4/192.168.3.21/tcp/14000 同上，就是指定需要监听本机哪个端口，ip地址需要本机对应局域网IP
common-node 表示当前是普通节点

### 提示
可以通过RUST_LOG=WARN 属性改变日志级别，当前默认是WARN级别，通过INFO级别可以看到一些P2P网络之间节点的通信细节

## 测试报告
### 测试总结
通过验证文件传输准确性的方案可以得知传输中文件未发生变化
通过测试1GB文件传输时间的方案可以得知在我的测试环境中：
  同机不同进程情况下从网络中获取1GB文件需要33.271秒
  同局域网下跨机获取1GB文件需要97.660961秒
### 测试环境
测试环境网络拓扑
路由器 192.168.6.1
      - 192.168.6.55 ubuntu机器
      - 192.168.6.182 macos机器

从192.168.6.182PING 192.168.6.55网络情况：  
```
PING 192.168.6.55 (192.168.6.55): 56 data bytes
64 bytes from 192.168.6.55: icmp_seq=0 ttl=64 time=3.102 ms
64 bytes from 192.168.6.55: icmp_seq=1 ttl=64 time=20.781 ms
64 bytes from 192.168.6.55: icmp_seq=2 ttl=64 time=3.711 ms
64 bytes from 192.168.6.55: icmp_seq=3 ttl=64 time=66.506 ms
64 bytes from 192.168.6.55: icmp_seq=4 ttl=64 time=3.932 ms
64 bytes from 192.168.6.55: icmp_seq=5 ttl=64 time=3.575 ms
64 bytes from 192.168.6.55: icmp_seq=6 ttl=64 time=3.874 ms
64 bytes from 192.168.6.55: icmp_seq=7 ttl=64 time=3.609 ms
64 bytes from 192.168.6.55: icmp_seq=8 ttl=64 time=12.228 ms
64 bytes from 192.168.6.55: icmp_seq=9 ttl=64 time=3.777 ms

--- 192.168.6.55 ping statistics ---
10 packets transmitted, 10 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 3.102/12.509/66.506/18.798 ms
```

同时 ubuntu机器开放12000、13000、14000、15000端口的防火墙

### 验证文件传输准确性的方案
通过查看传输的ans_4KB.txt来检查传输的准确性

1.修改代码中main.rs中main()函数下面的test_content_flag，将其改为true

2.在Ubuntu机器的12000端口启动引导节点
RUST_LOG=INFO cargo run --  --listen-address-with-port /ip4/192.168.6.55/tcp/12000 bootstrap
得到控制台打印：
```
2023-12-15T11:55:24.458627Z  INFO libp2p_swarm: local_peer_id=12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X
2023-12-15T11:55:24.459453Z  INFO file_sharing: 引导节点创建成功
Local node is listening on "/ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X"
```
3.在Ubuntu机器的13000端口启动普通节点1
RUST_LOG=INFO cargo run --  --bootstrap-peer /ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.6.55/tcp/13000 common-node 
得到控制台打印：
```
Local node is listening on "/ip4/192.168.6.55/tcp/13000/p2p/12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi"
2023-12-15T11:58:53.185008Z  INFO file_sharing: 文件ans_4KB.txt对应任务：开始获取文件
2023-12-15T11:58:53.185044Z  WARN file_sharing: >>>>>>>>>>>>启动文件:ans_4KB.txt获取，该文件由1个子文件组成
2023-12-15T11:58:53.189360Z  INFO file_sharing::network: swarm中收到如下提供者：{}
2023-12-15T11:58:53.190972Z  INFO file_sharing::network: 收到DHT记录响应
2023-12-15T11:58:53.191052Z  INFO file_sharing::network: 当前获取的引导节点DHT记录：Ok(["/ip4/192.168.6.55/tcp/13000/p2p/12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi"])
2023-12-15T11:58:53.191109Z  INFO file_sharing::network: 该文件有如下提供者:{}
2023-12-15T11:58:53.191127Z  INFO file_sharing: 文件ans_4KB.txt对应任务：当前网络无提供者，从S3中下载
2023-12-15T11:58:53.191161Z  INFO file_sharing: 处理GetFileFromS3,从s3中获取文件：ans_4KB.txt
2023-12-15T11:58:53.191223Z  INFO file_sharing: 文件ans_4KB.txt对应任务：当前网络无提供者，从S3中下载完成
2023-12-15T11:58:53.191241Z  WARN file_sharing: 查看文件内容："DatenLord aims to break cloud barrier by deeply integrating hardware and software to build a unified storage-access mechanism to provide high-performance and secure storage support for applications across clouds."
2023-12-15T11:58:53.191286Z  INFO file_sharing: 处理SetFileCache,将文件:ans_4KB.txt存入文件管理器缓存
2023-12-15T11:58:53.193140Z  INFO file_sharing: 文件ans_4KB.txt对应任务：当前节点开始提供该文件
2023-12-15T11:58:53.193157Z  WARN file_sharing: <<<<<<<<<<<<文件:ans_4KB.txt获取完成，该文件可能由1个子文件组成
```
因为该节点作为网络中的第一个普通节点，在查找文件的时候并没有其他的提供者，所以使用的是通过S3下载（这里为了方便测试直接是读取的本地文件，由此来模拟从S3获取文件内容的Vec<u8>）
通过这一行可以得知ans_4KB.txt的内容:
2023-12-15T11:58:53.191241Z  WARN file_sharing: 查看文件内容："DatenLord aims to break cloud barrier by deeply integrating hardware and software to build a unified storage-access mechanism to provide high-performance and secure storage support for applications across clouds."
注意该节点开始提供该文件了

4.在Ubuntu机器的14000端口启动普通节点2
RUST_LOG=INFO cargo run --  --bootstrap-peer /ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.6.55/tcp/14000 common-node 
得到控制台打印：
```
2023-12-15T12:02:51.916865Z  INFO libp2p_swarm: local_peer_id=12D3KooWEdP9pdSkTrPfBZ82MP24PimDuBCuEAvn3D99z2tV6GvG
Local node is listening on "/ip4/192.168.6.55/tcp/14000/p2p/12D3KooWEdP9pdSkTrPfBZ82MP24PimDuBCuEAvn3D99z2tV6GvG"
2023-12-15T12:02:51.930382Z  INFO file_sharing: 文件ans_4KB.txt对应任务：开始获取文件
2023-12-15T12:02:51.930420Z  WARN file_sharing: >>>>>>>>>>>>启动文件:ans_4KB.txt获取，该文件由1个子文件组成
2023-12-15T12:02:51.933191Z  INFO file_sharing::network: swarm中收到如下提供者：{PeerId("12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi")}
2023-12-15T12:02:51.934680Z  INFO file_sharing::network: 收到DHT记录响应
2023-12-15T12:02:51.934719Z  INFO file_sharing::network: 当前获取的引导节点DHT记录：Ok(["/ip4/192.168.6.55/tcp/13000/p2p/12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi", "/ip4/192.168.6.55/tcp/14000/p2p/12D3KooWEdP9pdSkTrPfBZ82MP24PimDuBCuEAvn3D99z2tV6GvG"])
2023-12-15T12:02:51.934809Z  INFO file_sharing::network: 该文件有如下提供者:{PeerId("12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi")}
2023-12-15T12:02:51.934858Z  INFO file_sharing: 文件ans_4KB.txt对应任务：从网络中的提供者处下载文件
2023-12-15T12:02:51.934945Z  INFO file_sharing::network: 处理文件请求命令:ans_4KB.txt
2023-12-15T12:02:51.952482Z  INFO file_sharing::network: 收到文件内容响应
2023-12-15T12:02:51.952525Z  INFO file_sharing: 文件ans_4KB.txt对应任务：从网络中的提供者处下载文件完成
2023-12-15T12:02:51.952543Z  WARN file_sharing: 查看文件内容："DatenLord aims to break cloud barrier by deeply integrating hardware and software to build a unified storage-access mechanism to provide high-performance and secure storage support for applications across clouds."
2023-12-15T12:02:51.952579Z  INFO file_sharing: 处理SetFileCache,将文件:ans_4KB.txt存入文件管理器缓存
2023-12-15T12:02:51.954555Z  INFO file_sharing: 文件ans_4KB.txt对应任务：当前节点开始提供该文件
2023-12-15T12:02:51.954578Z  WARN file_sharing: <<<<<<<<<<<<文件:ans_4KB.txt获取完成，该文件可能由1个子文件组成
```
此时因为有普通节点1的提供，所以此处可以从网络中获取提供者的网络地址，然后与提供者通信获取到文件内容。
此时文件内容与文件本身内容一致：
2023-12-15T12:02:51.952543Z  WARN file_sharing: 查看文件内容："DatenLord aims to break cloud barrier by deeply integrating hardware and software to build a unified storage-access mechanism to provide high-performance and secure storage support for applications across clouds."

5.在macos机器的15000端口启动普通节点3
需要注意：macos机器的程序也进行了第一步bool值的修改
RUST_LOG=INFO cargo run --  --bootstrap-peer /ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.6.182/tcp/15000 common-node 
得到控制台打印如下：
```
2023-12-15T12:06:05.224883Z  INFO libp2p_swarm: local_peer_id=12D3KooWJMMgejGfVgoCGrdTpZMZSgvqXunuYNqeByuE7GVCsmFS
Local node is listening on "/ip4/192.168.6.182/tcp/15000/p2p/12D3KooWJMMgejGfVgoCGrdTpZMZSgvqXunuYNqeByuE7GVCsmFS"
2023-12-15T12:06:05.244949Z  INFO file_sharing: 文件ans_4KB.txt对应任务：开始获取文件
2023-12-15T12:06:05.245010Z  WARN file_sharing: >>>>>>>>>>>>启动文件:ans_4KB.txt获取，该文件由1个子文件组成
2023-12-15T12:06:05.272841Z  INFO file_sharing::network: swarm中收到如下提供者：{PeerId("12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi"), PeerId("12D3KooWEdP9pdSkTrPfBZ82MP24PimDuBCuEAvn3D99z2tV6GvG")}
2023-12-15T12:06:05.336512Z  INFO file_sharing::network: 收到DHT记录响应
2023-12-15T12:06:05.336560Z  INFO file_sharing::network: 当前获取的引导节点DHT记录：Ok(["/ip4/192.168.6.182/tcp/15000/p2p/12D3KooWMq8GSKqitpCDfCsd2hNx1QK5ZBdSZFNQnKdfQJsWUc5q", "/ip4/192.168.6.55/tcp/13000/p2p/12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi", "/ip4/192.168.6.55/tcp/14000/p2p/12D3KooWEdP9pdSkTrPfBZ82MP24PimDuBCuEAvn3D99z2tV6GvG", "/ip4/192.168.6.182/tcp/15000/p2p/12D3KooWJMMgejGfVgoCGrdTpZMZSgvqXunuYNqeByuE7GVCsmFS"])
2023-12-15T12:06:05.336660Z  INFO file_sharing::network: 该文件有如下提供者:{PeerId("12D3KooWH3hEq8QohGVoF5nqVBrxUL29oWyv4h46JVEdnu5S2RWi"), PeerId("12D3KooWEdP9pdSkTrPfBZ82MP24PimDuBCuEAvn3D99z2tV6GvG")}
2023-12-15T12:06:05.336695Z  INFO file_sharing: 文件ans_4KB.txt对应任务：从网络中的提供者处下载文件
2023-12-15T12:06:05.336771Z  INFO file_sharing::network: 处理文件请求命令:ans_4KB.txt
2023-12-15T12:06:05.336809Z  INFO file_sharing::network: 处理文件请求命令:ans_4KB.txt
2023-12-15T12:06:05.383176Z  INFO file_sharing::network: 收到文件内容响应
2023-12-15T12:06:05.383227Z  INFO file_sharing: 文件ans_4KB.txt对应任务：从网络中的提供者处下载文件完成
2023-12-15T12:06:05.383280Z  WARN file_sharing: 查看文件内容："DatenLord aims to break cloud barrier by deeply integrating hardware and software to build a unified storage-access mechanism to provide high-performance and secure storage support for applications across clouds."
2023-12-15T12:06:05.383342Z  INFO file_sharing: 处理SetFileCache,将文件:ans_4KB.txt存入文件管理器缓存
2023-12-15T12:06:05.401194Z  INFO file_sharing::network: 将12D3KooWMq8GSKqitpCDfCsd2hNx1QK5ZBdSZFNQnKdfQJsWUc5q节点从DHT中去除
2023-12-15T12:06:05.425368Z  INFO file_sharing::network: 收到文件内容响应
2023-12-15T12:06:05.440652Z  INFO file_sharing: 文件ans_4KB.txt对应任务：当前节点开始提供该文件
2023-12-15T12:06:05.440770Z  WARN file_sharing: <<<<<<<<<<<<文件:ans_4KB.txt获取完成，该文件可能由1个子文件组成
```

此时macos已经是跨机器的节点了，而文件传输依旧正确

### 测试1GB文件传输时间的方案
在此方案下，假设一个file block的size是1MB，下载一个1GB文件可以假设成下载1024个1MB的文件组合而成。所以该方案中会把ans_1MB.txt这个文件从网络中下载1024次。为了该测试方案通过，文件管理程序中没有对文件进行缓存中存在与否的检验。

1.修改代码中main.rs中main()函数下面的test_content_flag，将其改为false

2.在Ubuntu机器的12000端口启动引导节点
cargo run --  --listen-address-with-port /ip4/192.168.6.55/tcp/12000 bootstrap
控制台打印：
```
Local node is listening on "/ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X"
```

3.在Ubuntu机器的13000端口启动普通节点1
cargo run --  --bootstrap-peer /ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.6.55/tcp/13000 common-node 
控制台打印：
```
Local node is listening on "/ip4/192.168.6.55/tcp/13000/p2p/12D3KooWJ1ZEzRVJ3wP8vzf2dBeM463g9VTvTn4baXL9K4CN7f78"
2023-12-15T12:16:43.638470Z  WARN file_sharing: >>>>>>>>>>>>启动文件:ans_1GB.txt获取，该文件由1024个子文件组成
2023-12-15T12:17:04.146938Z  WARN file_sharing: <<<<<<<<<<<<文件:ans_1GB.txt获取完成，该文件可能由1024个子文件组成
```
此时的节点1还是从S3(也就是本地文件读取)获取的，在获取文件之后会正常的进行文件提供

4.在Ubuntu机器的14000端口启动普通节点2
cargo run --  --bootstrap-peer /ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.6.55/tcp/14000 common-node 
得到控制台打印：
```
Local node is listening on "/ip4/192.168.6.55/tcp/14000/p2p/12D3KooWHv5hrwkuT8MPn9sLbkVpKfjdJoxMfiDtrZZgcF82gbhq"
2023-12-15T12:18:20.718768Z  WARN file_sharing: >>>>>>>>>>>>启动文件:ans_1GB.txt获取，该文件由1024个子文件组成
2023-12-15T12:18:53.990708Z  WARN file_sharing: <<<<<<<<<<<<文件:ans_1GB.txt获取完成，该文件可能由1024个子文件组成
```
通过日志的时间戳可以得知在同机不同进程情况下从网络中获取1GB文件需要33.271秒

5.在macos机器的15000端口启动普通节点3
需要注意：macos机器的程序也进行了第一步bool值的修改
cargo run --  --bootstrap-peer /ip4/192.168.6.55/tcp/12000/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X --listen-address-with-port /ip4/192.168.6.182/tcp/15000 common-node 
得到控制台打印
```
Local node is listening on "/ip4/192.168.6.182/tcp/15000/p2p/12D3KooWQB6WGXjedJfYN4pk7ZV5W6ywLAusYATC6fcvgT7HmFb8"
2023-12-15T12:21:06.788267Z  WARN file_sharing: >>>>>>>>>>>>启动文件:ans_1GB.txt获取，该文件由1024个子文件组成
2023-12-15T12:22:44.448228Z  WARN file_sharing: <<<<<<<<<<<<文件:ans_1GB.txt获取完成，该文件可能由1024个子文件组成
```
通过时间戳日志可以得知在同局域网下跨机获取1GB文件需要97.660961秒

## 项目不足
受限于时间和RUST开发经验不足的原因，该项目只是完成了基本的功能实现，还有很多设想没有实现
- 性能的自动化测试方案。设想是使用docker compose快速搭建测试环境，进行多节点并发、节点断连等情况的测试
- 测试工具的使用。目前只是使用日志时间戳的方式简单的计算了获取文件的时间，对于管道传输压力、CPU占用情况、内存占用情况等还不知道用什么工具来很好的测试
- 对于项目中必须使用引导节点获取提供者网络地址的优化。因为对Libp2p底层实现原理不太了解，所以对于为什么不能通过Kad获取指定peerID的问题不知道如何解决（事实上这个也是实现中一个巨坑的地方，查阅了很多资料浪费了很多时间），目前是用引导节点获取网络地址的方案来间接实现了
- 代码架构的调整。因为RUST开发经验不足，一些代码模型还有可以优化调整的空间。同时也因为自动化测试框架的缺失，无法知道一些改动对性能的影响
- 功能的优化。功能上存在很多局限，比如引导节点不会在普通节点断连后在DHT中自动删除对应的节点记录、同名文件的处理问题等
- 一些网络方面技术的知识面缺失，比如传输背压怎么解决，数据包传输时怎么保证数据一致性

