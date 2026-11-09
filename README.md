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

### 1. リリースバイナリのダウンロード（推奨）

[GitHub Releases](https://github.com/asa321/devclean/releases) からお使いのOSに応じたアーカイブをダウンロードして解凍し、PATHの通ったディレクトリへ配置してください。

#### Windows
1. `devclean-windows-x86_64.zip` をダウンロードして解凍します。
2. `devclean.exe` を任意のフォルダ（例: `C:\Tools\devclean`）に配置します。
3. ユーザー環境変数の `PATH` に上記フォルダを追加します。
   - **PowerShellでの一時追加（現在のセッションのみ）**:
     ```powershell
     $env:PATH += ";C:\Tools\devclean"
     ```
   - **恒久的な追加（ユーザー環境変数）**:
     ```powershell
     [Environment]::SetEnvironmentVariable(
       "Path",
       [Environment]::GetEnvironmentVariable("Path", "User") + ";C:\Tools\devclean",
       "User"
     )
     ```
4. 新しいターミナルを開き、`devclean --version` が実行できることを確認します。

#### macOS
1. アーカイブをダウンロードして解凍します。
   - Apple Silicon (M1/M2/M3等): `devclean-macos-arm64.tar.gz`
   - Intel Mac: `devclean-macos-x86_64.tar.gz`
2. 解凍した `devclean` バイナリを `/usr/local/bin`（または `~/.local/bin`）へ移動し、実行権限を付与します。
   ```bash
   tar -xzvf devclean-macos-arm64.tar.gz
   chmod +x devclean
   sudo mv devclean /usr/local/bin/
   ```
3. 初回実行時に開発元未確認の警告が出る場合は、以下を実行して隔離属性を解除してください。
   ```bash
   xattr -d com.apple.quarantine /usr/local/bin/devclean
   ```
4. `devclean --version` で動作確認します。

#### Linux
1. `devclean-linux-x86_64.tar.gz` をダウンロードして解凍します。
2. 解凍した `devclean` バイナリを `/usr/local/bin`（または `~/.local/bin`）へ配置します。
   ```bash
   tar -xzvf devclean-linux-x86_64.tar.gz
   chmod +x devclean
   sudo mv devclean /usr/local/bin/
   ```
   ※ `~/.local/bin` に配置する場合は、`~/.bashrc` や `~/.zshrc` に `export PATH="$HOME/.local/bin:$PATH"` を追加してください。
3. `devclean --version` で動作確認します。

---

### 2. ソースコードからビルド

#### 必要要件
- Rust 1.75+ (stable)
- Git (CLIがPATHに通っていること)

```bash
# クローンとビルド
git clone https://github.com/asa321/devclean.git
cd devclean
cargo build --release
```

ビルド完了後、実行バイナリは `target/release/devclean`（Windowsは `target/release/devclean.exe`）に出力されます。上記と同様にPATHの通ったディレクトリへコピーしてご使用ください。

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

### 5. セルフアップデート (--check-update / --update)
GitHub Releases の最新バージョンを確認し、実行中バイナリを自動更新できます。

```bash
# 最新バージョンの確認のみ
devclean --check-update

# 最新バージョンへアップデート（対話確認あり）
devclean --update

# 対話確認を省略してアップデート
devclean --update --yes
```

※ Private リポジトリの場合は、環境変数 `GITHUB_TOKEN` または `DEV_CLEAN_GITHUB_TOKEN` に Personal Access Token を設定してください。

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
| `--check-update` | GitHub Releases の最新バージョンを確認 |
| `--update` | 最新バージョンへのセルフアップデートを実行 |
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
