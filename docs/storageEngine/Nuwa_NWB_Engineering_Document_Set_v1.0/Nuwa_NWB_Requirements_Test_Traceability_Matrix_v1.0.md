# Nüwa NWB 需求-测试追溯矩阵 v1.0

> 本文档由 `traceability-checker` 从 `Nuwa_NWB_Traceability_Registry_v1.0.toml` 确定性生成。请勿手工编辑。

## 1. 覆盖摘要

| 指标 | 数量 |
|---|---:|
| 正式需求 | 37 |
| P0需求 | 34 |
| P1需求 | 3 |
| 正式测试 | 145 |
| 已实现测试 | 30 |
| 计划测试 | 115 |
| 已映射测试 | 126 |
| Source-scoped测试 | 19 |
| 唯一映射 | 148 |

## 2. 需求到测试

| Requirement | Priority | Source ID | Requirement | Tests |
|---|---|---|---|---|
| FMT-001 | P0 | FMT-001 | 所有整数使用Little Endian | TST-FMT-001, TST-FMT-013 |
| FMT-002 | P0 | FMT-002 | 偏移长度与计数使用无符号64位整数 | TST-FMT-001, TST-FMT-005, TST-SEC-004 |
| FMT-003 | P0 | FMT-003 | Writer只追加且不得原位修改已提交数据 | TST-FMT-006, TST-FMT-014 |
| FMT-004 | P0 | FMT-004 | Full自包含且Diff只绑定一个Full | TST-DIFF-002, TST-DIFF-004 |
| FMT-005 | P0 | FMT-005 | 格式结构不得跨物理卷 | TST-FMT-015, TST-VOL-001 |
| FMT-006 | P0 | FMT-006 | 只有合法Commit引用的Manifest构成提交归档 | TST-FAULT-001, TST-FAULT-002, TST-FMT-008, TST-FMT-009, TST-FMT-010 |
| FMT-007 | P0 | FMT-007 | Reader不信任外部或未经校验的信息 | TST-DIFF-004, TST-FILE-002, TST-FMT-005, TST-FMT-011, TST-SEC-007 |
| FMT-008 | P0 | FMT-008 | 未知必需特性拒绝且未知可选特性可跳过 | TST-FMT-003, TST-FMT-004 |
| FMT-009 | P0 | FMT-009 | 加密先于持久化且认证前不释放明文 | TST-CRY-002, TST-CRY-003, TST-CRY-004 |
| FMT-010 | P0 | FMT-010 | 结构具有显式版本长度和校验 | TST-FMT-001, TST-FMT-002, TST-FMT-005 |
| PRV-001 | P0 | PRV-001 | Provider不得操作NWB物理记录 | TST-PRV-001, TST-PRV-002 |
| PRV-002 | P0 | PRV-002 | Provider只向Core提交逻辑数据 | TST-PRV-003, TST-PRV-004 |
| PRV-003 | P0 | PRV-003 | Core独占格式与提交职责 | TST-PRV-005, TST-PRV-006 |
| PRV-004 | P0 | PRV-004 | Provider不得获取密钥或密码 | TST-PRV-007, TST-PRV-008, TST-SEC-006 |
| PRV-005 | P0 | PRV-005 | Provider崩溃不得产生伪成功 | TST-PRV-009, TST-PRV-010, TST-SEC-006 |
| PRV-006 | P0 | PRV-006 | Provider私有元数据具有公共Envelope和版本 | TST-PRV-011, TST-PRV-012 |
| PRV-007 | P0 | PRV-007 | 首版只加载Nüwa签名Provider | TST-PRV-013, TST-PRV-014 |
| PRV-008 | P0 | PRV-008 | 归档不得嵌入或执行Provider代码 | TST-PRV-015, TST-PRV-016 |
| REQ-001 | P0 | P-01 | 每次成功备份产生一个逻辑NWB归档 | TST-REQ-001, TST-REQ-002 |
| REQ-002 | P0 | P-02 | 完整逻辑归档独立可恢复 | TST-FILE-001, TST-FILE-003, TST-SAL-003, TST-VOL-005 |
| REQ-003 | P0 | P-03 | 差异备份只依赖指定完整备份 | TST-DIFF-001, TST-DIFF-002, TST-DIFF-003, TST-DIFF-004, TST-DIFF-005, TST-DIFF-006 |
| REQ-004 | P0 | P-04 | NWB归档是恢复真相 | TST-FILE-002, TST-FMT-011, TST-SAL-001, TST-SAL-002 |
| REQ-005 | P0 | P-05 | 提交后的归档不可原位修改 | TST-CRY-008, TST-FAULT-003, TST-FAULT-004, TST-FAULT-007, TST-FAULT-008, TST-SAL-004, TST-SAL-005 |
| REQ-006 | P1 | P-06 | 文件、卷和磁盘统一容器与记录层 | TST-REQ-003 |
| REQ-007 | P0 | P-07 | 大小、偏移和计数使用64位字段 | TST-FMT-001, TST-FMT-005, TST-SEC-004 |
| REQ-008 | P0 | P-08 | 提交内容可验证且加密具备机密性和真实性 | TST-CRY-004, TST-CRY-005, TST-CRY-006, TST-FMT-012, TST-VER-001, TST-VER-002, TST-VER-003 |
| REQ-009 | P0 | P-09 | Reader拒绝未知强制特性与非法布局 | TST-FMT-003, TST-FMT-004, TST-FMT-005, TST-SEC-004, TST-SEC-007 |
| REQ-010 | P0 | P-10 | BMR可仅凭完整归档和用户密钥恢复 | TST-REQ-004, TST-REQ-005 |
| REQ-011 | P1 | P-11 | BMR与异机还原是第一版标准能力 | TST-BMR-L-001, TST-BMR-L-002, TST-BMR-L-003, TST-BMR-L-004, TST-BMR-L-005, TST-BMR-L-006, TST-BMR-L-007, TST-BMR-L-008 |
| REQ-012 | P1 | P-12 | 第一版支持Windows UEFI/GPT恢复场景 | TST-BMR-W-001, TST-BMR-W-002, TST-BMR-W-003, TST-BMR-W-004, TST-BMR-W-005, TST-BMR-W-006, TST-BMR-W-007, TST-BMR-W-008 |
| REQ-013 | P0 | P-13 | Windows启动关键数据必须入档 | TST-REQ-006, TST-REQ-007 |
| REQ-014 | P0 | P-14 | 单文件与分卷共用协议 | TST-REQ-008, TST-REQ-009 |
| REQ-015 | P0 | P-15 | Chunk与Segment不得跨物理卷 | TST-FMT-007, TST-VOL-001, TST-VOL-002, TST-VOL-003 |
| REQ-016 | P0 | P-16 | 多分卷归档以Final Volume提交为有效恢复点 | TST-VOL-001, TST-VOL-004, TST-VOL-005, TST-VOL-006, TST-VOL-007, TST-VOL-009 |
| REQ-017 | P0 | IMP-001 | Format Registry ID唯一且生成一致 | TST-REG-001, TST-REG-002, TST-REG-003, TST-REG-004, TST-REG-005, TST-REG-006, TST-REG-007 |
| REQ-018 | P0 | IMP-002 | P0需求100%映射且CI拒绝孤儿 | TST-TRC-001, TST-TRC-002, TST-TRC-003, TST-TRC-004, TST-TRC-005, TST-TRC-006, TST-TRC-007, TST-TRC-008, TST-TRC-009, TST-TRC-010, TST-TRC-011, TST-TRC-012, TST-TRC-013, TST-TRC-014, TST-TRC-015, TST-TRC-016 |
| REQ-019 | P0 | IMP-003 | 结构化错误与日志不得泄露密钥或密码 | TST-ERR-001, TST-ERR-002, TST-ERR-003, TST-ERR-004, TST-ERR-005, TST-ERR-006, TST-ERR-007 |

## 3. 测试登记

| Test | Kind | Status | Traceability | Test case | Source |
|---|---|---|---|---|---|
| TST-BLK-001 | POSITIVE | PLANNED | SOURCE_SCOPED | BLOCK-01 Full/restore | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-002 | POSITIVE | PLANNED | SOURCE_SCOPED | 块Diff | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-003 | POSITIVE | PLANNED | SOURCE_SCOPED | GPT主表损坏 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-004 | NEGATIVE | PLANNED | SOURCE_SCOPED | GPT主备冲突 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-005 | NEGATIVE | PLANNED | SOURCE_SCOPED | 目标盘过小 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-006 | POSITIVE | PLANNED | SOURCE_SCOPED | 目标盘更大 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-007 | POSITIVE | PLANNED | SOURCE_SCOPED | 512e↔4Kn | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-008 | NEGATIVE | PLANNED | SOURCE_SCOPED | 源坏扇区 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BLK-009 | FAULT | PLANNED | SOURCE_SCOPED | 恢复目标I/O错误 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-001 | POSITIVE | PLANNED | MAPPED | UEFI/GPT/LVM/ext4 Full | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-002 | POSITIVE | PLANNED | MAPPED | Linux同场景Diff | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-003 | POSITIVE | PLANNED | MAPPED | Linux恢复到更大盘 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-004 | POSITIVE | PLANNED | MAPPED | 控制器变化异机 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-005 | POSITIVE | PLANNED | MAPPED | UUID变化 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-006 | NEGATIVE | PLANNED | MAPPED | XFS恢复与缩容阻止 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-007 | POSITIVE | PLANNED | MAPPED | Btrfs简单子卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-L-008 | NEGATIVE | PLANNED | MAPPED | 复杂RAID/ZFS阻止误导BMR | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-001 | POSITIVE | PLANNED | MAPPED | UEFI/GPT/NTFS Full同机 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-002 | POSITIVE | PLANNED | MAPPED | UEFI/GPT/NTFS Diff同机 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-003 | POSITIVE | PLANNED | MAPPED | Windows恢复到更大盘 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-004 | POSITIVE | PLANNED | MAPPED | SATA到NVMe虚拟异机 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-005 | POSITIVE | PLANNED | MAPPED | ESP/BCD重建 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-006 | NEGATIVE | PLANNED | MAPPED | 缺启动必需卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-007 | POSITIVE | PLANNED | MAPPED | BitLocker认证场景 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-BMR-W-008 | POSITIVE | PLANNED | MAPPED | Windows 7 SP1 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-001 | POSITIVE | PLANNED | SOURCE_SCOPED | Zstd和None选择 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-002 | POSITIVE | PLANNED | MAPPED | 正确密码 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-003 | NEGATIVE | PLANNED | MAPPED | 错误密码 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-004 | SECURITY | PLANNED | MAPPED | Ciphertext/Tag/AAD位翻转 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-005 | NEGATIVE | PLANNED | MAPPED | Key Slot损坏 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-006 | POSITIVE | PLANNED | MAPPED | Nonce唯一性 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-007 | SECURITY | PLANNED | SOURCE_SCOPED | 日志/崩溃/转储Secret扫描 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-CRY-008 | POSITIVE | PLANNED | MAPPED | 密码变更 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-DIFF-001 | POSITIVE | PLANNED | MAPPED | DIFF-01创建 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-DIFF-002 | POSITIVE | PLANNED | MAPPED | Full+Diff恢复 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-DIFF-003 | POSITIVE | PLANNED | MAPPED | 删除中间Diff | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-DIFF-004 | NEGATIVE | PLANNED | MAPPED | 错Full同名替换 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-DIFF-005 | POSITIVE | PLANNED | MAPPED | 变化日志回绕/丢失 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-DIFF-006 | POSITIVE | PLANNED | MAPPED | 只改元数据 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-ERR-001 | POSITIVE | IMPLEMENTED | MAPPED | 结构化错误使用Error Registry身份 | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-ERR-002 | NEGATIVE | IMPLEMENTED | MAPPED | 保留Invalid ErrorId拒绝 | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-ERR-003 | POSITIVE | IMPLEMENTED | MAPPED | 结构化JSON日志确定性与单LF | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-ERR-004 | SECURITY | IMPLEMENTED | MAPPED | Secret Canary格式化脱敏 | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-ERR-005 | SECURITY | IMPLEMENTED | MAPPED | Secret Canary日志写入错误不泄漏 | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-ERR-006 | SECURITY | IMPLEMENTED | MAPPED | 日志Schema无自由文本路径或载荷字段 | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-ERR-007 | NEGATIVE | IMPLEMENTED | MAPPED | 诊断事件严重度与阶段单一来源 | crates/nwb-diagnostics/tests/diagnostics_contracts.rs |
| TST-FAULT-001 | FAULT | PLANNED | MAPPED | 进程强杀 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-002 | FAULT | PLANNED | MAPPED | ENOSPC | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-003 | FAULT | PLANNED | MAPPED | USB拔出 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-004 | FAULT | PLANNED | MAPPED | SMB断开重连 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-005 | FAULT | PLANNED | SOURCE_SCOPED | 用户取消 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-006 | FAULT | PLANNED | SOURCE_SCOPED | 系统重启 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-007 | FAULT | PLANNED | MAPPED | Cache写失败 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FAULT-008 | FAULT | PLANNED | MAPPED | 陈旧锁/孤儿临时卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FILE-001 | POSITIVE | PLANNED | MAPPED | FILE-01 Full | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FILE-002 | POSITIVE | PLANNED | MAPPED | 删除外部Cache后浏览 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FILE-003 | POSITIVE | PLANNED | MAPPED | Full全量恢复 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-001 | POSITIVE | PLANNED | MAPPED | Header编码解码 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-002 | NEGATIVE | PLANNED | MAPPED | 坏Magic/CRC/SHA | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-003 | NEGATIVE | PLANNED | MAPPED | 未知Required Feature | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-004 | POSITIVE | PLANNED | MAPPED | 未知Optional Feature | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-005 | NEGATIVE | PLANNED | MAPPED | Record长度溢出/越界 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-006 | POSITIVE | PLANNED | MAPPED | Segment正常密封 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-007 | NEGATIVE | PLANNED | MAPPED | Segment各边界截断 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-008 | POSITIVE | PLANNED | MAPPED | Commit A损坏 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-009 | NEGATIVE | PLANNED | MAPPED | Commit A/B均损坏 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-010 | NEGATIVE | PLANNED | MAPPED | 两Commit合法但不一致 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-011 | NEGATIVE | PLANNED | MAPPED | Manifest错误引用 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-012 | POSITIVE | PLANNED | MAPPED | 独立Checker互验 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-FMT-013 | NEGATIVE | PLANNED | MAPPED | Endian Swap或Big Endian规范输入被拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-FMT-014 | NEGATIVE | PLANNED | MAPPED | 已密封Segment或已提交Archive重写被拒绝且原字节不变 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-FMT-015 | NEGATIVE | PLANNED | MAPPED | 跨卷Chunk、Record、Catalog Page或Segment被Reader拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-001 | POSITIVE | PLANNED | MAPPED | Provider只提交逻辑数据且Core创建物理Record | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-002 | NEGATIVE | PLANNED | MAPPED | Provider直接创建、定位或修改物理Record被拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-003 | POSITIVE | PLANNED | MAPPED | Object、Metadata、Stream、Range、Dependency和State正常提交 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-004 | NEGATIVE | PLANNED | MAPPED | 物理Offset、Record或未声明对象类型被拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-005 | POSITIVE | PLANNED | MAPPED | 格式、压缩、加密、Catalog、Commit与分卷由Core完成 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-006 | NEGATIVE | PLANNED | MAPPED | Provider指定物理布局、密文、Commit或绕过Core策略失败 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-007 | POSITIVE | PLANNED | MAPPED | 不透明数据接口完成且Provider不接触凭据 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-008 | SECURITY | PLANNED | MAPPED | 凭据Canary不出现在IPC、回调、内存输出或日志 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-009 | POSITIVE | PLANNED | MAPPED | Provider正常结束后Core才允许Commit | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-010 | FAULT | PLANNED | MAPPED | Provider生命周期崩溃不提交且不报告成功 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-011 | POSITIVE | PLANNED | MAPPED | 公共Envelope与Schema版本往返一致 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-012 | NEGATIVE | PLANNED | MAPPED | 缺Envelope、非法长度、未知Required Schema或坏Hash被拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-013 | POSITIVE | PLANNED | MAPPED | 合法Nüwa签名Provider可以加载 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-014 | SECURITY | PLANNED | MAPPED | 无签名、错误签名者、篡改Binary或Hash Mismatch被拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-015 | POSITIVE | PLANNED | MAPPED | Archive只保存Provider Metadata且Reader不执行 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-PRV-016 | SECURITY | PLANNED | MAPPED | 嵌入代码、脚本、Binary或执行指令作为非执行数据处理 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REG-001 | POSITIVE | IMPLEMENTED | MAPPED | RecordType完整性 | crates/nwb-format/tests/registry_tests.rs |
| TST-REG-002 | NEGATIVE | IMPLEMENTED | MAPPED | 重复ErrorId编译失败 | crates/nwb-format/tests/registry_tests.rs |
| TST-REG-003 | POSITIVE | IMPLEMENTED | MAPPED | Feature bit唯一 | crates/nwb-format/tests/registry_tests.rs |
| TST-REG-004 | NEGATIVE | IMPLEMENTED | MAPPED | Feature bit范围 | crates/nwb-format/tests/registry_tests.rs |
| TST-REG-005 | POSITIVE | IMPLEMENTED | MAPPED | Header枚举精确值 | crates/nwb-format/tests/registry_tests.rs |
| TST-REG-006 | POSITIVE | IMPLEMENTED | MAPPED | Registry快照 | crates/nwb-format/tests/registry_tests.rs |
| TST-REG-007 | NEGATIVE | IMPLEMENTED | MAPPED | 生成Rust与提交文件一致 | crates/nwb-format/tests/registry_tests.rs |
| TST-REQ-001 | POSITIVE | PLANNED | MAPPED | 成功Run恰好发布一个逻辑Archive与唯一Identity | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-002 | NEGATIVE | PLANNED | MAPPED | 零个、多个或未绑定输出不得报告成功 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-003 | POSITIVE | PLANNED | MAPPED | File、Volume与Disk共用Envelope、Record层和Reader入口 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-004 | POSITIVE | PLANNED | MAPPED | 删除Cache与配置数据库后只凭Archive和Key完成BMR | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-005 | NEGATIVE | PLANNED | MAPPED | 缺Archive、Required Volume或错误Key时写盘前阻止 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-006 | POSITIVE | PLANNED | MAPPED | 启动关键Artifact带Version与Hash入档 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-007 | NEGATIVE | PLANNED | MAPPED | 逐项缺少必要Boot Artifact时预检不得Ready | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-008 | POSITIVE | PLANNED | MAPPED | 同一Reader与协议打开等价单文件和多卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-REQ-009 | NEGATIVE | PLANNED | MAPPED | 模式专用或协议分叉结构被确定拒绝 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml |
| TST-SAL-001 | NEGATIVE | PLANNED | MAPPED | 尾部截断 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SAL-002 | NEGATIVE | PLANNED | MAPPED | Catalog页损坏 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SAL-003 | POSITIVE | PLANNED | MAPPED | 缺卷选择性恢复 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SAL-004 | POSITIVE | PLANNED | MAPPED | 输出抢救NWB | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SAL-005 | FAULT | PLANNED | MAPPED | Salvage空间不足/取消 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-001 | SECURITY | PLANNED | SOURCE_SCOPED | 路径穿越与设备路径 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-002 | SECURITY | PLANNED | SOURCE_SCOPED | Symbolic/Reparse链接逃逸 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-003 | SECURITY | PLANNED | SOURCE_SCOPED | Catalog循环和深度炸弹 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-004 | SECURITY | PLANNED | MAPPED | 超大长度和数量 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-005 | SECURITY | PLANNED | SOURCE_SCOPED | 压缩炸弹 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-006 | SECURITY | PLANNED | MAPPED | Provider恶意或崩溃 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-007 | SECURITY | PLANNED | MAPPED | Fuzz Record/Page/Manifest | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-SEC-008 | SECURITY | PLANNED | SOURCE_SCOPED | 目标磁盘误选 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-TRC-001 | POSITIVE | IMPLEMENTED | MAPPED | 合法Registry通过 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-002 | POSITIVE | IMPLEMENTED | MAPPED | Markdown确定性生成 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-003 | NEGATIVE | IMPLEMENTED | MAPPED | 重复Requirement拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-004 | NEGATIVE | IMPLEMENTED | MAPPED | 重复Test拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-005 | NEGATIVE | IMPLEMENTED | MAPPED | 重复Mapping拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-006 | NEGATIVE | IMPLEMENTED | MAPPED | 非法ID拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-007 | NEGATIVE | IMPLEMENTED | MAPPED | 悬空Requirement引用拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-008 | NEGATIVE | IMPLEMENTED | MAPPED | 悬空Test引用拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-009 | NEGATIVE | IMPLEMENTED | MAPPED | P0缺正向测试拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-010 | NEGATIVE | IMPLEMENTED | MAPPED | P0缺负向故障或安全测试拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-011 | NEGATIVE | IMPLEMENTED | MAPPED | 零P0拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-012 | NEGATIVE | IMPLEMENTED | MAPPED | MAPPED与SOURCE_SCOPED Disposition组合校验 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-013 | NEGATIVE | IMPLEMENTED | MAPPED | Source文件Anchor或Excerpt异常拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-014 | NEGATIVE | IMPLEMENTED | MAPPED | Markdown缺失或漂移拒绝 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-015 | NEGATIVE | IMPLEMENTED | MAPPED | 代码TST ID漏登记拒绝且旧裸测试仅库存 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-TRC-016 | NEGATIVE | IMPLEMENTED | MAPPED | CLI退出码0/2/3/4 | tools/traceability-checker/tests/traceability_tests.rs |
| TST-VER-001 | POSITIVE | PLANNED | MAPPED | 正常Archive Quick/Full Verify | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VER-002 | NEGATIVE | PLANNED | MAPPED | 单Chunk位翻转 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VER-003 | NEGATIVE | PLANNED | MAPPED | 多处损坏 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-001 | POSITIVE | PLANNED | MAPPED | 64MiB测试阈值多卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-002 | NEGATIVE | PLANNED | MAPPED | Segment接近卷边界 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-003 | NEGATIVE | PLANNED | MAPPED | Catalog Page接近边界 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-004 | POSITIVE | PLANNED | MAPPED | 分卷重命名和乱序 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-005 | NEGATIVE | PLANNED | MAPPED | 缺中间卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-006 | NEGATIVE | PLANNED | MAPPED | 缺Final Volume | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-007 | NEGATIVE | PLANNED | MAPPED | 混入其他Set卷 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-008 | POSITIVE | PLANNED | SOURCE_SCOPED | FAT32真实目标 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |
| TST-VOL-009 | FAULT | PLANNED | MAPPED | 空间不足 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md |

## 4. Source-scoped测试库存

| Test | Authority source | Anchor | Justification |
|---|---|---|---|
| TST-VOL-008 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | - `AUTO`：探测文件系统限制 | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-CRY-001 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | Writer对压缩收益不足的Chunk写`NONE` | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-CRY-007 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-303` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-FAULT-005 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md | 无论成功、失败、取消或超时 | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-FAULT-006 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Storage_Engine_Architecture_v2.0.md | 失败、取消和超时都必须释放Snapshot | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-001 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-500` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-002 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-503` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-003 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-502` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-004 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-502` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-005 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-504` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-006 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-504` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-007 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-504` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-008 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-500` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-BLK-009 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | \| `IMP-505` \| | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-SEC-001 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | 恢复路径必须经过包含性检查 | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-SEC-002 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | 符号链接/Reparse Point最后创建 | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-SEC-003 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | Reader必须验证页内偏移、Key排序、重复Key策略和树层级 | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-SEC-005 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | - 最大解压比例和单Chunk明文长度 | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |
| TST-SEC-008 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md | 块恢复接口必须暴露目标稳定ID | 当前37项正式需求未为该测试主题分配独立Requirement ID；保留为权威来源范围内计划测试，不计入需求覆盖。 |

## 5. 逐边证明语义

| Requirement | Test | Role | Rationale |
|---|---|---|---|
| FMT-001 | TST-FMT-001 | POSITIVE | Header按Little Endian规范编码解码。 |
| FMT-001 | TST-FMT-013 | NEGATIVE | Endian Swap或Big Endian规范输入被拒绝。 |
| FMT-002 | TST-FMT-001 | POSITIVE | 正常Header编码覆盖无符号64位偏移、长度与计数。 |
| FMT-002 | TST-FMT-005 | NEGATIVE | 超大长度、溢出和越界输入被拒绝。 |
| FMT-002 | TST-SEC-004 | NEGATIVE | 超大长度和数量触发checked arithmetic与资源上限。 |
| FMT-003 | TST-FMT-006 | POSITIVE | Segment按追加流程正常密封。 |
| FMT-003 | TST-FMT-014 | NEGATIVE | 密封或提交后重写被拒绝且原字节不变。 |
| FMT-004 | TST-DIFF-002 | POSITIVE | Full与其绑定Diff组合恢复当前状态。 |
| FMT-004 | TST-DIFF-004 | NEGATIVE | 替换为错误Full时ID与根哈希绑定拒绝。 |
| FMT-005 | TST-FMT-015 | NEGATIVE | 跨卷Chunk、Record、Page或Segment被Reader拒绝。 |
| FMT-005 | TST-VOL-001 | POSITIVE | 合法多卷创建恢复证明卷边界布局可用。 |
| FMT-006 | TST-FAULT-001 | FAULT | 进程强杀后只接受完整Commit。 |
| FMT-006 | TST-FAULT-002 | FAULT | ENOSPC时保持INCOMPLETE且无伪Commit。 |
| FMT-006 | TST-FMT-008 | POSITIVE | Commit A损坏时合法B仍构成可识别提交。 |
| FMT-006 | TST-FMT-009 | NEGATIVE | 两份Commit均损坏时不得正常打开。 |
| FMT-006 | TST-FMT-010 | NEGATIVE | 两份合法但不一致的Commit被拒绝。 |
| FMT-007 | TST-DIFF-004 | NEGATIVE | 外部同名错误Full不能绕过ID与根哈希校验。 |
| FMT-007 | TST-FILE-002 | POSITIVE | 删除外部Cache后Reader仍从归档列出和恢复。 |
| FMT-007 | TST-FMT-005 | NEGATIVE | 未经校验的非法偏移与长度被拒绝。 |
| FMT-007 | TST-FMT-011 | NEGATIVE | Manifest错误引用被拒绝。 |
| FMT-007 | TST-SEC-007 | SECURITY | Fuzz输入证明Reader不信任Record、Page或Manifest。 |
| FMT-008 | TST-FMT-003 | NEGATIVE | 未知Required Feature导致拒绝打开。 |
| FMT-008 | TST-FMT-004 | POSITIVE | 未知Optional Feature被安全跳过。 |
| FMT-009 | TST-CRY-002 | POSITIVE | 正确密码完成Catalog与数据恢复。 |
| FMT-009 | TST-CRY-003 | NEGATIVE | 错误密码不输出明文并返回确定错误。 |
| FMT-009 | TST-CRY-004 | SECURITY | 认证失败前不释放被篡改密文。 |
| FMT-010 | TST-FMT-001 | POSITIVE | Header往返证明显式长度、版本与校验。 |
| FMT-010 | TST-FMT-002 | NEGATIVE | 坏Magic、CRC或SHA输入被明确拒绝。 |
| FMT-010 | TST-FMT-005 | NEGATIVE | 非法结构长度与越界被拒绝。 |
| PRV-001 | TST-PRV-001 | POSITIVE | Provider仅提交逻辑数据且Core创建物理Record。 |
| PRV-001 | TST-PRV-002 | NEGATIVE | Provider直接操作物理Record被拒绝。 |
| PRV-002 | TST-PRV-003 | POSITIVE | 批准的逻辑对象、元数据、流、范围、依赖与状态正常提交。 |
| PRV-002 | TST-PRV-004 | NEGATIVE | 物理Offset、Record或未声明对象类型被拒绝。 |
| PRV-003 | TST-PRV-005 | POSITIVE | Core独占格式、Codec、Catalog、Commit与分卷。 |
| PRV-003 | TST-PRV-006 | NEGATIVE | Provider指定物理布局、密文、Commit或绕过Core失败。 |
| PRV-004 | TST-PRV-007 | POSITIVE | 不透明接口完成且Provider不接触凭据。 |
| PRV-004 | TST-PRV-008 | SECURITY | 凭据Canary不出现在Provider边界或日志。 |
| PRV-004 | TST-SEC-006 | SECURITY | 恶意或崩溃Provider不能取得或泄露Core密钥。 |
| PRV-005 | TST-PRV-009 | POSITIVE | Provider正常结束后Core才允许Commit。 |
| PRV-005 | TST-PRV-010 | FAULT | Provider任一生命周期崩溃均不提交且不报成功。 |
| PRV-005 | TST-SEC-006 | FAULT | 恶意或崩溃Provider不能使Core提交。 |
| PRV-006 | TST-PRV-011 | POSITIVE | 公共Envelope与Schema版本往返一致。 |
| PRV-006 | TST-PRV-012 | NEGATIVE | 缺Envelope、非法长度、未知Required Schema或坏Hash被拒绝。 |
| PRV-007 | TST-PRV-013 | POSITIVE | 合法Nüwa签名Provider可加载。 |
| PRV-007 | TST-PRV-014 | SECURITY | 无签名、错误签名或篡改Provider被拒绝。 |
| PRV-008 | TST-PRV-015 | POSITIVE | Archive只保存Provider Metadata且Reader不执行。 |
| PRV-008 | TST-PRV-016 | SECURITY | 嵌入可执行内容按非执行数据拒绝或忽略。 |
| REQ-001 | TST-REQ-001 | POSITIVE | 成功运行发布恰好一个归档与唯一身份，直接证明单次成功备份的归档基数。 |
| REQ-001 | TST-REQ-002 | NEGATIVE | 拒绝零个、多个或未绑定输出被报告为成功。 |
| REQ-002 | TST-FILE-001 | POSITIVE | 完整Catalog包含全部对象与数据，支持独立恢复。 |
| REQ-002 | TST-FILE-003 | POSITIVE | Full全量恢复内容与元数据一致。 |
| REQ-002 | TST-SAL-003 | POSITIVE | 缺卷时不引用缺卷的数据仍可选择性恢复。 |
| REQ-002 | TST-VOL-005 | NEGATIVE | 缺必要中间卷时明确列出受影响对象。 |
| REQ-003 | TST-DIFF-001 | POSITIVE | Diff仅保存变化数据并呈现当前Catalog。 |
| REQ-003 | TST-DIFF-002 | POSITIVE | 指定Full与Diff组合恢复当前状态。 |
| REQ-003 | TST-DIFF-003 | POSITIVE | 删除其他Diff不影响本Diff对Full的依赖。 |
| REQ-003 | TST-DIFF-004 | NEGATIVE | 错误Full的ID或根哈希不匹配时拒绝。 |
| REQ-003 | TST-DIFF-005 | FAULT | 变化日志失效时回退扫描或Full，避免遗漏。 |
| REQ-003 | TST-DIFF-006 | POSITIVE | 仅元数据变化时仍只依赖指定Full。 |
| REQ-004 | TST-FILE-002 | POSITIVE | 删除外部Cache后仍能从归档浏览和恢复。 |
| REQ-004 | TST-FMT-011 | NEGATIVE | Manifest错误引用不能作为恢复真相打开。 |
| REQ-004 | TST-SAL-001 | NEGATIVE | 尾部截断时仅从归档内密封边界识别可救对象。 |
| REQ-004 | TST-SAL-002 | NEGATIVE | Catalog损坏时正常打开失败且只进入显式Salvage。 |
| REQ-005 | TST-CRY-008 | POSITIVE | 密码变更输出新归档且原归档字节不变。 |
| REQ-005 | TST-FAULT-003 | FAULT | 介质拔出时已发布卷不被重写。 |
| REQ-005 | TST-FAULT-004 | FAULT | SMB中断仅从安全密封边界继续或重启。 |
| REQ-005 | TST-FAULT-007 | FAULT | Cache失败时已提交归档保持COMMITTED。 |
| REQ-005 | TST-FAULT-008 | FAULT | 陈旧锁与孤儿临时卷不会导致误删有效归档。 |
| REQ-005 | TST-SAL-004 | POSITIVE | Salvage写入新NWB并记录来源，不原位修复。 |
| REQ-005 | TST-SAL-005 | FAULT | Salvage空间不足或取消时原归档不变。 |
| REQ-006 | TST-REQ-003 | POSITIVE | 三类数据源共用Envelope、Record层与Reader入口，直接证明统一容器协议。 |
| REQ-007 | TST-FMT-001 | POSITIVE | Header编码验证64位长度与偏移字段的正常表示。 |
| REQ-007 | TST-FMT-005 | NEGATIVE | 长度溢出与越界被checked arithmetic拒绝。 |
| REQ-007 | TST-SEC-004 | NEGATIVE | 超大长度和数量受64位checked arithmetic与资源上限约束。 |
| REQ-008 | TST-CRY-004 | SECURITY | 密文、Tag或AAD篡改时认证失败且不释放数据。 |
| REQ-008 | TST-CRY-005 | NEGATIVE | 损坏Key Slot失败且不破坏其他合法Slot。 |
| REQ-008 | TST-CRY-006 | POSITIVE | 同一DEK下Nonce保持唯一以维持真实性。 |
| REQ-008 | TST-FMT-012 | POSITIVE | 独立Checker验证字段和全部根哈希一致。 |
| REQ-008 | TST-VER-001 | POSITIVE | 正常归档Quick与Full Verify通过。 |
| REQ-008 | TST-VER-002 | NEGATIVE | 单Chunk篡改被定位到精确对象与范围。 |
| REQ-008 | TST-VER-003 | NEGATIVE | 多处损坏全部报告且不被早退掩盖。 |
| REQ-009 | TST-FMT-003 | NEGATIVE | 未知Required Feature被明确拒绝。 |
| REQ-009 | TST-FMT-004 | POSITIVE | 未知Optional Feature可安全跳过并报告。 |
| REQ-009 | TST-FMT-005 | NEGATIVE | 非法长度、溢出与越界被拒绝。 |
| REQ-009 | TST-SEC-004 | NEGATIVE | 超大长度与数量被有界拒绝。 |
| REQ-009 | TST-SEC-007 | SECURITY | Record、Page与Manifest模糊输入不得导致崩溃或越界。 |
| REQ-010 | TST-REQ-004 | POSITIVE | 删除外部状态后仅凭Archive与Key完成BMR。 |
| REQ-010 | TST-REQ-005 | NEGATIVE | 缺少归档、必要卷或正确密钥时在写盘前停止。 |
| REQ-011 | TST-BMR-L-001 | POSITIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-002 | POSITIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-003 | POSITIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-004 | POSITIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-005 | POSITIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-006 | NEGATIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-007 | POSITIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-011 | TST-BMR-L-008 | NEGATIVE | Linux BMR场景直接验证第一版BMR或异机恢复能力边界。 |
| REQ-012 | TST-BMR-W-001 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-002 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-003 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-004 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-005 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-006 | NEGATIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-007 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-012 | TST-BMR-W-008 | POSITIVE | Windows BMR场景直接验证批准的UEFI/GPT恢复能力或阻断边界。 |
| REQ-013 | TST-REQ-006 | POSITIVE | 启动关键Artifact带版本与哈希完整入档。 |
| REQ-013 | TST-REQ-007 | NEGATIVE | 逐项缺失启动关键Artifact时预检不得Ready。 |
| REQ-014 | TST-REQ-008 | POSITIVE | 同一Reader与协议读取等价单文件和多卷归档。 |
| REQ-014 | TST-REQ-009 | NEGATIVE | 确定拒绝模式专用结构或协议分叉。 |
| REQ-015 | TST-FMT-007 | NEGATIVE | 截断Segment不进入正常Catalog。 |
| REQ-015 | TST-VOL-001 | POSITIVE | 多卷创建与恢复证明结构在合法卷边界内完成。 |
| REQ-015 | TST-VOL-002 | NEGATIVE | Segment接近边界时提前密封而不跨卷。 |
| REQ-015 | TST-VOL-003 | NEGATIVE | Catalog Page接近边界时保持单卷完整。 |
| REQ-016 | TST-VOL-001 | POSITIVE | 合法多卷集合创建并恢复成功。 |
| REQ-016 | TST-VOL-004 | POSITIVE | 重命名与乱序后仍由Set身份识别。 |
| REQ-016 | TST-VOL-005 | NEGATIVE | 缺中间卷时不形成完整恢复点。 |
| REQ-016 | TST-VOL-006 | NEGATIVE | 缺Final Volume时不得视为已提交。 |
| REQ-016 | TST-VOL-007 | NEGATIVE | 混入其他Set卷时哈希链或Set ID拒绝。 |
| REQ-016 | TST-VOL-009 | FAULT | 空间不足时不产生伪Commit。 |
| REQ-017 | TST-REG-001 | POSITIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-017 | TST-REG-002 | NEGATIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-017 | TST-REG-003 | POSITIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-017 | TST-REG-004 | NEGATIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-017 | TST-REG-005 | POSITIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-017 | TST-REG-006 | POSITIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-017 | TST-REG-007 | NEGATIVE | Format Registry自测直接验证ID唯一性、快照或生成一致性。 |
| REQ-018 | TST-TRC-001 | POSITIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-002 | POSITIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-003 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-004 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-005 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-006 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-007 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-008 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-009 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-010 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-011 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-012 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-013 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-014 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-015 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-018 | TST-TRC-016 | NEGATIVE | Traceability Checker自测直接验证Registry、映射、来源、定位器或CLI强检。 |
| REQ-019 | TST-ERR-001 | POSITIVE | 结构化诊断绑定唯一Error Registry ID与固定恢复语义字段。 |
| REQ-019 | TST-ERR-002 | NEGATIVE | 保留的零ErrorId不能形成有效诊断或日志记录。 |
| REQ-019 | TST-ERR-003 | POSITIVE | 确定性JSON Line证明结构化日志可机器处理且边界明确。 |
| REQ-019 | TST-ERR-004 | SECURITY | 密码与密钥Canary在Secret的Debug和Display路径中保持脱敏。 |
| REQ-019 | TST-ERR-005 | SECURITY | 底层I/O错误即使携带Canary文本也不会被二次输出。 |
| REQ-019 | TST-ERR-006 | SECURITY | Wire Schema没有自由文本、路径、载荷或底层错误文本入口。 |
| REQ-019 | TST-ERR-007 | NEGATIVE | 诊断事件从Diagnostic派生顶层严重度与阶段，消除同一日志内的冲突语义。 |

## 6. 旧测试库存（非正式追溯分母）

| Scope | `#[test]` count | Disposition |
|---|---:|---|
| Repository Rust test attributes before IMP-002 | 174 | Inventory only; existing unnumbered tests are not part of the formal IMP-002 denominator and are not renamed. |
