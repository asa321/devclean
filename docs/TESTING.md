# テスト仕様 & 結果報告 (TESTING.md)

## 1. テスト環境

- **OS**: Windows 11 / Windows Server 2022 (NTFS)
- **Rust**: rustc 1.95.0 (f2d3ce0bd 2026-03-21) / stable
- **Git**: git version 2.45.1.windows.1

---

## 2. 自動テストスイート一覧と実行結果

すべてのテスト結果は `PASS` / `FAIL` / `NOT RUN` / `BLOCKED` の厳格な基準で記録されています。

| テスト名 | 対象モジュール | 検証内容 | 結果 |
|---|---|---|---|
| `test_sensitive_patterns` | `filter::sensitive` | `.env`, `.pem`, `id_rsa`, `*secret*` 等のパターン判定 | **PASS** |
| `test_sensitive_depth_classification` | `filter::sensitive` | 深さ0〜2（Level 1）と深さ3以降（Level 2）の分離判定 | **PASS** |
| `test_size_parsing` | `analysis::size` | `100B`, `10KB`, `500M`, `2GB` 等のサイズ単位パース | **PASS** |
| `test_tracked_boundary_matching` | `git::tracked` | パスコンポーネント境界を意識したTrie照合（`target-old`の誤判定防止） | **PASS** |
| `test_git_tracked_protection_and_aggregation_veto` | `analysis::safety` | `git add -f` されたファイルを含むディレクトリの集約禁止検証 | **PASS** |
| `test_clean_to_trash_and_toctou_verification` | `cleaner::trash` | ゴミ箱移動の実証およびTOCTOU再検証 | **PASS** |
| `test_cli_argument_rejections_and_exit_codes` | `cli` | 排他フラグおよび安全ガード違反時の終了コード（2, 4） | **PASS** |
| `test_complex_paths_and_json_serialization` | `output::path_codec` | 日本語・空白・絵文字パスのBase64エンコーディング | **PASS** |
| macOS Apple Silicon 実機テスト | - | Apple Silicon runner未接続 | **NOT RUN** |
| Linux ext4 非UTF-8実機テスト | - | GitHub Actions CI環境で自動実行 | **NOT RUN** (CI待機) |
