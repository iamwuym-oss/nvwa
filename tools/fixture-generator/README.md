# NWB Fixture Generator

该工具为后续 NWB 工程测试生成跨平台、可重复且可自校验的普通文件数据集。它不生成 `.nwb` 归档，也不实现 Writer、Reader、Provider 或真实备份/恢复。

## 固定契约

- Manifest：UTF-8 JSON、LF 换行、字段顺序由结构固定；
- 路径：相对路径、`/` 分隔、按 UTF-8 字节排序；
- 数据集版本：`1`；
- 默认 Seed：`0x4e57422d46495831`；
- 默认数据集根SHA-256：`8ce810bd57473b4af044855996291c7ba34860606b6f6ae9a3e98aa774bc7222`；
- 内容摘要：SHA-256；
- 根摘要：域分隔字符串、路径长度、路径字节、文件长度和内容 SHA-256 的确定性组合；
- 禁止符号链接、路径穿越、额外文件和非空生成目标。

## 使用

```text
cargo run --locked -p fixture-generator -- generate <empty-directory>
cargo run --locked -p fixture-generator -- verify <fixture-directory>
```

相同工具版本、数据集版本和 Seed 在 Windows 与 Linux 上必须产生完全相同的 Manifest 字节和 `root_sha256`。

## 首轮数据集范围

包含空文件、1 字节、4 KiB、256 KiB、全零、重复内容、固定 Seed 内容、深目录与 Unicode 文件名。

不包含稀疏文件、符号链接、ACL、ADS、xattr、非 UTF-8 路径、块设备、差异备份、BMR 或真实 NWB Archive。
