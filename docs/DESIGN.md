# 設計仕様書 (DESIGN.md)

## 1. 全体アーキテクチャ

devcleanはステートレスなCLIツールとして設計されており、ローカルに独自の設定ファイルやデータベースを永続化しません。

### パイプライン構成
1. **CLI解析 & 安全ガード (`cli.rs`)**:
   - 引数の整合性チェックと排他制約（`--clean` と `--dry-run`、`--allow-non-git` と `--yes` 等）のバリデーション。
2. **プロジェクト検出 (`scanner/project.rs`)**:
   - `.git` ディレクトリまたは `.gitignore` を探索し、判定対象のプロジェクト境界を特定。
3. **Git判定 & Trackedインデックス化 (`git/`)**:
   - `git ls-files -z` によりリポジトリ内の全インデックス登録済みパスを一括取得し、パスコンポーネント境界を考慮した `TrackedPathIndex` (Trie/Set) に格納。
   - `git check-ignore --no-index -v -z --stdin` により、候補パス群をGit本体でバッチ判定。
4. **安全判定エンジン (`analysis/safety.rs`)**:
   - 機密ファイル（深さ0〜2の絶対保護）、システム保護領域、再解析ポイント、ネストしたGitリポジトリを多層防御で除外。
5. **削除 & TOCTOU検証 (`cleaner/`)**:
   - 削除直前にファイル種別・更新・Tracked再照合を実施。
   - OSネイティブのゴミ箱機能（`trash` クレート）を呼び出し。

---

## 2. クロスプラットフォーム・パス設計

JSON出力では RFC 8259 準拠のUTF-8を保証しつつ、OSネイティブの生パス表現を Base64 化して提供します。

```json
{
  "path_display": "C:\\work\\test\\日本語 フォルダ 🚀",
  "path_encoding": "windows_utf16le",
  "path_raw_base64": "QwA6AFwAdwBvAHIAawBcAHQAZQBzAHQAXADml6XmlKzoqp4AIABmAG8AbABkAGUAcgAgAP3n/0AA"
}
```

- Windows: UTF-16LE コード単位列（リトルエンディアン）
- Unix: OSの生バイト列
