# Reactive BGM

キーボード入力に応じてリアルタイムにBGMが変化するバックグラウンド常駐型デスクトップアプリ。

## アーキテクチャ

3層構成で、各層の境界は型で定義されており、差し替え可能。

```
[Input Adapters]          [ScoreProvider]               [Engine]
                          (trait: 差し替え可能)
rdev  ──┐                ┌──────────────────┐
        ├── InputEvent ──│ on_event()       │── Score ──> Scheduler ──> Faust DSP ──> cpal
egui  ──┘                │ score() -> Score │            (playhead を atomic 共有 → GUI)
                         └──────────────────┘
                         実装: RawRhythmProvider
```

### 型安全な境界

| 型 | 役割 | 定義場所 |
|----|------|---------|
| `InputEvent` | Input → ScoreProvider の境界。入力ソース非依存 | `engine/src/core/event.rs` |
| `Score` | ScoreProvider → Engine の境界。NoteEvent のリスト + ループ長 | `engine/src/core/scheduler.rs` |
| `ScoreProvider` | Record + Interpret を束ねたトレイト。差し替え単位 | `engine/src/core/score_provider.rs` |

### Engine 内部 (FC/IS)

```
engine/
├── core/                  Functional Core（純粋・テスト可能）
│   ├── scheduler.rs       再生位置管理 + NoteOn/Off イベント生成
│   ├── dsp.rs             Faust シンセラッパー
│   ├── score_provider.rs  ScoreProvider トレイト定義
│   ├── event.rs           InputEvent 定義
│   └── config.rs          EngineConfig（環境変数対応）
└── shell/                 Imperative Shell（副作用・I/O）
    ├── audio.rs           cpal デバイス管理・ストリーム構築
    ├── bridge.rs          Scheduler → Faust → cpal（リングバッファ）
    └── command.rs         Engine ↔ 音声スレッド間の Command enum
```

## 技術スタック

| レイヤー | 技術 | 役割 |
|---------|------|------|
| DSP/シンセ | [Faust](https://faust.grame.fr/) → Rust | ビルド時に `.dsp` → `.rs` 生成 |
| スケジューラ | 自前 (Rust) | 再生位置管理、NoteOn/Off 発火 |
| オーディオ出力 | [cpal](https://github.com/RustAudio/cpal) | クロスプラットフォーム |
| GUI | [egui](https://github.com/emilk/egui) | パラメータ表示・リズムグリッド |
| 入力検知 | [rdev](https://github.com/Narsil/rdev) | グローバルキーボードキャプチャ |

## クレート構成

```
reactive-bgm/
├── engine/   — コアオーディオエンジン (lib + cdylib)
├── app/      — デスクトップアプリ (bin)
│   ├── input.rs                Input アダプター (rdev, egui)
│   └── raw_rhythm_provider.rs  ScoreProvider 実装
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
- [x] 自前スケジューラ（playhead を GUI と atomic 共有）
- [x] キーボード入力 → リズムパターン → 音声出力
- [x] egui GUI（リズムグリッド + playhead 表示）
- [x] ScoreProvider → InputEffect トレイト階層（ImmediateEffect / AccumulativeEffect）
- [x] 8 ボイスポリフォニー + VoiceAllocator
- [x] VoiceType（複数シンセルーティング基盤）
- [x] tick ベーススケジューラ（480 ticks/beat、パターン + イベントキュー）
- [x] ParamValue 共有型（NoteEvent.overrides + ParamEvent）
- [ ] 複数 Faust DSP（VoiceType ごとに別シンセ）
- [ ] システムトレイ常駐
- [ ] 複数の AccumulativeEffect 実装（WPM ティア、メロディ変換等）

### 拡張予定

アーキテクチャ上対応可能な設計になっている。

- **C ABI (cdylib)**: engine を共有ライブラリとしてビルドし、Unity (C# P/Invoke) 等から利用
- **OSC / WebSocket サーバー**: `server` クレートで engine をネットワーク経由で制御
- **追加 Faust DSP**: `engine/dsp/` に `.dsp` ファイルを追加し build.rs で自動コンパイル。VoiceType でルーティング
- **MIDI コントローラー対応**: InputEvent に MidiNote バリアントを追加、新しい InputAdapter を実装
- **サンプル再生**: Faust の soundfile プリミティブ、または Rust 側で wav 読み込み + バッファ管理
- **リアルタイム録音**: cpal マイク入力 → バッファ → サンプラー
- **プリセット管理**: PatternSlot + DSP パラメータのシリアライズ/デシリアライズ

## ライセンス

- **Faust コンパイラ**: LGPL（生成コードには適用されない）
- **cpal**: Apache-2.0
