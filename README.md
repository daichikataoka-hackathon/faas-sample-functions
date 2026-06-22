# faas-sample-functions

オリジナル FaaS 基盤の「GitHub から取り込み」機能で登録できるサンプル関数集。

各関数は WASI command の規約（**stdin で入力を受け取り、stdout へ出力**）に従い、
入力を **大文字化して返す**最小実装。外部依存は持たないため offline ビルドが通る。

## 言語別ディレクトリ

| 言語 | ディレクトリ | manifest | 対応 runtime |
| --- | --- | --- | --- |
| Rust | `rust/` | `Cargo.toml` | barewasm (`wasm32-wasip1`) / securewasm (`wasm32-wasip2`) |
| Go (TinyGo) | `go/` | `go.mod` | barewasm |
| C | `c/` | `wasi.build.json` | barewasm |

## 取り込み URL（コンソールの GitHub 取り込みに貼る）

サブディレクトリ単位で取り込むと、manifest のある階層が自動でプロジェクト root に再マップされる。

- Rust: `https://github.com/daichikataoka-hackathon/faas-sample-functions/tree/main/rust`
- Go:   `https://github.com/daichikataoka-hackathon/faas-sample-functions/tree/main/go`
- C:    `https://github.com/daichikataoka-hackathon/faas-sample-functions/tree/main/c`

## 動作

```
入力:  hello faas
出力:  HELLO FAAS
```
