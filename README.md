# Reactive BGM

キーボード入力に応じてリアルタイムにBGMが変化するバックグラウンド常駐型デスクトップアプリ。

## 概要

タイピング速度や入力パターンに応じて、BGMのテンポ・音色・パターンがリアルタイムに変化する。音声エンジンとインターフェースは完全に分離されており、デスクトップアプリ・ゲームエンジン・外部ツール等から利用可能。

## 技術スタック

| レイヤー | 技術 | 役割 |
|---------|------|------|
| シーケンシング | [Glicol](https://glicol.org/) | ライブコーディングエンジン（Rust） |
| DSP/シンセ | [Faust](https://faust.grame.fr/) → Rust | ビルド時にRustコード生成 |
| オーディオ出力 | [cpal](https://github.com/RustAudio/cpal) | クロスプラットフォーム |
| GUI | egui | パラメータ表示・操作 |
| 入力検知 | rdev | グローバルキーボードキャプチャ |

## クレート構成

```
reactive-bgm/
├── engine/   — コアオーディオエンジン (lib + cdylib)
│   ├── core/     Functional Core: パターン評価、DSP、設定
│   └── shell/    Imperative Shell: オーディオI/O、コマンドブリッジ
├── app/      — デスクトップアプリ (bin)
└── server/   — OSC/WebSocket サーバー (bin) [予定]
```

## ビルド・実行

```bash
# 前提: Faust コンパイラが必要
# macOS/Linux: brew install faust
# Windows: https://faust.grame.fr/downloads/

# Linux (WSL2) でビルド → Windows で実行
cargo build -p reactive-bgm-app --target x86_64-pc-windows-gnu
./target/x86_64-pc-windows-gnu/debug/reactive-bgm-app.exe

# テスト
cargo test -p reactive-bgm-engine
```

### 環境変数

| 変数 | デフォルト | 説明 |
|------|-----------|------|
| `RBGM_SAMPLE_RATE` | デバイス自動検出 | サンプルレート (Hz) |
| `RBGM_DEVICE` | デフォルトデバイス | オーディオデバイス名（部分一致） |
| `RUST_LOG` | (なし) | ログレベル (`error`, `warn`, `info`, `debug`) |

## ロードマップ

- [x] Faust DSP シンセ統合
- [x] Glicol シーケンサー統合
- [x] Glicol + Faust ミキシング
- [ ] キーボード入力 → パラメータ変換
- [ ] egui GUI（パラメータ表示・スライダー操作）
- [ ] システムトレイ常駐

### 拡張予定

以下は現時点では未実装だが、アーキテクチャ上対応可能な設計になっている。

- **C ABI (cdylib)**: engine を共有ライブラリとしてビルドし、Unity (C# P/Invoke) 等の外部エンジンから利用
- **OSC / WebSocket サーバー**: `server` クレートで engine をネットワーク経由で制御。TouchOSC、SC、ゲームエンジン等から接続
- **追加 Faust DSP**: `engine/dsp/` に `.dsp` ファイルを追加し build.rs で自動コンパイル。複数シンセの切り替え・レイヤー
- **MIDI コントローラー対応**
- **プリセット管理**: パターン + DSP パラメータのセットを保存・切り替え

## ライセンス

- **Glicol**: MIT
- **Faust コンパイラ**: LGPL（生成コードには適用されない）
- **cpal**: Apache-2.0
