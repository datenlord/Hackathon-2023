use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

fn main() {
    let file_size = 10 * 1024 * 1024; // 10MB
    let file_path = "output_10MB.txt";

    // 创建文件并打开写入模式
    let file = File::create(file_path).expect("无法创建文件");
    let mut writer = BufWriter::new(file);

    let mut bytes_written = 0;
    let buffer_size = 1024; // 每次写入的缓冲区大小

    // 持续写入数据，直到达到目标文件大小
    while bytes_written < file_size {
        let remaining_bytes = file_size - bytes_written;
        let bytes_to_write = std::cmp::min(buffer_size, remaining_bytes);

        // 创建一个缓冲区并填充为 ASCII 字符 'A'
        let buffer: Vec<u8> = vec![b'A'; bytes_to_write];

        // 将缓冲区写入文件
        writer.write_all(&buffer).expect("写入文件时发生错误");

        bytes_written += bytes_to_write;
    }

    // 刷新缓冲区并写入文件
    writer.flush().expect("刷新缓冲区时发生错误");

    println!("生成的文件大小为 {}MB", file_size / (1024 * 1024));
}