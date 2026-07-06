# Nüwa Backup 设计决策日志 (ADL)

**文件用途：** 记录所有已讨论确认的设计决策，作为快速参考源，避免因会话重置导致信息丢失。
**更新日期：** 2026-07-05
**状态：** v2.1 — 持续更新

---

## 一、备份策略决策

### ADL-001：备份类型
| 决策 | 值 | 理由 |
|------|---|------|
| 完整备份 (Full) | ✅ 支持 | 基础备份类型 |
| 差异备份 (Differential) | ✅ 支持 | 基于最近一次 Full，单层依赖 |
| 增量备份 (Incremental) | ❌ 不支持 | 依赖链复杂，恢复时链条越长越慢，已排除 |
| 持续备份 (Continuous) | ❌ 不支持 | 仅做定时任务触发，不做实时文件监控 |

### ADL-002：差异备份变更追踪
| 决策 | 值 |
|------|-----|
| 追踪机制 | Windows 用 USN Journal + NTFS ；Linux 用 LVM |
| 差异基座 | 始终基于最近一次 Full，单层依赖 |
| 差异块索引 | 差异 .nwb 格式与全量相同，只是 Block Table 更小 |
| 恢复时 | 合并两个 Block Table（Diff 覆盖 Full 的对应条目） |

### ADL-003：备份源类型
| 决策 | 值 |
|------|-----|
| 文件/文件夹级 | ✅ 支持。用户可逐项选择 |
| 卷级 | ✅ 支持。可选择单个卷或多个卷 |
| 磁盘级 | ✅ 支持。可选择整个磁盘（含所有分区） |
| 启动分区 | ✅ 全部包含。ESP/MSR/Windows Recovery/OEM 均在内 |
| 动态磁盘 | ✅ 支持单磁盘模式 |
| OS RAID | ✅ 支持 |
| 硬件 RAID | ✅ 支持 |
| USB 设备作为源 | ✅ 支持 |
| 非 Windows 分区 | ✅ 按块级处理，已分配块全部备份，未分配跳过 |

### ADL-004：备份目标
| 决策 | 值 |
|------|-----|
| 本地副盘 | ✅ 支持 |
| USB 外置硬盘/SSD | ✅ 支持，自动创建 Nüwa Backup 专用目录结构 |
| NAS / SMB 共享 | ✅ 支持，CIFS 和 NFS 协议 |
| 本机同一磁盘 | ✅ 支持（备份到同一物理磁盘的不同分区）|
| 多个目标 | ❌ 不支持。一个备份任务只写入一个目标位置 |
| 网络目标管理 | ✅ 手动输入共享路径 + 浏览网络邻居 |
| SMB 凭据管理 | ✅ 用户名密码保存本地，NAS 密码变更后提示重新输入 |
| NFS 校验 | ✅ 用户选择时校验，不支持则弹出告警 |

### ADL-005：调度策略
| 决策 | 值 |
|------|-----|
| 一次性 | ✅ 支持 |
| 每日 (HH:MM) | ✅ 支持 |
| 每周 (选某几天) | ✅ 支持 |
| 每月 (每月第 X 天) | ✅ 支持 |
| 系统启动时 | ✅ 支持 |
| 用户登录时 | ✅ 支持 |
| USB 插入触发 | ✅ 支持 |
| 组合调度 | ✅ 可按月/周/天/小时自由组合 Full + Diff |

### ADL-006：保留策略
| 决策 | 值 |
|------|-----|
| 保留 N 个 Full | ✅ 支持，用户设置保留数量 |
| 依赖感知删除 | ✅ 差异依赖的 Full 不可删除，推后清理 |
| 事前空间检测 | ✅ 备份前预估所需空间，不足则取消并提示 |

---

## 二、恢复功能决策

### ADL-007：文件级恢复
| 决策 | 值 |
|------|-----|
| 浏览备份内容 | ✅ 支持，树形结构浏览 |
| 搜索 | ✅ 支持，按文件名搜索 |
| 恢复原位置 | ✅ 支持 |
| 恢复新位置 | ✅ 支持 |
| 恢复并改名 | ✅ 支持 |
| 文件索引机制 | ✅ 卷级备份时同步建立 SQLite 文件索引（NTFS USN Journal + ），差异备份同步更新索引 |

### ADL-008：卷级恢复
| 决策 | 值 |
|------|-----|
| 恢复到原卷 | ✅ 支持 |
| 恢复到不同卷 | ✅ 支持 |
| 恢复到更大目标盘 | ✅ 自动扩展 |
| 恢复到更小目标盘 | ✅ 条件支持（已用空间 <= 目标容量则自动缩容）|
| 恢复后引导修复 | ✅ 自动检测并修复 Windows BCD |

### ADL-009：裸机恢复
| 决策 | 值 |
|------|-----|
| 引导介质启动 | WinPE 或 Linux 恢复环境 |
| 自动发现备份 | 自动扫描本地磁盘和 USB 上的 .nwb 文件 |
| 网络备份定位 | 用户手动输入 SMB/NFS 路径 |
| 手动指定备份 | 用户浏览并选择 .nwb 文件 |

### ADL-010：异机还原 (Universal Restore)
| 决策 | 值 |
|------|-----|
| 恢复到不同硬件 | ✅ 支持 |
| 驱动注入时机 | ✅ 在恢复环境中完成，不占用桌面版资源 |
| 驱动库 | ✅ 内置通用驱动库（Intel/AMD/NVMe/RAID/网卡），用户可手动加载额外驱动 |
| 桌面版限制 | 桌面版可恢复备份到另一块硬盘，但不做驱动注入 |

---

## 三、磁盘克隆

### ADL-011：克隆模式
| 决策 | 值 |
|------|-----|
| 磁盘 → 磁盘 | ✅ 支持 |
| 磁盘 → 文件 (.nwb) | ✅ 支持 |
| 独立功能入口 | ✅ 集成到管理界面但复用备份/恢复引擎（Phase 3）|
| 目标盘确认 | ✅ 多重确认，目标盘所有数据将永久覆盖 |
| 禁止自我克隆 | ✅ 源磁盘 ≠ 目标磁盘 |

---

## 四、引导介质（Phase 3）


### ADL-012a：引导介质实现细节
| 决策 | 值 |
|------|-----|
| 讨论时间 | Phase 3 启动时再细化 |
| 未定项 | WinPE 最小体积、恢复环境语言（英文/中文）、网络配置 UI、驱动注入 UI |
| 理由 | 核心引擎优先，不提前设计 Phase 3 的实现细节 |
### ADL-012：恢复环境
| 决策 | 值 |
|------|-----|
| WinPE 版本 | ✅ Windows ADK 构建 |
| Linux 版本 | ✅ Alpine Linux 或类似轻量方案 |
| ISO 输出 | ✅ 提供 ISO，用户自行烧录到 USB 或光盘 |
| 驱动支持 | ✅ 预装常见网卡驱动，支持用户手动加载 |
| 恢复环境 UI | ✅ 完整 GUI，和 Acronis 恢复环境类似 |
| 构建策略 | ✅ WinPE/Alpine 在研发时下载到本地，直接集成到安装包中 |

---

## 五、验证与安全

### ADL-013：校验体系
| 决策 | 值 |
|------|-----|
| 校验级别 | 文件级 SHA-256 + 块级 xxHash3 |
| 校验时机 | 备份完成后立即校验 / 定期自动巡检 / 手动触发 |
| 损坏后行为 | 发出告警通知 → 自动尝试深度修复 → 修复失败则提示重建 |

### ADL-014：加密
| 决策 | 值 |
|------|-----|
| 默认状态 | ❌ 不加密。不影响备份速度 |
| 用户可选 | ✅ 可对单个备份任务启用 AES-256-GCM |
| 密码丢失 | ✅ 数据不可恢复，不设密码找回机制 |
| 信创扩展 | 后期支持 SM4 作为国产版选项 |

---

## 六、通知与性能

### ADL-015：通知方式
| 决策 | 值 |
|------|-----|
| Windows 系统通知 | ✅ Toast Notification |
| 邮件通知 (SMTP) | ✅ 支持（Phase 2/3 实现） |
| 声音提示 | ✅ 支持 |

### ADL-016：日志
| 决策 | 值 |
|------|-----|
| 保留周期 | 90 天 |
| 日志内容 | 操作时间、状态、数据量、错误详情 |

### ADL-017：性能限制
| 决策 | 值 |
|------|-----|
| 备份速度限制 | ✅ 用户可选 |
| CPU 使用率限制 | ✅ 支持 |
| 内存使用率限制 | ✅ 支持 |
| 磁盘 I/O 限制 | ✅ 支持 |
| 网络使用量限制 | ✅ 支持 |
| 备份窗口 | ✅ 仅在指定时间段内运行 |

---

## 七、命令行工具

### ADL-018：CLI
| 决策 | 值 |
|------|-----|
| 定位 | 与 GUI 并列，完整功能覆盖 |
| Windows 场景 | 系统排错、脚本自动化 |
| Linux 场景 | Linux 版和国产 OS 版的主要交互方式 |
| 支持命令 | 核心备份、恢复、验证、列表、磁盘信息 |

---

## 八、核心引擎技术

### ADL-019：.nwb 镜像格式
| 决策 | 值 |
|------|-----|
| 文件形态 | 单文件 .nwb，无 FAT32 约束，不分卷 |
| 块大小 | 64KB（用户可按 64KB 倍数上调，最高 64MB）|
| 内部 Block Group | 固定 65,536 数据块/Group |
| Segment | 4GB Segment（Group 原始数据量）|
| 空洞表示 | 完全不在索引中出现，恢复时隐式填零 |
| 压缩 | zstd 4 档：无 / L5 / L12 / L19 |
| 加密 | AES-256-GCM（RustCrypto）|
| 双层校验 | 块级 xxHash3-40 + 文件级 SHA-256 |
| XOR 奇偶校验 | 每 63 数据块 + 1 XOR，默认开启可选关闭，空间开销 1.56% |
| Block Index | 24 字节/条（3层×u64 位段打包）|
| 写入安全 | .tmp → fsync → 读回验证 → 原子重命名 |
| 合并写入 | 4MB 累积，减少 64x 系统调用 |

### ADL-020：管线架构
| 决策 | 值 |
|------|-----|
| 备份管线 | Reader(1线程) → 压缩池(N核) → Writer(1线程) |
| 恢复管线 | Reader(1线程) → 解压池(N核) → Sorter(1线程) → Writer(1线程) |
| 自适应压缩 | 熵检测：已压缩直写、全零变空洞 |
| 环缓冲 | Reader 预读 64MB |
| I/O 速度控制 | Token Bucket |
| CPU 控制 | 可配线程数（默认 N=CPU 核数）|
| 备份窗口 | 配置时间段，Group 边界暂停恢复 |
| 恢复排序缓冲 | 16MB（256 块窗口，乱序解压 → 排序写入）|

### ADL-021：技术栈
| 决策 | 选型 | 理由 |
|------|------|------|
| 核心引擎 | Rust | 跨平台编译、零运行时、内存安全、C ABI |
| UI 框架 | egui + eframe（纯 Rust）| 零运行时依赖、Win7~Win11 全覆盖、单 exe、Linux/国产 OS 复用 100% UI 代码 |
| IPC | ZeroMQ | 轻量、跨平台、异步模式 |
| 元数据存储 | SQLite | 零配置、嵌入式中高性能 |
| 压缩 | zstd | 压缩比和速度平衡 |
| 加密 | AES-256-GCM (RustCrypto) | 认证加密、无外部依赖 |
| 镜像挂载 | WinFsp + FUSE | 工业级 Windows FUSE |
| 构建系统 | Cargo (Rust) + CMake (Qt) | 各司其职 |

### ADL-022：线程模型
| 组件 | 功能 |
|------|------|
| nuwa-daemon | 系统服务，SYSTEM 权限 |
| nuwa-agent | 系统托盘进程，用户态 |
| nuwa-ui | Qt6 桌面主程序 |
| nuwa-cli | 命令行工具，和 GUI 并列 |

---

## 九、UI 界面（已确定方向，细节 Phase 2）

### ADL-023：UI 原则
| 决策 | 值 |
|------|-----|
| 风格 | 参考 Acronis True Image 风格和流程 |
| 备份源选择 | 列表式菜单 |
| 启动分区 | 自动识别并默认包含所有启动相关分区 |
| 恢复内容浏览 | 树形文件浏览 |
| 磁盘拓扑可视化 | 图形方式展示磁盘和分区布局 |
| 操作流程 | 向导式界面，每一步清晰说明 |
| UI 技术 | egui + eframe（纯 Rust，跨平台原生编译）|

---

## 十、信创与国产化（Phase 4）

### ADL-024：国产 OS 支持
| 决策 | 值 |
|------|-----|
| 麒麟 V10 | x86_64 + ARM64（飞腾）适配，Grub 引导修复 |
| 统信 UOS | x86_64 + ARM64（飞腾/鲲鹏）适配 |
| LoongArch | 交叉编译支持 |

---


## 十一、UI 界面设计（2026-07-05 确认）

### ADL-025：页面结构
| 决策 | 值 |
|------|-----|
| 导航方式 | 左侧导航栏 + 右侧内容区 |
| 导航项 | 仪表盘 / 备份 / 恢复 / 磁盘克隆 / 工具 / 日志 / 设置（共 7 项）|
| UI 风格 | 参考 Acronis True Image，深色科技风 |

### ADL-026：egui 组件体系
| 层级 | 内容 | 说明 |
|------|------|------|
| 应用层 | MainWindow / Navigator / PageStack | 窗口框架与页面切换 |
| 页面层 | 7 个页面：Dashboard/Backup/Restore/Clone/Tools/Log/Settings | 每个导航对应一个独立 QML 页面 |
| 业务组件 | DiskMap / BackupCalendar / ProgressCard / FileBrowser / BackupList | 可跨页面复用的功能模块 |
| 基础组件 | Sidebar / TopBar / Card / ButtonGroup / StatusIndicator / Toast / ConfirmDialog / WizardStepper | 最小 UI 积木 |
| 主题系统 | darkTheme（深蓝/黑底/青色高亮） | 深色科技风 |

---

## 十二、安装与部署（2026-07-05 确认）

### ADL-027：安装包
| 决策 | 值 |
|------|-----|
| 安装包形态 | 单 .exe 安装包，非 MSI |
| 默认路径 | C:\Program Files\Nüwa Backup\ |
| 依赖自动安装 | VC++ Redist + WinFsp |
| 安装后行为 | 注册 nuwa-daemon 系统服务，需重启 |
| 开始菜单 | 创建快捷方式 |

### ADL-028：自动更新
| 决策 | 值 |
|------|-----|
| 检查时机 | 每次启动时 + 设置中"检查更新"按钮 |
| 更新方式 | 整包替换（不下增量补丁）|
| 用户确认 | 需用户点击确认后才下载更新 |
| 旧版保留 | 不保留，官网提供上一版本下载链接 |
| 回滚 | 不提供自动回滚 |

### ADL-029：代码签名
| 决策 | 值 |
|------|-----|
| 测试阶段 | 不签名 |
| 正式发布 | 购买标准代码签名证书（非 EV）|
| 时间戳 | 加盖时间戳 |
| 证书提供商 | DigiCert / Sectigo，年费约 2000-3000 元 |
| 签名工具 | SignTool（Windows SDK 自带）|

---

## 十三、通知与质量保障（2026-07-05 确认）

### ADL-030：SMTP 邮件通知
| 决策 | 值 |
|------|-----|
| 配置入口 | 设置 → 通知 → 邮件 |
| 用户需提供 | SMTP 服务器地址、端口、邮箱地址、密码/授权码 |
| 触发时机 | 备份完成/失败/验证发现问题/磁盘空间不足 |
| 测试功能 | 提供"发送测试邮件"按钮 |
| 密码存储 | 加密后存 Windows Credential Manager |
| 默认状态 | 关闭，用户手动启用 |

### ADL-031：多语言策略
| 决策 | 值 |
|------|-----|
| 首发语言 | 英文（硬编码）|
| 国际化架构 | 当前不做，后续通过文本抽取 + qsTr() + 翻译文件实现 |
| 切换策略 | 不需要改业务逻辑代码，只需加翻译文件 |

### ADL-032：质量保障体系
| 环节 | 措施 |
|------|------|
| 编码阶段 | cargo fmt（统一风格）+ cargo clippy（静检）+ cargo test（单元测试）|
| 核心测试 | 备份/恢复闭环测试（创建备份→删除源→恢复→SHA-256 比对）|
| 容错测试 | 畸形 .nwb 文件模糊测试，模拟拔盘/断网等异常 |
| 版本兼容 | 每次发布前用旧版备份在新版上恢复验证 |
| 交付物 | 每个里程碑提供可双击安装的 .exe，用户可亲手测试 |

---


## 十四、跨平台代码复用（2026-07-05 确认）

### ADL-033：跨平台复用率
| 模块 | 复用率 | 说明 |
|------|--------|------|
| nuwa-core（核心引擎）| 100% | .nwb 格式、压缩、加密、校验、管线引擎——所有平台完全共享 |
| GUI（egui）| 100% | 纯 Rust 编译，Windows/Linux/国产 OS 各平台的 UI 代码完全一致 |
| CLI | 100% | 命令解析、输出格式、脚本接口——全平台相同 |
| OS 适配层 | 0%（需重写）| Windows 侧（VSS/PhysicalDriveX/bcdboot）与 Linux 侧（fsfreeze/dev/grub）完全不同的 API |
| **总体跨平台代码复用率** | **~80%** | 同一份 Rust 代码，不同平台重新编译即可，UI 和核心引擎无需修改 |

### ADL-034：GUI 框架调整
| 决策 | 旧方案 | 新方案 | 理由 |
|------|--------|--------|------|
| UI 框架 | Qt6 + QML | **egui + eframe（纯 Rust）** | Win7~Win11 全覆盖；零运行时依赖；单 exe 部署；跨平台编译即用 |
| 目标平台 | Win10+ | **Win7 SP1+ ~ Win11 + Server 全系列** | 工控机 Win7 场景必需 |


## 十四、性能基准测试标准（暂定，2026-07-05）

| 场景 | 指标 | 目标值 |
|------|------|--------|
| 全量备份（SSD→SSD）| 速度 | ≥ 500 MB/s |
| 全量备份（SMB 千兆网）| 速度 | ≥ 100 MB/s |
| 差异备份（5% 变更）| 速度 | ≥ 1000 MB/s |
| 全量恢复（SSD→SSD）| 速度 | ≥ 600 MB/s |
| 文件级恢复（搜索+开始）| 延迟 | ≤ 3 秒 |
| zstd 压缩比（系统盘）| 比率 | ≥ 2.0x |
| 备份验证（1TB 镜像）| 耗时 | ≤ 5 分钟 |
| CPU 使用率（限速关）| 平均 | ≤ 80% |
| 内存使用（备份峰值）| 峰值 | ≤ 512MB |

## 附录：Phase 划分

| Phase | 范围 | 状态 |
|-------|------|------|
| Phase 1 | Rust 核心引擎 + Windows CLI | ✅ 当前阶段 |
| Phase 2 | Windows 桌面 UI (Qt6 QML) | 🔵 Phase 1 完成后 |
| Phase 3 | 磁盘克隆 + 引导介质 + 异机还原 | 🔵 Phase 2 完成后 |
| Phase 4 | Linux + 国产 OS 适配 | 🔵 Phase 3 完成后 |
---

## 十五、灾备专家审查修正（2026-07-05 确认）

### ⚠️ SUPERSEDED 决策清单

以下 ADL 中与专家审查结论冲突的内容已被标记为 SUPERSEDED（被取代），以新 ADL 为准：

#### SUPERSEDED：ADL-001 中的 "差异备份 (Differential) ✅ 支持"
- **原值：** ✅ 支持
- **新值：** ❌ Future (Phase 5+)
- **理由：** USN Journal + 块映射过于复杂，不能作为第一版正确性基础
- **参考：** ADL-New-003

#### SUPERSEDED：ADL-002 差异备份变更追踪
- **原值：** USN Journal + NTFS $Bitmap 作为当前实现目标
- **新值：** 降级为 Future 研究方向，Phase 1 仅文件级完整备份
- **理由：** 差异备份已整体后置
- **参考：** ADL-New-003

#### SUPERSEDED：ADL-004 中的 "NFS 挂载 ✅ 支持"
- **原值：** ✅ 支持
- **新值：** ❌ Future (Phase 5+)
- **理由：** 网络存储场景复杂，SMB 优先
- **参考：** ADL-New-010

#### SUPERSEDED：ADL-010 异机还原 (Universal Restore)
- **原值：** Phase 3
- **新值：** Phase 5+
- **理由：** 异机还原依赖驱动注入，复杂度高，后置到 Phase 5+
- **参考：** ADL-New-001

#### SUPERSEDED：ADL-011 磁盘克隆
- **原值：** Phase 3
- **新值：** Phase 5
- **理由：** 克隆操作风险高，需先完成系统级恢复
- **参考：** ADL-New-001

#### SUPERSEDED：ADL-012a 引导介质
- **原值：** Phase 3
- **新值：** Phase 4（需在 Phase 3 卷级恢复稳定后进行）
- **理由：** 可启动介质细节推迟到 Phase 3 讨论

#### SUPERSEDED：ADL-013 WinPE 构建
- **原值：** Phase 3
- **新值：** Phase 4
- **理由：** 与引导介质一同后置

#### SUPERSEDED：附录「Phase 划分」表中的 Phase 2/3/4 定义
- **原值：** Phase 2=GUI, Phase 3=克隆+介质+异机, Phase 4=跨平台
- **新值：** Phase 2=可用文件备份+GUI, Phase 3=NTFS镜像, Phase 4=可启动恢复, Phase 5=磁盘克隆, Phase 5+=高级
- **理由：** 专家审查要求更合理的阶段划分

#### SUPERSEDED：性能基准测试标准
- **原值：** 500MB/s 等目标作为验收标准
- **新值：** 性能目标不作为 Phase 1 验收标准，仅作为 Future 参考
- **理由：** Phase 1 正确性优先，性能优化后置

---

### 新增：ADL-New-001 ~ ADL-New-010

#### ADL-New-001：MVP Scope Freeze
| 决策 | 值 |
|------|-----|
| MVP 范围 | 文件级最小备份/恢复闭环 |
| 不包含 | 卷、磁盘、VSS、WinPE、daemon、UI、差异、克隆、异机还原 |
| Phase 1 格式 | 平文件存储（目录 + JSON manifest），不使用 .nwb |
| Phase 1 OS | Win10/Win11/Win Server 2019/2022（不含 Win7）|
| 验收标准 | 恢复验证优先于备份完成 |

#### ADL-New-002：Restore-Validation-First
| 决策 | 值 |
|------|-----|
| 完成标准 | 以恢复验证为准，非"备份任务完成" |
| 文件级 | 恢复后文件 SHA-256 与源文件一致 |
| 卷级（Future）| 恢复后卷可挂载，文件系统校验通过 |
| 系统级（Future）| 恢复后系统可启动、登录、识别系统盘 |
| 异机还原（Future）| 恢复后系统在不同硬件上可启动、无蓝屏 |

#### ADL-New-003：Differential Backup Deferred
| 决策 | 值 |
|------|-----|
| 差异备份 | ❌ Phase 1/2/3 不实现 |
| 计划阶段 | Phase 5+（Future）|
| USN Journal | 仅作为未来优化研究方向，不作为当前实现目标 |
| 差异 .nwb 字段 | 标记为 Reserved / Future，不得在 v0.1 中实现 |

#### ADL-New-004：XOR Parity Deferred
| 决策 | 值 |
|------|-----|
| XOR Parity | ❌ Phase 1/2/3 不实现 |
| 计划阶段 | Phase 5+（v1.0+ 格式）|
| 第一版策略 | 只做 SHA-256 校验检测，不做自动修复 |
| 损坏处理 | 检测到损坏后告警 + 提示用户重建备份 |

#### ADL-New-005：.nwb v0.1 Experimental
| 决策 | 值 |
|------|-----|
| 格式版本 | v0.1（实验阶段）|
| 引入阶段 | Phase 2 中期（可选实验）|
| 正式格式 | v0.2+ → Phase 3 |
| 排除特性 | Differential、XOR、AES 加密、Block Group 复杂索引 |
| Phase 1 格式 | 平文件存储（目录 + JSON manifest）|

#### ADL-New-006：Windows MVP Platform Scope
| 决策 | 值 |
|------|-----|
| 优先验证平台 | Windows 10、Windows 11、Windows Server 2019、Windows Server 2022 |
| 后续验证 | Windows 7 SP1、Server 2008 R2、Server 2012 R2 |
| Win7 支持 | Phase 2+（工控机场景）|
| GUI 兼容 | egui + eframe 原生支持 Win7 SP1+ |

#### ADL-New-007：Unknown File System Safety
| 决策 | 值 |
|------|-----|
| 未知文件系统 | 不得假装支持智能已用块识别 |
| Phase 3 策略 | NTFS 卷级备份使用 VSS + $Bitmap 识别已用块 |
| 非 Windows FS（Future）| 按 raw device 全盘备份或标记 unsupported |
| 安全原则 | 宁可多备（raw block），不能漏备 |

#### ADL-New-008：Same-Disk Backup Warning
| 决策 | 值 |
|------|-----|
| 同盘备份策略 | 同一物理磁盘不同分区作为备份目标必须强警告 |
| Phase 1 | 禁止将源路径和目标路径设置为同一位置 |
| Phase 2 | 允许同盘不同分区，但要求明确用户确认 |
| 安全设计 | 如果系统盘故障，同盘备份也会丢失，必须告知用户风险 |

#### ADL-New-009：Daemon/IPC Deferred
| 决策 | 值 |
|------|-----|
| lcb-daemon 系统服务 | ❌ Phase 1 不实现 |
| 计划阶段 | Phase 3 |
| Phase 1 模式 | CLI 直接运行，不以后台服务形式常驻 |
| IPC 安全 | 如后续实现 daemon，必须先完成安全设计（凭据验证、权限隔离）|

#### ADL-New-010：Performance Goals Deferred
| 决策 | 值 |
|------|-----|
| 性能目标 | ❌ 不作为 Phase 1 验收标准 |
| 数据完整性优先级 | 数据不丢 > 可校验 > 可恢复 > 中断安全 > 用户安全 > 性能 |
| 优化启动阶段 | Phase 4+（卷级恢复稳定后）|
| 管线化引擎 | 全部移入 `05_Pipeline_Optimization_Design.md` Future 附录 |

---

## 十六、当前 Phase 划分（修正后）

| Phase | 范围 | 状态 |
|-------|------|------|
| Phase 0 | 文档规范修正 | ✅ 已完成 |
| **Phase 1** | **MVP：文件级备份/恢复 CLI** | **✅ 当前阶段** |
| Phase 2 | 可用文件备份 + 计划任务 + SMB + GUI（egui） | 🔵 Phase 1 完成后 |
| Phase 3 | NTFS 非系统卷镜像 + VSS + GPT/MBR + .nwb v0.2 | 🔵 Phase 2 完成后 |
| Phase 4 | WinPE/Alpine 引导介质 + 系统卷恢复 + 裸机恢复 | 🔵 Phase 3 完成后 |
| Phase 5 | 磁盘克隆 | 🔵 Phase 4 完成后 |
| Phase 5+ | 差异备份、XOR、加密、异机还原、Linux、国产 OS | 🔵 Future |

---

## Revision History

| 版本 | 日期 | 变更原因 |
|------|------|---------|
| v1.0 | 2026-07-04 | 初始版本 |
| v2.0 | 2026-07-05 | 产品名统一为 Nüwa Backup |
| v2.1 | 2026-07-05 | 灾备专家审查后：新增 10 条 ADL（New-001~New-010），标记冲突 ADL 为 SUPERSEDED，更新 Phase 划分 |

---

## 十七、T2-08 Dashboard 设计决策

| # | 决策 | 值 | 说明 |
|---|------|-----|------|
| ADL-GUI-001 | 多 Job 显示策略 | **聚合显示** | Protected Jobs 显示总数，Storage Usage 用默认 job 的目标存储，Last Backup 取全局最新 |
| ADL-GUI-002 | Backup Jobs 行 | **仅最近一次任务** | 第三行第二列只显示最近一条备份任务的完整详情 |
| ADL-GUI-003 | 跨页面跳转机制 | **egui Id 信号** | Quick Actions 通过设置 `ctx.memory_mut().data.insert_temp` 触发页面切换，`app.rs` 在 CentralPanel 渲染后检测并跳转 |
| ADL-GUI-004 | Dashboard 主题 | **侧栏深色 + 内容区白底黑字** | 用户要求中央内容区白底黑字，导航栏保持深色科技风 |
| ADL-GUI-005 | 语言 | **运行时全英文** | 遵循 T2-LANG-01 政策，所有 UI 字符串为英文 |

---

## Revision History

| 版本 | 日期 | 变更原因 |
|------|------|---------|
| v1.0 | 2026-07-04 | 初始版本 |
| v2.0 | 2026-07-05 | 产品名统一为 Nüwa Backup |
| v2.1 | 2026-07-05 | 灾备专家审查后：新增 10 条 ADL |
| v2.2 | 2026-07-06 | T2-08 Dashboard 完成，新增 5 条 ADL-GUI 决策 |

---

## 十八、Phase 3 Scope Reset — T3-00

### ADR-P3-001 — Phase 3 Scope Reduced to NTFS Non-System Volume Image MVP

| 决策 | 值 |
|------|-----|
| Phase 3 范围 | 仅限 .nwb v0.2、块级 SHA-256、VSS 快照生命周期、非系统 NTFS 卷备份 CLI、非系统 NTFS 卷恢复 CLI、Phase 3 关闭验证 |
| 排除范围 | GPT/MBR 分区解析、启动分区识别、BCD 修复、WinPE、裸机恢复、daemon、GUI 卷操作页面、动态磁盘、RAID、增量/差异备份、加密、压缩、去重 |
| 理由 | 保持 Phase 3 聚焦可测试的核心链，避免将系统恢复、BCD、WinPE、克隆、daemon、GUI 混入卷镜像 MVP；降低对系统卷的破坏操作风险；先奠定 .nwb 和 VSS 基础 |

### ADR-P3-002 — Phase 3 Safety Boundary

| 决策 | 值 |
|------|-----|
| 支持环境 | Windows 仅、管理员模式、本地固定磁盘、NTFS 仅、非系统/非启动卷 |
| CLI 优先 | Phase 3 不做 GUI 卷操作页面 |
| 拒绝列表 | C:、系统卷、ESP、MSR、恢复分区、FAT32/exFAT、动态磁盘、RAID、网络路径、可移动介质 |
| 恢复确认 | 卷恢复是破坏性操作，必须显式 Y/N 确认 |

### ADR-P3-003 — GPT/MBR Not Required for Phase 3

| 决策 | 值 |
|------|-----|
| Phase 3 是否需要 GPT/MBR | 不需要 |
| 理由 | 非系统卷可通过 Windows 卷 API 直接枚举和访问，不需要底层分区表解析。GPT/MBR 是系统恢复和磁盘克隆的前置依赖。 |
| 延后 | GPT/MBR 解析归入 Phase 3.5 |

---

## Revision History

| 版本 | 日期 | 变更原因 |
|------|------|---------|
| v1.0 | 2026-07-04 | 初始版本 |
| v2.0 | 2026-07-05 | 产品名统一为 Nüwa Backup |
| v2.1 | 2026-07-05 | 灾备专家审查后：新增 10 条 ADL |
| v2.2 | 2026-07-06 | T2-08 Dashboard 完成，新增 5 条 ADL-GUI 决策 |
| v2.3 | 2026-07-06 | T3-00 Phase 3 范围重置，新增 3 条 ADR-P3 决策 |

---

## ADR-T2.5-001 — Replace egui Desktop GUI with Tauri 2.0

**Date:** 2026-07-06
**Status:** APPROVED
**Supersedes:** ADL-034, ADL-026, ADL-039, ADR-GUI-003 (egui-related decisions)

### Decision

Replace egui + eframe (Rust immediate-mode GUI) with Tauri 2.0 + React + TypeScript + Vite for the desktop GUI layer.

### Rationale

| Concern | egui | Tauri 2.0 |
|---------|------|-----------|
| Visual quality ceiling | Low — limited by immediate-mode architecture | Unlimited — full CSS/HTML control |
| Commercial-grade UI | Not achievable | Same capability as modern desktop apps |
| Component reusability | Poor — no component model | High — React component architecture |
| Type safety across IPC | One side (Rust) | Both sides (Rust + TypeScript) |
| Ecosystem | Small, niche | Large, mainstream |
| Long-term maintenance | Poor as UI grows | Structured and scalable |

### Consequences

Positive:
- Professional-quality UI is now achievable
- Better development experience with React + TypeScript
- Tauri ecosystem has rich community support

Negative:
- ~200MB Node.js toolchain required for development
- Frontend development now requires frontend tooling (npm, Vite)
- GUI rewrite required — egui code cannot be reused

Migration actions:
- src/gui/ and src/gui_main.rs deleted
- Cargo.toml egui/eframe references removed
- All core library code preserved unchanged
- Tauri scaffolding to be implemented in T2.5-01+
