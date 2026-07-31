# Nüwa NWB 结构化错误与日志规范 v1.0

**工作包：** IMP-003
**状态：** CURRENT ENGINEERING STANDARD
**适用范围：** NWB Core、Reader、Writer、Catalog、Provider 边界、Restore、Verify、Salvage 与 CLI 的新增实现
**权威边界：** 本规范服从工程文档集 #1–6，不修改 ErrorId 数值、NWB 二进制格式、恢复语义或支持矩阵

---

## 1. 目标

IMP-003 建立统一的结构化错误与日志基础，使错误语义不依赖自由文本，并从类型和输出 Schema 两端阻止密码、密钥、敏感文件内容、路径载荷和底层错误文本进入日志。

现有 `crates/nwb-format/registry/error_ids.toml` 继续作为 ErrorId 的唯一来源；`format-registry-generator` 继续保证 ID 唯一性和生成代码一致性。本工作包不分配或改变任何 ErrorId 数值。

## 2. 强制规则

1. 新增 NWB 引擎代码必须使用 `nwb-diagnostics` 产生结构化诊断和日志事件。
2. 诊断必须绑定非零 `ErrorId`；`ErrorId::Invalid` 不得输出。
3. 错误对象必须显式包含严重度、阶段、可重试性和恢复影响；绑定诊断的日志事件必须从该诊断派生顶层严重度与阶段，不得接受第二套冲突输入。
4. 操作系统错误只允许记录数字错误码，不得记录可能包含路径、凭据或载荷的自由文本错误源。
5. 日志 Schema 不提供任意 `message`、`detail`、`path`、`payload`、`source_error` 或键值字符串入口。
6. 动态上下文只允许固定枚举、128 位不透明操作 ID、布尔状态和有界数值计数。
7. 用户可见文字必须在日志之外按 ErrorId 和事件类型解析；日志不得成为唯一错误语义。
8. `Secret` 不实现 `Clone`、`Serialize`、`AsRef` 或日志字段转换；`Debug` 和 `Display` 固定输出 `[REDACTED]`。
9. 日志写入失败必须丢弃底层错误文本，同时返回 `ErrorId::IoError`、固定阶段、严重度、可重试性、恢复影响和安全错误类别，避免泄漏与语义丢失。
10. 任何新增自由文本日志字段都属于契约变更，必须先完成安全审查、Canary 负向测试和本规范更新。

## 3. JSON Lines Schema

每条事件是单行 UTF-8 JSON，并以一个 LF 结束。字段顺序由实现固定，以便测试和工具稳定比较。

| 字段 | 类型 | 说明 |
|---|---|---|
| `schema_version` | `u16` | 当前为 1 |
| `timestamp_unix_ms` | `u64` | 调用方提供的 UTC Unix 毫秒时间 |
| `level` | enum | `info` / `warning` / `error` / `critical` |
| `event` | enum | 固定事件类型，不接受自由文本 |
| `stage` | enum | 固定引擎阶段 |
| `operation_id` | optional 32-char hex | 不透明关联 ID，不含用户数据 |
| `diagnostic` | optional object | ErrorId、严重度、阶段、重试与恢复影响 |
| `metrics` | object | 对象数、字节数、重试数和警告数 |

`diagnostic` 只允许：

- `error_id`：Error Registry 数值；
- `error_name`：生成枚举的固定名称；
- `severity`；
- `stage`；
- `retryability`；
- `recovery_impact`；
- `os_error_code`：可选数字。

## 4. Secret 生命周期边界

`Secret` 拥有私密字节，普通格式化永远只显示 `[REDACTED]`，析构前覆盖其持有的缓冲区。只有密码学边界可以通过显式 `expose_secret()` 短暂借用字节。调用方不得保存该借用、转换为日志字段或把它嵌入错误文本。

本工作包证明日志 API 与标准格式化不泄漏 Canary；它不替代 IMP-303 的真实 AMK/KEK/DEK 生命周期、异常转储、崩溃转储和进程内存扫描。`TST-CRY-007` 因此继续为 `PLANNED / NOT_RUN`。

## 5. 结构化错误语义

| 字段 | 规则 |
|---|---|
| ErrorId | 必须来自 `error_ids.toml`；0 值拒绝 |
| Severity | 固定枚举 |
| Stage | 固定枚举 |
| Retryability | `never` / `transient` / `after_user_action` |
| RecoveryImpact | `none` / `degraded` / `blocks_commit` / `blocks_restore` |
| OS error | 只记录数字码 |

结构化错误不得把 Debug 字符串、调用栈、路径、文件名、密码、Token、密钥、Catalog 明文或数据载荷放入日志。需要面向用户的说明时，上层应使用 ErrorId 映射到本地化资源。

## 6. 验收测试

| Test ID | 证明目标 |
|---|---|
| `TST-ERR-001` | 结构化错误使用生成的 Error Registry 身份 |
| `TST-ERR-002` | 保留的 `Invalid` ErrorId 被拒绝 |
| `TST-ERR-003` | JSON Line 确定、合法且只有一个 LF |
| `TST-ERR-004` | 密码/密钥 Canary 在 `Debug`、`Display` 中被脱敏 |
| `TST-ERR-005` | 含 Canary 的底层 I/O 错误文本不会二次泄漏 |
| `TST-ERR-006` | Wire Schema 不含自由文本、路径或载荷字段 |
| `TST-ERR-007` | 绑定诊断的事件从单一来源派生严重度和阶段，不能形成冲突语义 |

CI 必须在 Windows 与 Ubuntu 上单独执行 Secret Canary 过滤测试，并在完整 workspace 测试中再次执行全部契约测试。

## 7. 已知边界

- IMP-003 建立错误与日志基础契约，不实现 NWB Writer/Reader、加密、Key Slot 或归档持久化。
- 真实密码学密钥清理、崩溃/转储 Secret 扫描由 IMP-303 和 `TST-CRY-007` 验收。
- 日志轮转、保留周期、访问权限、导出与遥测上传策略不属于 IMP-003；在这些能力获批前不得上传日志。
- 旧 `src/`、`src-tauri/` 和 `ui/` 的 Repository 语义残留不是 NWB 日志契约的实现依据，后续清理不得削弱本规范。
