# faas-sample-functions

オリジナル FaaS 基盤の **「GitHub から取り込み」** 機能で登録できるサンプル関数集。
対応言語（Rust / Go / C）ごとにディレクトリを分け、各言語に複数のサンプル関数を置いている。

## 関数の規約（共通）

全サンプルは **WASI command** の規約に従う最小実装。

- **入力**: 標準入力（stdin）から受け取る
- **出力**: 標準出力（stdout）へ書く（エラーは stderr）
- **終了コード**: 成功 `0` / 失敗 非 `0`
- **依存ライブラリなし**（標準ライブラリのみ）＝ ビルドワーカーの offline ビルドが通る

## 取り込み単位は「関数ディレクトリ」

取り込み機能は URL 先のファイルをブラウザから GitHub API で取得し、**manifest があるディレクトリを
プロジェクト root に再マップ**してから言語を判定する。

| manifest | 判定される言語 |
| --- | --- |
| `Cargo.toml` | Rust |
| `go.mod` | Go |
| `wasi.build.json` | C / C++ |

そのため取り込み URL には **関数 1 個分のディレクトリ（manifest がある階層）** を指定する。

- ⚠️ **言語ディレクトリ（`/rust`）やリポジトリ root を指定しない**こと。複数の manifest が混ざると
  最初の 1 個だけが採用され、他のサンプルが取り込まれない。
- 取り込み上限: 50 ファイル / 合計 1MB / 1 ファイル 512KB（本サンプルはいずれも十分小さい）。

## サンプル一覧

| 関数 | 言語 | 取り込み URL（`/tree/main/...`） | 動作 |
| --- | --- | --- | --- |
| echo | Rust | `rust/echo` | 入力をそのまま返す |
| uppercase | Rust | `rust/uppercase` | 入力（UTF-8）を大文字化 |
| rot13 | Rust | `rust/rot13` | ASCII 英字を ROT13 変換 |
| reverse | Go (TinyGo) | `go/reverse` | 入力を rune 単位で逆順に |
| wordcount | C | `c/wordcount` | 行数・単語数・バイト数を `lines words bytes` 形式で出力 |
| vibration-fft | Rust | `rust/vibration-fft` | 振動波形を FFT 解析し異常判定（EdgeFaaS デモ・CPU バウンド） |

### vibration-fft（EdgeFaaS デモ用・現場前処理）

予知保全の現場前処理を模した CPU バウンド関数。`ingress(大きい生波形) → runtime(FFT 解析) → egress(小さな結果)` を体現する。

- **入力**: 加速度の生波形（comma/空白区切りの float・例 4096 点 ≒ 32KB）
- **処理**: 位相シフトした窓を `channels` 個 FFT し、RMS / peak / 主要周波数 / peakiness（突出度）を算出。最も peaky な窓で異常判定
- **出力**: `{"n":..,"channels":..,"rms":..,"peak":..,"dom_hz":..,"peakiness":..,"anomaly":..,"anomaly_channels":..}`（~110B）
- **データ削減**: 32KB → ~110B ≈ 300x（ingress/egress 分離で現場前処理＝送信量削減）
- **パラメータ**（`fs`=サンプルレート Hz・既定 1000 / `channels`=解析窓・軸数・既定 1）は 2 通りで渡せる。**CPU は channels × O(n log n)** で線形に増えるので、入力を増やさず 1 呼び出しの負荷を校正できる（ランタイム限界の量試験用）:
  - **argv**: `<fs> <channels>`（HTTP sync/async は argv を運ぶ）
  - **stdin ヘッダ**: 波形の前に `=` を含む 1 行を置く（例 `fs=1000 channels=256`・先頭 `#` 可・`ch=` エイリアス可）。**MQTT は payload=stdin のみで argv を運ばない**ため、MQTT で負荷を校正する唯一の方法
  - 優先順位: 既定 < stdin ヘッダ < argv（argv があれば優先）
- 入力例（stdin ヘッダ付き・MQTT でも有効）:
  ```
  fs=1000 channels=256
  0.012,0.034,-0.021, ... (波形)
  ```
- 例: 異常波形（58.6Hz 強周期）→ `dom_hz≈58.6, peakiness≫8, anomaly=true` / 広帯域ノイズ → `anomaly=false`

取り込み URL の例（uppercase）:

```
https://github.com/daichikataoka-hackathon/faas-sample-functions/tree/main/rust/uppercase
```

## runtime（barewasm / securewasm）の選び方

runtime は **登録ウィザードで選択する**項目で、ソースには現れない。同じソースが両 runtime に
ビルドされるため、**どのサンプルも barewasm / securewasm の両方で登録できる**。

| runtime | ビルドターゲット | 実行形態 |
| --- | --- | --- |
| barewasm | `wasm32-wasip1`（コア module） | WASI Preview 1 |
| securewasm | `wasm32-wasip2`（component を生成 → AOT 事前コンパイル） | Hyperlight microVM |

→ 「Secure 用」に別のソースは不要。取り込み後に runtime で **securewasm** を選べば、
同じソースが wasip2 component としてビルド・AOT され、Hyperlight 上で実行される。

## 動作例

```
echo:       hello faas     -> hello faas
uppercase:  hello faas     -> HELLO FAAS
rot13:      hello faas     -> uryyb snnf
reverse:    hello faas     -> saaf olleh
wordcount:  "a b c\n"      -> 1 3 6
```
