# 测试文件生成

### 生成测试文件

默认生成10MB的文本文件，文件中文本内容为字符`A`。
```rs
let file_size = 10 * 1024 * 1024; // 10MB
let file_path = "output_10MB.txt";
```

在测试环境中运行服务，生成的文件在当前目录下。

```bash
cargo run
```