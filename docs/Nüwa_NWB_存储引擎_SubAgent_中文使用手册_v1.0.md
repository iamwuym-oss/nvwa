# Nüwa NWB存储引擎Sub-Agent中文使用手册 v1.0

**适用项目：** Nüwa Backup  
**适用阶段：** NWB存储引擎研发，当前重点为GATE-0～GATE-4  
**工程文档根目录：** `C:\Users\Tony\LCB\docs\storageEngine\Nuwa_NWB_Engineering_Document_Set_v1.0`  
**Sub-Agent定义：** `nwb_storage_engine_subagents.zip`中的9个TOML文件  
**手册目标：** 教你如何安装、检查、调用和管理9个Sub-Agent，按质量门完成NWB存储引擎开发。

---

## 1. 先理解：你不是在管理9个同时写代码的人

正确关系是：

```text
你（产品负责人/最终决策人）
        │
        ▼
Codex主会话（根协调代理）
        │
        ├── 规划、架构、审查类Sub-Agent
        ├── 当前唯一生产实现Sub-Agent
        ├── 独立验证Sub-Agent
        └── 证据文档Sub-Agent
```

你主要与“主Codex会话”说话。主Codex根据你的指令调用指定Sub-Agent、等待结果、汇总结论并安排下一步。

不要把9个角色理解成9个人同时修改仓库。NWB是永久备份格式，同时写入会产生冲突、格式漂移和责任不清。因此本项目的硬规则是：

> 任意时刻只能有一个生产代码实现角色写入仓库。

只读分析可以并行；生产代码、测试修改和证据回填必须按既定顺序进行。

---

## 2. 9个角色分别做什么

| 角色 | 简称 | 权限 | 什么时候使用 |
|---|---|---|---|
| `nwb_storage_project_manager` | 项目经理 | 只读 | 拆解IMP、检查依赖、安排角色、判断Gate |
| `nwb_format_architect` | 格式架构师 | 只读 | 冻结格式契约、审查ADR、解决设计冲突 |
| `nwb_core_engineer` | Core工程师 | 生产写入 | 容器、Record、Segment、Commit、分卷、压缩、加密 |
| `nwb_catalog_diff_engineer` | Catalog/Diff工程师 | 生产写入 | Chunk/Extent、Catalog、文件Full/Diff、Cache |
| `nwb_verify_salvage_engineer` | Verify/Salvage工程师 | 生产写入 | 验证、抢救、故障注入、崩溃恢复 |
| `nwb_validation_engineer` | 验证工程师 | 测试写入 | 独立测试、Fixture、故障测试、恢复比对、Gate建议 |
| `nwb_code_reviewer` | 代码审查员 | 只读 | 审查代码、测试、错误处理、安全和规范符合性 |
| `nwb_recovery_integrity_reviewer` | 恢复完整性审查员 | 只读 | 审查数据丢失、伪成功、损坏、错误恢复和崩溃风险 |
| `nwb_evidence_documenter` | 证据文档员 | 文档写入 | 回填测试结果、追溯、缺陷、Gate和已知限制 |

### 2.1 谁可以修改生产代码

只有以下3个角色可以修改生产代码：

- `nwb_core_engineer`
- `nwb_catalog_diff_engineer`
- `nwb_verify_salvage_engineer`

但三者不能同时工作。项目经理必须为每个IMP指定唯一实现者。

### 2.2 谁不能修代码

以下角色发现问题后只能报告，不能顺手修复：

- `nwb_format_architect`
- `nwb_code_reviewer`
- `nwb_recovery_integrity_reviewer`
- `nwb_storage_project_manager`

问题必须退回原实现角色修复，然后重新审查。

### 2.3 谁负责最终验收

没有任何单个Sub-Agent可以独立宣布完成。

一个工作包进入`ACCEPTED`至少需要：

1. 实现角色完成代码；
2. 代码审查员无阻塞问题；
3. 验证工程师完成规定测试；
4. 恢复完整性审查员确认没有伪成功和数据损坏风险；
5. 文档员完成证据归档；
6. 项目经理确认Gate条件满足；
7. 你批准进入下一工作包或Gate。

---

## 3. 安装Sub-Agent

### 3.1 目录位置

官方Codex的项目级自定义Agent放在仓库根目录：

```text
<NÜWA_REPO_ROOT>\.codex\agents\
```

假设`C:\Users\Tony\LCB`是Nüwa仓库根目录，最终结构应为：

```text
C:\Users\Tony\LCB\
├── .git\
├── AGENTS.md
├── .codex\
│   ├── config.toml
│   └── agents\
│       ├── nwb_storage_project_manager.toml
│       ├── nwb_format_architect.toml
│       ├── nwb_core_engineer.toml
│       ├── nwb_catalog_diff_engineer.toml
│       ├── nwb_verify_salvage_engineer.toml
│       ├── nwb_validation_engineer.toml
│       ├── nwb_code_reviewer.toml
│       ├── nwb_recovery_integrity_reviewer.toml
│       └── nwb_evidence_documenter.toml
└── docs\storageEngine\
    └── Nuwa_NWB_Engineering_Document_Set_v1.0\
```

官方Codex以TOML中的`name`字段识别Agent；文件名与`name`保持一致最容易维护。

### 3.2 不要同时保留旧角色

原来的这些角色属于旧Phase 3/Repository路线：

```text
phase3_project_manager
volume_backup_architect
repository_integration_engineer
vss_engineer
volume_io_engineer
volume_restore_engineer
phase3_validation_engineer
recovery_integrity_reviewer
evidence_documenter
```

不要让旧角色与新NWB角色混在同一个有效Agent目录中，否则主代理可能错误选择旧Repository角色。

安全做法是先移动到：

```text
.codex\agents-retired\phase3-old\
```

不要直接删除，确认新团队加载正常后再决定是否清理。

### 3.3 推荐的`.codex/config.toml`

在Nüwa仓库的`.codex\config.toml`中加入或合并：

```toml
[agents]
max_threads = 4
max_depth = 1
interrupt_message = true
```

含义：

- `max_threads = 4`：最多保留4个并发Agent线程；
- `max_depth = 1`：只有主代理能创建直接子代理，子代理不能继续无限创建下级代理；
- `interrupt_message = true`：中断Agent时保留可见记录。

`max_threads`是线程上限，不代表应该同时运行4个代码实现者。本项目仍执行“一个生产写入者”规则。

### 3.4 Codex++与DeepSeek注意事项

官方Codex允许在Agent文件中指定`model`和`model_reasoning_effort`，也允许省略并继承主会话。

如果Codex++实际使用`DeepSeek-v4-flash`，但Agent文件写着`gpt-5.4`，可能出现三种情况：

1. Codex++忽略Agent内模型字段，继续使用当前DeepSeek；
2. Codex++尝试调用不存在的GPT模型并报错；
3. 界面仍显示角色描述中的GPT字样，但真实模型没有改变。

最稳妥的兼容方法是：如果确认Codex++不能解析这些GPT模型名，从9个TOML中删除：

```toml
model = "..."
model_reasoning_effort = "..."
```

让角色继承主会话实际配置。角色身份来自`name`和`developer_instructions`，不是模型自报的“我是GPT-5”。

---

## 4. 首次启动前检查

在PowerShell中执行：

```powershell
cd C:\Users\Tony\LCB

git rev-parse --show-toplevel
Test-Path .\AGENTS.md
Get-ChildItem .\.codex\agents\*.toml
Get-ChildItem .\docs\storageEngine\Nuwa_NWB_Engineering_Document_Set_v1.0
git status --short
```

你需要确认：

- 当前目录确实是Nüwa仓库；
- `AGENTS.md`存在；
- 9个新TOML存在；
- 存储引擎文档可以读取；
- 开始前知道工作区是否已有未提交修改。

修改Agent文件或`AGENTS.md`后，应重新启动Codex或新建会话，使其重新加载项目指令。

---

## 5. 第一次验证Agent是否加载

在Nüwa仓库根目录启动一个新Codex会话，然后粘贴：

```text
这是Nüwa NWB存储引擎的Sub-Agent加载检查，只读，不允许修改任何文件。

请完成：
1. 报告当前仓库根目录和Git分支；
2. 报告已读取的AGENTS.md路径；
3. 检查项目级.codex/agents目录；
4. 列出以下9个自定义Agent是否可用：
   nwb_storage_project_manager
   nwb_format_architect
   nwb_core_engineer
   nwb_catalog_diff_engineer
   nwb_verify_salvage_engineer
   nwb_validation_engineer
   nwb_code_reviewer
   nwb_recovery_integrity_reviewer
   nwb_evidence_documenter
5. 读取工程文档目录：
   C:\Users\Tony\LCB\docs\storageEngine\Nuwa_NWB_Engineering_Document_Set_v1.0
6. 只返回加载结果、缺失项和阻塞项，不要开始实施。
```

通过标准：

- 9个名字完全一致；
- 没有调用旧`repository_integration_engineer`等角色；
- 能找到今天的NWB v2.0文档；
- 没有修改代码。

不要用“它自称是某个角色”作为唯一证据。真正证据是Codex显示了子代理线程，或者主代理明确报告已加载并能按名称派遣。

---

## 6. 每次开发前先给主代理一条总控制指令

每个新的主会话建议先粘贴下面这段：

```text
你是Nüwa NWB存储引擎本轮工作的根协调代理。

权威工程文档目录：
C:\Users\Tony\LCB\docs\storageEngine\Nuwa_NWB_Engineering_Document_Set_v1.0

本轮必须使用项目中的NWB自定义Sub-Agent。遵守以下规则：
1. 一次只处理一个明确IMP工作包；
2. 开始前由nwb_storage_project_manager只读规划；
3. 涉及格式契约时由nwb_format_architect只读确认；
4. 任意时刻只能有一个生产代码实现角色写入；
5. 实现完成后必须由nwb_code_reviewer独立只读审查；
6. 审查问题由原实现者修复，Reviewer不得修代码；
7. 修复完成后由nwb_validation_engineer独立验证；
8. 涉及恢复、提交、损坏或数据完整性时，由nwb_recovery_integrity_reviewer独立签署；
9. 只有真实证据齐全后，nwb_evidence_documenter才能回填文档；
10. 未执行测试保持NOT_RUN；代码完成最多是IMPLEMENTED；
11. 旧Phase S Repository不是当前架构，不得恢复repo.db、BlockStore、backup-objects或block-map.db路线；
12. GATE-8通过前只允许Format 0.x；
13. 每个阶段完成后先向我汇报并等待我确认，不要自动进入下一个工作包。

现在只确认规则和已加载角色，不开始写代码。
```

这条指令的作用是让主代理成为“项目总协调”，而不是直接冲进代码。

---

## 7. 标准工作包执行流程

每个IMP都使用同一套八步流程。

### 第1步：项目经理制定任务卡

示例：

```text
使用nwb_storage_project_manager子代理，只读分析IMP-100。

先读取权威工程文档和当前代码，输出：
- 需求/FMT/REL/SEC编号；
- 当前真实代码状态；
- 依赖项；
- 允许修改的文件边界；
- 唯一实现角色；
- 必须测试；
- 验收标准；
- 风险和阻塞项；
- 后续Sub-Agent调用顺序。

等待该子代理结束后汇总给我。不要写代码。
```

你检查任务卡。如果范围正确，再回复：

```text
我批准IMP-100任务卡，进入格式契约审查。除此之外不要实施其他IMP。
```

### 第2步：格式架构师确认契约

```text
使用nwb_format_architect子代理，只读审查IMP-100的Bootstrap Header契约。

必须核对Architecture v2.0和Binary Format Specification，输出：
- 当前权威字段和编码规则；
- 不变量；
- 文档内部是否存在冲突；
- 是否需要ADR；
- 必须正向、负向和截断测试；
- 结论：APPROVED、NEEDS_ADR或BLOCKED。

不要写代码。等待子代理完成后汇总。
```

只有结论为`APPROVED`才能实现。若为`NEEDS_ADR`，先解决ADR，不允许开发者自行选择字段布局。

### 第3步：唯一实现者写代码

IMP-100示例：

```text
使用nwb_core_engineer子代理实施且仅实施IMP-100。

约束：
- 读取已批准任务卡和格式架构结论；
- 只修改IMP-100批准范围内的文件；
- 不修改其他IMP；
- 使用Format 0.x；
- 添加编码、解码、往返、坏Magic、坏CRC、未知Feature、截断和溢出测试；
- 运行相关fmt、clippy、单元测试；
- 不提交Git、不推送、不进入IMP-101；
- 完成后报告文件、命令、退出码、测试结果和剩余NOT_RUN项。

其他生产写入Sub-Agent不得同时运行。等待该子代理结束后汇总。
```

完成后状态最多是`IMPLEMENTED`。

### 第4步：独立代码审查

```text
代码冻结，不允许任何写入。

使用nwb_code_reviewer子代理，只读审查IMP-100的全部变更、测试和生成字节。
重点检查：规范符合性、checked arithmetic、资源上限、panic/unsafe、错误处理、兼容性、测试是否会产生假阳性。

每个问题必须包含严重级别、文件位置、证据、影响、修复要求和复测要求。
结论只能是REJECT、CHANGES_REQUIRED、REVIEW_PASS_PENDING_VALIDATION或REVIEW_PASS。
等待审查完成后汇总，不要修代码。
```

### 第5步：原实现者修复

如果Reviewer发现问题：

```text
只调用原实现角色nwb_core_engineer，修复IMP-100审查报告中的BLOCKER/HIGH/MEDIUM问题。

不得扩展范围，不得删除失败测试，不得降低断言。
完成后重新运行相关测试并输出逐项修复映射。
然后停止，等待再次只读审查。
```

修复后必须再次调用`nwb_code_reviewer`。不能因为“已经修了”就跳过复审。

### 第6步：独立验证

```text
代码审查已经通过。使用nwb_validation_engineer子代理独立验证IMP-100。

要求：
- 根据测试计划建立REQ/FMT/REL/SEC到TST追溯；
- 执行规定的正向、负向、截断、损坏和边界测试；
- 记录Git提交/工作区状态、环境、命令、退出码、日志和产物SHA-256；
- 验证独立重新打开，不接受仅内存往返；
- 首次失败证据不得被重试覆盖；
- 未执行项目保持NOT_RUN；
- 输出PASS、CONDITIONAL_PASS或FAIL建议。

不要修改生产代码。等待验证完成后汇总。
```

如果验证发现生产缺陷，应退回原实现者，不能让验证工程师直接改生产代码。

### 第7步：恢复完整性审查

对于Header、Segment、Manifest、Commit、Full/Diff、加密、Verify和Salvage等任务，必须执行：

```text
使用nwb_recovery_integrity_reviewer子代理，对IMP-100的实现、代码审查和验证证据进行独立只读审查。

重点判断：
- 损坏Header是否可能被误认为有效；
- 不可信长度是否可能越界或无限分配；
- 是否存在伪成功、错误恢复或恢复真相缺失；
- 当前证据是否足以支持Gate。

输出BLOCKER/HIGH/MEDIUM/NOTE及FAIL、CONDITIONAL_PASS或INTEGRITY_PASS。
不要修改文件。
```

### 第8步：证据归档和项目经理签署

```text
使用nwb_evidence_documenter子代理，只更新IMP-100相关追溯、测试结果、缺陷、限制和Gate记录。

只能记录已经提供的真实证据：
- 实现文件和提交/工作区身份；
- 测试命令、退出码和日志；
- 产物SHA-256；
- Reviewer结论；
- Validation结论；
- Integrity结论；
- NOT_RUN项目和已知限制。

不得修改代码或测试，不得把IMPLEMENTED写成ACCEPTED。
```

最后调用项目经理：

```text
使用nwb_storage_project_manager子代理，只读检查IMP-100闭环证据。
给出最终状态和是否允许进入下一工作包。不要开始下一工作包。
```

你看到完整结果后再明确回复：

```text
我确认IMP-100状态为ACCEPTED，批准进入IMP-101规划。
```

---

## 8. 不同Gate应该调用哪些角色

| Gate | 工作包 | 主要实现者 | 强制配套角色 |
|---|---|---|---|
| GATE-0 | IMP-000～005 | `nwb_core_engineer` | PM、架构师、Reviewer、Validation、Documenter |
| GATE-1 | IMP-100～106 | `nwb_core_engineer` | 架构师、Reviewer、Validation、Integrity、Documenter |
| GATE-2 | IMP-200～207 | `nwb_catalog_diff_engineer` | 架构师、Reviewer、Validation、Integrity、Documenter |
| GATE-3 | IMP-300～306 | `nwb_core_engineer` | 架构师、Reviewer、Validation、Integrity、Documenter |
| GATE-4 | IMP-400～405 | `nwb_verify_salvage_engineer` | 架构师、Reviewer、Validation、Integrity、Documenter |

### 8.1 GATE-0：工程和契约基线

目标不是写备份功能，而是建立：

- Workspace与依赖方向；
- Format Registry；
- 需求—测试追溯；
- 结构化错误和脱敏日志；
- 可重复Fixture；
- Format 0.x策略；
- Windows/Linux构建基线。

第一条正式任务建议从`IMP-000`开始，不要直接跳到Header或Segment。

### 8.2 GATE-1：最小NWB容器

目标是形成：

```text
Header → Record → Sealed Segment → Manifest → Commit → 关闭 → 重新打开
```

必须证明截断和损坏不会成为合法Archive。

### 8.3 GATE-2：Catalog与文件Full/Diff

目标是证明：

- Full无外部数据库可恢复；
- Diff有完整逻辑Catalog；
- Diff只依赖指定Full；
- 错误Full被Root Hash拒绝；
- 删除Cache仍能浏览和恢复。

### 8.4 GATE-3：分卷、压缩和加密

目标是证明：

- 明文/加密；
- Zstd/NONE；
- 单文件/多分卷；
- 正确密码/错误密码；
- 缺卷/错序/替换；
- Final Volume未发布不能成功；
- Nonce不重复；
- 加密Catalog不泄漏文件名。

### 8.5 GATE-4：Verify、Salvage和故障恢复

目标是证明：

- Quick Verify正确识别身份和提交；
- Full Verify定位真实损坏；
- Salvage只读扫描，不覆盖原档案；
- 抢救结果明确标记缺失和降级；
- 任意写入阶段故障都不会产生伪Commit；
- 临时文件和陈旧锁可以安全处理。

---

## 9. 哪些任务可以并行，哪些绝对不能

### 9.1 可以并行

仅在相互独立且不修改同一工作区时，可以考虑并行：

- 两个只读Agent分别检查规范冲突和代码风险；
- 只读代码审查与只读安全分析；
- 多个平台读取测试日志；
- 多个独立Fixture的只读分析；
- 文档差异检查与日志摘要。

即使并行，也必须要求主代理等待全部Agent完成后统一汇总。

### 9.2 不能并行

- 两个生产实现Agent同时改代码；
- Reviewer审查时实现者继续修改同一文件；
- Validation运行基线时实现者继续提交变化；
- 文档员在测试尚未完成时提前写PASS；
- GATE-1和GATE-2同时推进；
- Core工程师与Catalog工程师各自定义不同Record或Chunk格式；
- 任何Agent在没有ADR时改变永久格式。

### 9.3 本项目推荐并发配置

虽然`max_threads = 4`，日常工作建议最多同时运行：

```text
1个主协调线程
+ 1个当前专业Agent
+ 最多2个独立只读分析Agent
```

不要为了“看起来像团队”而制造无意义并发。

---

## 10. 你每天怎么使用

### 开始工作

1. 打开Nüwa仓库；
2. 检查`git status`；
3. 新建主Codex会话；
4. 粘贴“总控制指令”；
5. 指定今天只做的一个IMP；
6. 先调用项目经理，不直接叫工程师写代码。

### 工作过程中

你只需要做三类决定：

- 批准任务范围；
- 批准ADR或设计决策；
- 根据独立证据批准进入下一步。

如果Codex问你是否继续，不要只回复“继续”，建议回复：

```text
我批准进入IMP-XXX的【具体阶段】。
只允许调用【角色名】完成【具体任务】。
不得进入下一个IMP，不得提交或推送。
完成后停止并汇报。
```

### 结束工作

要求主代理输出：

```text
请汇总今天的工作：
1. 完成的IMP和当前状态；
2. 实际修改文件；
3. 测试命令、退出码和证据；
4. Reviewer/Validation/Integrity结论；
5. NOT_RUN项目；
6. 未关闭缺陷；
7. Git工作区状态；
8. 明天第一个建议任务，但不要执行。
```

---

## 11. 如何查看、暂停和纠正Sub-Agent

### 11.1 官方Codex CLI

- 使用`/agent`查看和切换活动Agent线程；
- 可以进入某个线程查看它正在做什么；
- 可以直接要求主Codex停止、纠正或关闭某个Agent；
- 审批请求可能来自当前未打开的子线程，要注意线程标签。

### 11.2 IDE或桌面端

如果界面显示Background Agents/Subagents面板：

- `Active`表示正在运行；
- `Done`表示已经结束；
- 可以展开查看线程和结果；
- 发现两个生产实现者同时运行时应立即停止其中一个。

### 11.3 ChatGPT Work网页

网页可以运行Sub-Agent工作流，但本地Windows路径和本地TOML是否可见取决于任务是否连接了该仓库和环境。仅在网页聊天中输入`C:\...`，并不等于网页能够读取你的电脑。

### 11.4 Codex++

Codex++属于第三方客户端，其Agent面板、模型覆盖和TOML兼容性可能不同。判断标准不是界面是否显示漂亮的角色名称，而是：

- 是否真的创建了独立子线程；
- 是否按角色权限工作；
- 是否能等待并汇总；
- 只读Agent是否没有写文件；
- 生产写入是否保持单一。

如果Codex++不支持官方自定义Agent，临时降级方案是：每个阶段开一个新会话，把对应TOML中的`developer_instructions`作为角色指令粘贴进去，仍然按本手册串行执行。

---

## 12. 常见错误和纠正方式

### 错误1：一次下令“让9个Agent完成全部存储引擎”

后果：范围失控、并行冲突、测试和实现混杂。

纠正：一次只允许一个IMP，从项目经理规划开始。

### 错误2：开发者写完后自己宣布PASS

纠正：代码完成只能是`IMPLEMENTED`，必须独立Reviewer、Validation和Integrity审查。

### 错误3：继续使用旧Repository术语

出现以下内容应立即停止：

```text
repo.db作为提交真相
block-store/一块一文件
backup-objects/
block-map.db作为恢复真相
创建Repository
repository_id
```

要求项目经理和架构师重新核对NWB v2.0。

### 错误4：格式还没验证就写`format_major = 1`

纠正：GATE-8前只能使用0.x。

### 错误5：Reviewer直接修代码

纠正：Reviewer只报告，原实现者修复，再次复审。

### 错误6：测试失败后直接重跑，只保留成功结果

纠正：保留第一次失败日志、缺陷ID、修复记录和复测结果。

### 错误7：主会话上下文太长

处理方法：

- 每个IMP使用独立主会话；
- 主会话只保留决策和汇总；
- 原始日志留在证据文件；
- Gate结束后新建会话继续；
- 新会话先读取文档和前一Gate Closing Record。

### 错误8：Agent名称正确，但模型不对

角色名称和模型是两件事。先确认Provider和实际请求日志；如果使用DeepSeek，让Agent继承主模型，不要根据角色自我介绍判断模型。

### 错误9：只在ChatGPT网页里输入本地路径

网页不一定能访问本地Windows文件。需要在连接该本地仓库的Codex客户端运行，或把文档/仓库显式提供给当前环境。

---

## 13. 建议加入仓库`AGENTS.md`的永久规则

可以让Codex增加以下内容，但修改前先审查现有`AGENTS.md`，避免覆盖已有规则：

```markdown
## NWB Storage Engine Sub-Agent Governance

- Authoritative documents are under `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0`.
- Use the project-scoped custom agents under `.codex/agents/`.
- Process exactly one approved IMP work package at a time.
- Only one production-code subagent may write at a time.
- Implementers may not review, validate, or accept their own work.
- Required order: PM plan → format review → implementation → independent code review → fix → independent validation → recovery-integrity review → evidence update → gate decision.
- Old Phase S Repository structures and semantics are superseded and must not be reintroduced.
- Before GATE-8, generated archives must remain Format 0.x.
- Never convert planned or expected tests into PASS. Missing evidence remains NOT_RUN.
- Stop and request an ADR before changing Header, Record, Chunk, Segment, Catalog, Manifest, Commit, crypto, Volume Set, or Full/Diff dependency semantics.
```

---

## 14. 第一轮实际启动指令：从IMP-000开始

当9个Agent加载检查通过后，在主Codex会话粘贴：

```text
现在正式启动Nüwa NWB存储引擎研发，但本轮只做IMP-000的规划和现状盘点，不写代码。

权威文档目录：
C:\Users\Tony\LCB\docs\storageEngine\Nuwa_NWB_Engineering_Document_Set_v1.0

请执行：
1. 使用nwb_storage_project_manager读取AGENTS.md、全部权威工程文档和当前代码；
2. 确认Git仓库、分支、HEAD、工作区状态和现有目录；
3. 判断当前仓库结构与实施计划建议结构之间的真实差异；
4. 为IMP-000生成任务卡：需求ID、依赖、允许修改文件、唯一实现者、构建与CI测试、验收标准、证据、风险；
5. 明确旧Phase S Repository清理与IMP-000之间的边界，不得未经批准大规模删除；
6. 等待项目经理子代理结束后，由主代理汇总；
7. 不调用任何生产写入Agent，不修改文件，不进入IMP-001。

完成后停止，等待我批准。
```

这是整个存储引擎开发的正确起点。

---

## 15. Gate完成时的统一验收指令

例如GATE-1全部工作包完成后：

```text
对GATE-1进行独立最终验收，不开始GATE-2。

1. nwb_code_reviewer只读审查IMP-100～106最终代码和测试；
2. nwb_validation_engineer按GATE-1要求执行完整回归、截断、损坏和独立重开测试；
3. nwb_recovery_integrity_reviewer审查伪Commit、损坏识别和恢复真相；
4. nwb_evidence_documenter只根据实际证据更新Test Result Record和Gate记录；
5. nwb_storage_project_manager检查是否满足GATE-1全部退出条件；
6. 主代理汇总每个IMP状态、P0/P1缺陷、NOT_RUN项目和最终Gate建议；
7. 未经我批准，不得进入GATE-2。
```

只有当全部必要证据齐全时，你才回复：

```text
我批准GATE-1为ACCEPTED，允许开始GATE-2的只读规划。
```

---

## 16. 状态词怎么理解

| 状态 | 你应该如何理解 |
|---|---|
| `PLANNED` | 只完成计划 |
| `IN_PROGRESS` | 正在实施或测试 |
| `BLOCKED` | 有明确阻塞，不能继续 |
| `IMPLEMENTED` | 代码写完，但未完成独立验收 |
| `VERIFIED` | 自动测试和静态审查通过 |
| `ACCEPTED` | 恢复验证、证据、审查和Gate签署完整 |
| `REJECTED` | 验收失败，需要返工 |
| `NOT_APPLICABLE` | 架构批准不适用，有书面理由 |
| `NOT_RUN` | 尚未执行，绝对不是PASS |

你看到“完成了”“测试都过了”时，要继续问：

```text
是IMPLEMENTED、VERIFIED还是ACCEPTED？
对应命令、退出码、日志、产物哈希、恢复结果和独立复核人在哪里？
```

---

## 17. 官方Codex依据

- [Codex Subagents](https://developers.openai.com/codex/subagents)：项目自定义Agent目录、TOML字段、并发与线程管理。
- [Codex AGENTS.md](https://developers.openai.com/codex/guides/agents-md)：项目指令加载、优先级和验证方法。
- [Codex Advanced Configuration](https://developers.openai.com/codex/config-advanced)：项目`.codex/config.toml`、Agent配置和权限边界。

本手册中的NWB角色分工、IMP/Gate顺序、单一生产写入者、独立恢复完整性审查和证据闭环，是针对Nüwa Backup项目制定的工程治理规则。

---

## 18. 最简操作口诀

```text
一次一个IMP
先计划再设计
一次一个写入者
开发不能自审
审查后再验证
恢复必须有证据
文档不能提前写PASS
用户批准后才能进入下一步
```

