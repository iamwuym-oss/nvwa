# ADR-IMP-004 — 确定性 Fixture 最小契约 v1.0

**状态：** APPROVED FOR IMP-004 IMPLEMENTATION
**日期：** 2026-07-26
**范围：** GATE-0 / IMP-004

## 1. 决策

IMP-004 首轮只建立跨 Windows/Linux 的普通文件 Fixture。Fixture 是测试基础设施，不是 NWB Archive，不得提前实现或模拟 Writer、Reader、Catalog、Provider、差异备份或 BMR。

Manifest 使用 UTF-8 JSON 与 LF 换行。所有路径为 `/` 分隔的相对路径，并按 UTF-8 字节顺序排列。Manifest 固定记录 Schema版本、Dataset版本、Seed、各文件路径、长度、SHA-256及根SHA-256。

相同工具版本、Dataset版本和Seed必须生成完全相同的文件内容、Manifest字节与根哈希。验证器必须按Seed重建批准数据集预期，拒绝仅靠同时修改文件与Manifest绕过验证。

## 2. 首轮覆盖

- 空文件、1字节、4 KiB和256 KiB边界；
- 全零与重复内容；
- 固定Seed的确定性内容；
- 深目录；
- Unicode文件名。

## 3. 明确排除

稀疏文件、符号链接、ACL、Windows ADS、xattr、非UTF-8路径、设备节点、差异备份、块级数据、BMR和真实NWB Archive均不在本轮范围。

## 4. 安全与失败语义

- 生成目标必须不存在或为空，禁止覆盖现有文件；
- Fixture根、Manifest及数据树不得包含符号链接；
- 拒绝绝对路径、父目录穿越、反斜杠平台路径、重复或乱序条目；
- 拒绝缺失、额外、长度不符或SHA-256不符的文件；
- Schema版本或Dataset版本未知时明确失败。

## 5. 验收

以 `TST-FIX-001`～`TST-FIX-005` 证明确定性、批准数据集覆盖、自校验、篡改检测和路径/目标安全。Windows与Linux必须分别验证，不得互相外推。
