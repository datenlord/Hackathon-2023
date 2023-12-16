# Generate Test File

### Generate Test File

By default, a 10MB text file is generated with the content of the character `A`.

```rs
let file_size = 10 * 1024 * 1024; // 10MB
let file_path = "output_10MB.txt";
```

Run the service in the test environment, and the generated file will be in the current directory.

```bash
cargo run
```