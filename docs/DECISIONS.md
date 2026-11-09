# 設計判断記録 (DECISIONS.md)

## DEC-001: Git CLI 入出力プロトコルと Tracked 保護の分離

- **ステータス**: 承認・実証済み (Phase 1 Spike PASS)
- **背景**:
  - `git check-ignore --no-index` を使用すると、インデックス登録の有無に関係なく ignore ルールとの適合判定が行われる。
  - 実験結果（`target/forced.txt`）の通り、`git add -f` でインデックスに追加されたファイルであっても、`git check-ignore --no-index` は ignore パターン（`target/`）にマッチしたとして結果を返す。
- **決定事項**:
  - **Tracked ファイルの保護根拠は `git ls-files -z` の結果のみ**とする。
  - `git check-ignore --no-index` の結果で ignore 対象と報告されても、`git ls-files` のインデックス内に存在するパスは削除候補から完全に除外する。
  - 出力パース仕様:
    - `git ls-files -z`: NUL 区切りでパスが連続出力される。
    - `git check-ignore -v -z --stdin`: マッチごとに `<source> \0 <lineno> \0 <pattern> \0 <pathname> \0` の4トークンが出力される。否定パターン（`!`）でマッチした場合も出力されるため、`pattern` の先頭が `!` の場合は「ignore されていない（対象外）」として扱う。

## DEC-002: CLI 対象パスの引数仕様

- **ステータス**: 承認
- **決定事項**:
  - 第1引数の位置引数 `[PATH]` を正式仕様とする（未指定時はカレントディレクトリ `.`）。
  - `-p, --path <PATH>` も互換性のために受付可能とし、両方指定された場合は位置引数を優先するか整合性をチェックする。

## DEC-003: 走査エンジンの選定 (`ignore` vs `jwalk`)

- **ステータス**: 承認
- **決定事項**:
  - ファイル候補の走査補助として `ignore` クレートの `WalkBuilder` を採用する。
  - `ignore` クレート側の Git 判定のみに依存せず、候補列挙後の確定判定はすべて Git CLI（`git ls-files` と `git check-ignore`）に委ねる。
  - シンボリックリンク・ジャンクションの自動追従は無効（`follow_links(false)`）を強制する。

## DEC-004: Windows 再解析ポイント・ジャンクションとゴミ箱移動

- **ステータス**: 承認
- **決定事項**:
  - `trash` クレート (v5.3) の Windows 実装は `IFileOperation` を使用しており、単体の移動は安全に実行できる。
  - ディレクトリ集約に関しては、配下にジャンクションやシンボリックリンクが存在する場合、リンク先の実体が誤って巻き込まれるリスクを完全に排除するため、リンクを含むディレクトリの集約は禁止し、個別走査・判定にフォールバックする。
