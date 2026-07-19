# Nüwa NWB Engineering Document Set Manifest v1.0

**Document Set:** `Nuwa_NWB_Engineering_Document_Set_v1.0`
**Manifest updated:** 2026-07-19
**Task:** IMP-001 Generator Remediation evidence closure

---

## Current Authenticity Statement

| Item | Status |
|---|---|
| GATE-0 | IN_PROGRESS |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET — `3bceb34`, run `29684903853` |
| IMP-002 | NOT_RUN / NOT_STARTED |

The current IMP-001 evidence is EVD v1.2 and Test Result Record v1.1. Their v1.0 predecessors are historical malformed source records retained only for traceability.

## Hash Definition

SHA-256 covers the exact repository blob bytes as committed. Current text documents use UTF-8/LF; retained malformed historical files preserve their original bytes, including legacy line endings and control bytes. Git blob SHA-1 and platform-converted working-tree hashes are not substituted. This manifest excludes itself to avoid a recursive hash.

## Document Inventory

Total documents listed: **12**.

| # | File | Classification | SHA-256 |
|---:|---|---|---|
| 1 | `Nuwa_NWB_Engineering_Document_Set_README_v1.0.md` | REFERENCE / CURRENT | `561471C2B4062C615D40FBCAF641B29B18180C73673ADEF463B9C82E9468316D` |
| 2 | `Nuwa_NWB_Storage_Engine_Architecture_v2.0.md` | AUTHORITATIVE | `3996D95C3B25C06F7CE357230AFCED6A5FA37C9B31F11C6CC06FED7DFEA863C6` |
| 3 | `Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md` | AUTHORITATIVE | `57C0ADB457EC4E06EEA857B1058C2331FF3758433140B26FA6EB6028F8590660` |
| 4 | `Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md` | AUTHORITATIVE | `B9F6FF0B375FAA2B40C16BAD9F4FA1162DF2E4B0B8BE9303C1CCF153C0409903` |
| 5 | `Nuwa_NWB_Product_Support_Matrix_v1.0.md` | AUTHORITATIVE | `4EB8B5B00B85BF17C1B040D82A9E0FD0413D71A96668A5FB5556425E41C1A4C9` |
| 6 | `Nuwa_NWB_Implementation_Plan_v1.0.md` | AUTHORITATIVE | `5CF82052C072C0BE7CDA5344C517A3187E96442D1BD9C8085786D1133E29323A` |
| 7 | `Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md` | AUTHORITATIVE | `EF44291ECC9028D663D1C5ACFC226F830A6837F79AE53C7C5D9B884D7A08ECC9` |
| 8 | `Nuwa_NWB_Test_Result_Record_v1.1.md` | REFERENCE / CURRENT | `44ABB02A57CC417BD55BB335A3021589DFE7308B59BB76865CB6F2BD0D6FC496` |
| 9 | `IMP-000_EVD_Build_Evidence_v1.0.md` | REFERENCE / CURRENT FOR IMP-000 | `434F4107FB6A970A80C0A4DF4B9180048C3766EA74B4E3C4F1927F04A0A3E5D7` |
| 10 | `IMP-001_EVD_Test_Result_Evidence_v1.2.md` | REFERENCE / CURRENT FOR IMP-001 | `2A53BDB7750A3239AF8972C746F16154307DE488C7F3892AD8E6129B463F76B9` |
| 11 | `Nuwa_NWB_Test_Result_Record_v1.0.md` | HISTORICAL / SUPERSEDED / MALFORMED_SOURCE_RETAINED_FOR_TRACEABILITY | `854F5F7A51C226B8177FA57703AE8895D117294BAB3C4AF0AD2B657E99C1CCCE` |
| 12 | `IMP-001_EVD_Test_Result_Evidence_v1.0.md` | HISTORICAL / SUPERSEDED / MALFORMED_SOURCE_RETAINED_FOR_TRACEABILITY | `4CAC2A7802666EA45B81D1A63DDCA220CE0729F180F95DA86F295E2A00AB8396` |

## Verification Notes

- All 12 listed files exist; the manifest itself is excluded.
- Current documents use UTF-8 with LF and contain no prohibited control bytes.
- Historical files #11 and #12 are deliberately retained byte-for-byte. Their hashes certify the retained malformed sources, not current project truth.
- Previous manifest entries mixed line-ending conventions and contained stale hashes. This revision standardizes the byte definition and refreshes every entry.
- Implementation evidence remains bound to commit `3bceb34b4697a7552bccf9863b821a5b9d63e4b4` and CI run `29684903853`; the later docs-only evidence commit is not represented as the tested implementation.
