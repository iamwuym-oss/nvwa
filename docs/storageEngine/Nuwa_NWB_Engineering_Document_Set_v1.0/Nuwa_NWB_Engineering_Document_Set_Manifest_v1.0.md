# Nüwa NWB Engineering Document Set Manifest v1.0

**Document Set:** `Nuwa_NWB_Engineering_Document_Set_v1.0`
**Manifest updated:** 2026-07-19
**Task:** IMP-002 requirements–test traceability evidence closure

---

## Current Authenticity Statement

| Item | Status |
|---|---|
| GATE-0 | IN_PROGRESS |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET — `3bceb34`, run `29684903853` |
| IMP-002 | CLOSED / PASS / ACCEPTANCE MET — `8b2a68b`, run `29691731514` |
| IMP-003–005 | NOT_RUN |
| Product release | NOT_APPROVED |

The current result record is Test Result Record v1.2. Current work-package evidence is IMP-000 EVD v1.0, IMP-001 EVD v1.2 and IMP-002 EVD v1.0. Test Result Record v1.1 is historical/superseded; the malformed v1.0 records remain historical sources retained only for traceability.

The Traceability Registry is authoritative only for requirement/test IDs, priorities, authority-source bindings, mappings and dispositions. The generated Matrix is a reference rendering. Neither can override implementation contract documents #1–6.

## Hash Definition

SHA-256 covers each listed file's exact repository blob bytes. Current text artifacts use UTF-8/LF. Retained malformed historical files preserve their original bytes, including legacy line endings and control bytes. Git blob SHA-1 and platform-converted working-tree hashes are not substituted.

This manifest intentionally excludes itself to avoid a recursive hash. Therefore, the inventory count is the number of other governed files in this document-set directory.

## Document Inventory

Total governed files listed, excluding this manifest: **16**.

| # | File | Classification | SHA-256 |
|---:|---|---|---|
| 1 | `Nuwa_NWB_Engineering_Document_Set_README_v1.0.md` | REFERENCE / CURRENT | `56EF33D5605EC3F7CDDAC353E125C204B97FA01609F6CE936021BDA8FA46E7D6` |
| 2 | `Nuwa_NWB_Storage_Engine_Architecture_v2.0.md` | AUTHORITATIVE | `3996D95C3B25C06F7CE357230AFCED6A5FA37C9B31F11C6CC06FED7DFEA863C6` |
| 3 | `Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md` | AUTHORITATIVE | `57C0ADB457EC4E06EEA857B1058C2331FF3758433140B26FA6EB6028F8590660` |
| 4 | `Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md` | AUTHORITATIVE | `B9F6FF0B375FAA2B40C16BAD9F4FA1162DF2E4B0B8BE9303C1CCF153C0409903` |
| 5 | `Nuwa_NWB_Product_Support_Matrix_v1.0.md` | AUTHORITATIVE | `4EB8B5B00B85BF17C1B040D82A9E0FD0413D71A96668A5FB5556425E41C1A4C9` |
| 6 | `Nuwa_NWB_Implementation_Plan_v1.0.md` | AUTHORITATIVE | `5CF82052C072C0BE7CDA5344C517A3187E96442D1BD9C8085786D1133E29323A` |
| 7 | `Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md` | AUTHORITATIVE | `EF44291ECC9028D663D1C5ACFC226F830A6837F79AE53C7C5D9B884D7A08ECC9` |
| 8 | `Nuwa_NWB_Traceability_Registry_v1.0.toml` | TRACEABILITY AUTHORITY | `58A578D283BEFB10D988D365F9C6264E6E353F9C9125C8281D8E5506717E8C8E` |
| 9 | `Nuwa_NWB_Requirements_Test_Traceability_Matrix_v1.0.md` | GENERATED REFERENCE | `19CE717A522388193ABBB8C05DD088E0E48C9FE515B6DFA120F5F4B38199448C` |
| 10 | `Nuwa_NWB_Test_Result_Record_v1.2.md` | REFERENCE / CURRENT | `224A2DB6D5ECB8B796C33CEA3AF351BFAC1C350AF7A1DEBF7A2A52C8F81718F3` |
| 11 | `IMP-000_EVD_Build_Evidence_v1.0.md` | REFERENCE / CURRENT FOR IMP-000 | `434F4107FB6A970A80C0A4DF4B9180048C3766EA74B4E3C4F1927F04A0A3E5D7` |
| 12 | `IMP-001_EVD_Test_Result_Evidence_v1.2.md` | REFERENCE / CURRENT FOR IMP-001 | `2A53BDB7750A3239AF8972C746F16154307DE488C7F3892AD8E6129B463F76B9` |
| 13 | `IMP-002_EVD_Requirements_Traceability_Evidence_v1.0.md` | REFERENCE / CURRENT FOR IMP-002 | `400CA0C59D29E0D0CE39AC35E48FE83F576AF9D64032E4E6455721C74D36D176` |
| 14 | `Nuwa_NWB_Test_Result_Record_v1.1.md` | HISTORICAL / SUPERSEDED | `CA413B23E6499BB72E5E7F4095CE95C37F4A0E4A801D940900508EA198F424DB` |
| 15 | `Nuwa_NWB_Test_Result_Record_v1.0.md` | HISTORICAL / SUPERSEDED / MALFORMED_SOURCE_RETAINED_FOR_TRACEABILITY | `854F5F7A51C226B8177FA57703AE8895D117294BAB3C4AF0AD2B657E99C1CCCE` |
| 16 | `IMP-001_EVD_Test_Result_Evidence_v1.0.md` | HISTORICAL / SUPERSEDED / MALFORMED_SOURCE_RETAINED_FOR_TRACEABILITY | `4CAC2A7802666EA45B81D1A63DDCA220CE0729F180F95DA86F295E2A00AB8396` |

## Verification Notes

- All 16 listed files exist; the manifest itself is excluded.
- All current artifacts are UTF-8/LF and contain no prohibited control bytes.
- Historical files #15 and #16 are deliberately retained byte-for-byte. Their hashes certify retained malformed sources, not current project truth.
- Test Result Record v1.1 is well-formed historical evidence but is superseded by v1.2.
- Implementation evidence is bound to final tested commit `8b2a68b0528d37587636774eb76bc196b94dd59a` and CI run `29691731514`; later evidence-only edits do not alter that tested implementation.
- Registry inventory includes 115 `PLANNED` tests. Their registration and mapping status do not constitute execution evidence.
- PR #1 remains Draft and unmerged.
