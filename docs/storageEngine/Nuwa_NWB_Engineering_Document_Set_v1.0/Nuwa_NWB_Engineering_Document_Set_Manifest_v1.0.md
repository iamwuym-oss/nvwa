# Nüwa NWB Engineering Document Set Manifest v1.0

**生成日期：** 2026-07-12  
**文档数量：** 8（不含本Manifest）

| 顺序 | 文件 | SHA-256 |
|---:|---|---|
| 1 | `Nuwa_NWB_Engineering_Document_Set_README_v1.0.md` | `3b8a55ef5d7645ca4bdc2ae1feaab04ff2aceedc7de32fc68d00f821e965cc9e` |
| 2 | `Nuwa_NWB_Storage_Engine_Architecture_v2.0.md` | `3996d95c3b25c06f7ce357230afced6a5fa37c9b31f11c6cc06fed7dfea863c6` |
| 3 | `Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md` | `57c0adb457ec4e06eea857b1058c2331ff3758433140b26fa6eb6028f8590660` |
| 4 | `Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md` | `b9f6ff0b375faa2b40c16bad9f4fa1162df2e4b0b8be9303c1ccf153c0409903` |
| 5 | `Nuwa_NWB_Product_Support_Matrix_v1.0.md` | `4eb8b5b00b85bf17c1b040d82a9e0fd0413d71a96668a5fb5556425e41c1a4c9` |
| 6 | `Nuwa_NWB_Implementation_Plan_v1.0.md` | `5cf82052c072c0be7cda5344c517a3187e96442d1bd9c8085786d1133e29323a` |
| 7 | `Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md` | `ef44291ecc9028d663d1c5acfc226f830a6837f79ae53c7c5d9b884d7a08ecc9` |
| 8 | `Nuwa_NWB_Test_Result_Record_v1.0.md` | `897dae11eb881abc55c020c7984ad4eea84998508ecfc97eeb02a6081e15808e` |

## 使用方法

1. Codex首先读取README；
2. 按README中的权威顺序读取其余文档；
3. 实施前验证文件SHA-256；
4. 从`IMP-000`开始，按Gate顺序实施；
5. 所有实际结果回填到Test Result Record；
6. 任何影响冻结原则的变更先提交ADR；
7. Format 1.0冻结或文档修订后生成新Manifest，不覆盖本版本。

## 当前真实性声明

- 架构、计划、验收标准和测试设计已经建立；
- 产品代码和恢复测试尚未由本套文档执行；
- 当前实际测试状态为`NOT_RUN`；
- 文档中的`CERTIFIED目标`与预期PASS不代表已经认证；
- 只有测试结果记录、证据和Gate签署能够改变实际状态。
