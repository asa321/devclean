# devclean

開発プロジェクト内の不要な生成物・キャッシュ・一時ファイルを安全に検出し、OSのゴミ箱へ移動するクロスプラットフォームCLIツールです。

## 特徴

- **絶対的な安全性重視**:
  - Gitで管理されているファイル（Trackedファイル）は絶対に削除しません（`git add -f` による強制登録も含む）。
  - GitのIgnoreルール判定はGit本体（CLI）を正とし、ネストした `.gitignore` や否定パターン（`!`）も正確に扱います。
  - 通常の削除はOSのゴミ箱（Trash / Recycle Bin）へ移動し、復元可能です。
  - ゴミ箱移動に失敗した場合でも、完全削除への自動フォールバックは行いません。
  - プロジェクト直下（深さ0〜2）の `.env` や鍵ファイルなどの機密ファイルは強制保護されます。
  - シンボリックリンクやジャンクションのリンク先は絶対に追従・削除しません。
- **適応型走査と安全な集約**:
  - `node_modules/` や `target/` などの生成物ディレクトリ全体を集約移動する前に、内部にTrackedファイルや機密ファイル、リンクが含まれていないかを徹底検証します。
- **クロスプラットフォーム対応**:
  - Windows (NTFS / ジャンクション / UTF-16LE)、macOS、Linux (ext4 / 生バイト列) に完全対応。

---

## インストール & ビルド

### 必要要件
- Rust 1.75+ (stable)
- Git (CLIがPATHに通っていること)

```bash
# クローンとビルド
git clone https://github.com/your-org/devclean.git
cd devclean
cargo build --release
```

バイナリは `target/release/devclean` に生成されます。

---

## 使い方

### 1. 検出と確認 (Dry-Run: デフォルト動作)
実際の削除を行わず、候補や計画を確認します。

```bash
# カレントディレクトリを走査
devclean

# パスを指定して走査
devclean /path/to/projects

# 詳細表示
devclean -v
```

### 2. 集計のみ (--scan)
削除計画の詳細を出さず、検出と集計のみを行います。

```bash
devclean --scan
```

### 3. ゴミ箱へ移動 (--clean)
安全条件を満たした不要ファイルをOSのゴミ箱へ移動します。

```bash
# 対話確認付きで実行 (y または yes の入力が必要)
devclean --clean

# 対話確認を省略して実行
devclean --clean --yes
```

### 4. JSON形式での出力 (--format json)
外部ツールやCIと連携可能な RFC 8259 準拠のJSONを出力します。

```bash
devclean --format json
```

※ `--format json --clean` で実行する場合は、安全ガードのため `--yes` が必須となります。

---

## CLIオプション一覧

| オプション | 説明 |
|---|---|
| `[PATH]` | 対象ルートパス（省略時はカレントディレクトリ） |
| `-p, --path <PATH>` | 対象ルートパス（エイリアス） |
| `-r, --recursive` | 再帰走査（デフォルト有効） |
| `--depth <N>` | 最大走査深度 |
| `--git-only` | Gitリポジトリのみを対象とする |
| `--scan` | 検出・集計のみ |
| `--clean` | ゴミ箱移動を実行 |
| `--dry-run` | 実行せず計画を表示（デフォルト） |
| `-y, --yes` | 対話確認を省略 |
| `--allow-non-git` | Git管理外のプロジェクトの削除を許可（対話必須・`--yes`併用不可） |
| `--include-sensitive` | 生成物ディレクトリ深層の機密パターン保護を緩和 |
| `--include <PATTERN>` | 追加候補パターン |
| `--exclude <PATTERN>` | 除外パターン（`--include` より優先） |
| `--min-size <SIZE>` | 最小ファイルサイズ（例: 10MB, 1GB） |
| `--max-size <SIZE>` | 最大ファイルサイズ |
| `--max-operations <N>` | 実行操作数の上限（超過時はclean全体を中止） |
| `--format <FORMAT>` | `text` または `json` |
| `-v, --verbose` | 詳細表示 |
| `-q, --quiet` | 最小限の表示 |

---

## 終了コード

| コード | 意味 |
|---|---|
| `0` | 正常終了・候補なし |
| `1` | 一部スキップ（TOCTOU不整合、アクセス拒否等） |
| `2` | CLI引数エラー（矛盾するフラグ指定） |
| `3` | Ctrl+C による中断 |
| `4` | 安全ガードによる実行拒否（非TTY、安全上限超過等） |
| `5` | 致命的エラー（Gitタイムアウト、I/O障害等） |

---

## ライセンス

MIT License
