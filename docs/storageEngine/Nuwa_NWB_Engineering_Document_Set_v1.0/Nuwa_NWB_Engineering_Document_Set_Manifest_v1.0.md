# Nüwa NWB Engineering Document Set Manifest v1.0

**Document Set:** Nuwa_NWB_Engineering_Document_Set_v1.0
**Manifest generated:** 2026-07-16
**Task:** IMP-001 evidence remediation (Generator Remediation reopen)

---

## Current Authenticity Statement

| Item | Status |
|------|--------|
| GATE-0 | IN_PROGRESS |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET |
| IMP-001 | CLOSED (v1.0, d550907) -> REOPENED / IN_PROGRESS (Generator Remediation 2026-07-18) |
| IMP-002 | NOT_RUN |

> **Note:** This order is a checklist, not a new authority order. The authoritative document ordering is defined by the Document Set README (§2).

---

## Document Inventory

Total documents listed: **10** (this manifest file is excluded)

| # | File | SHA-256 |
|---|------|---------|
| 1 | Nuwa_NWB_Engineering_Document_Set_README_v1.0.md | BCC9BB782838497CDFCA4352407EAF600AFDA146031513D279B4A1BB949F8280 |
| 2 | Nuwa_NWB_Storage_Engine_Architecture_v2.0.md | 3996D95C3B25C06F7CE357230AFCED6A5FA37C9B31F11C6CC06FED7DFEA863C6 |
| 3 | Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | 57C0ADB457EC4E06EEA857B1058C2331FF3758433140B26FA6EB6028F8590660 |
| 4 | Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md | B9F6FF0B375FAA2B40C16BAD9F4FA1162DF2E4B0B8BE9303C1CCF153C0409903 |
| 5 | Nuwa_NWB_Product_Support_Matrix_v1.0.md | 4EB8B5B00B85BF17C1B040D82A9E0FD0413D71A96668A5FB5556425E41C1A4C9 |
| 6 | Nuwa_NWB_Implementation_Plan_v1.0.md | 5CF82052C072C0BE7CDA5344C517A3187E96442D1BD9C8085786D1133E29323A |
| 7 | Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md | EF44291ECC9028D663D1C5ACFC226F830A6837F79AE53C7C5D9B884D7A08ECC9 |
| 8 | Nuwa_NWB_Test_Result_Record_v1.0.md | 854F5F7A51C226B8177FA57703AE8895D117294BAB3C4AF0AD2B657E99C1CCCE |
| 9 | IMP-000_EVD_Build_Evidence_v1.0.md | 63F3B3741BB52E8FAA2E7E37F426E56B4C476E60DA0B35B49EC93DD956A0742D |
| 10 | IMP-001_EVD_Test_Result_Evidence_v1.0.md | 42BC7DE44779DF47C40442DB5CC9202802FBCEFF46A4FFFC9467830B99774758 |

All SHA-256 values computed via PowerShell Get-FileHash -Algorithm SHA256 from working tree files (CRLF line endings).

---

## Verification

Self-check: All 10 files present. No duplicates. No missing entries.

### Files not modified in this update

The following files are unchanged since the previous manifest (IMP-000-EVIDENCE-CLOSURE-1):
- #2 (Architecture), #3 (Binary Format Spec), #4 (Provider SDK), #5 (Support Matrix), #6 (Implementation Plan), #7 (Verification Plan), #9 (IMP-000 EVD)
- Their SHA-256 values differ from the previous manifest only because the previous manifest used Git index blob content (LF line endings), while this update uses working tree files (CRLF line endings) per task instructions.
- Content-unchanged verification: git hash-object for #9 (IMP-000 EVD) = 49ef38e, matching the committed working tree content.

### Files updated in this task

| # | File | Change |
|---|------|--------|
| 1 | Nuwa_NWB_Engineering_Document_Set_README_v1.0.md | §11 authenticity statement updated: IMP-001 state changed to CLOSED |
| 8 | Nuwa_NWB_Test_Result_Record_v1.0.md | 854F5F7A51C226B8177FA57703AE8895D117294BAB3C4AF0AD2B657E99C1CCCE revision: commit, CI evidence, test results updated for IMP-001 closure |
| 10 | IMP-001_EVD_Test_Result_Evidence_v1.0.md | 42BC7DE44779DF47C40442DB5CC9202802FBCEFF46A4FFFC9467830B99774758 revision: status changed from IN_PROGRESS/ACCEPTANCE NOT MET to CLOSED/PASS/ACCEPTANCE MET; ErrorId, trybuild, CI evidence, review history, governance deviation recorded |