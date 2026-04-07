# Reactive BGM

キーボード入力に応じてリアルタイムにBGMが変化するバックグラウンド常駐型アプリ。

## アーキテクチャ

```
[Input Adapters]          [ScoreProvider]              [Engine]
                          (trait: 差し替え可能)
rdev  ──┐                ┌──────────────────┐
        ├── InputEvent ──│ on_event()       │── Score ──> Scheduler ──> Faust DSP ──> cpal
egui  ──┘                │ score() -> Score │            (playhead を atomic 共有)
                         └──────────────────┘
                         実装: RawRhythmProvider
```

### 型安全な境界

- `InputEvent` — Input → ScoreProvider の境界。入力ソース非依存。
- `Score` — ScoreProvider → Engine の境界。NoteEvent のリスト + ループ長。
- `ScoreProvider` trait — Record + Interpret を束ねた差し替え単位。

### FC/IS 構成 (engine クレート)

- `core/` — Functional Core: Scheduler, DspProcessor, EngineConfig, Score, ScoreProvider trait
- `shell/` — Imperative Shell: cpal 出力, Bridge (Scheduler→Faust→cpal), Command channel

## 技術スタック

- **Faust → Rust**: DSP・シンセサイザー（ビルド時に .dsp → .rs 生成）
- **cpal**: クロスプラットフォームオーディオ出力
- **自前 Scheduler**: 再生位置管理 + NoteOn/Off イベント生成
- **egui**: GUI
- **rdev**: グローバルキーボードキャプチャ

## クレート構成

- `engine/` (lib + cdylib): コアオーディオエンジン
- `app/` (bin): egui デスクトップアプリ + 入力アダプター + ScoreProvider 実装
- `server/` (bin): OSC/WebSocket サーバー [予定]

## ドキュメント

設計ドキュメントは `docs/` 配下に documentation スキルのフォーマットで管理する。
