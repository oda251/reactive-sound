# Reactive BGM

キーボード入力に応じてリアルタイムにBGMが変化するバックグラウンド常駐型アプリ。
音声エンジンとインターフェースを完全分離し、egui/Unity 等の任意のフロントエンドから利用可能。

## 技術スタック

- **Glicol**: シーケンシング・ライブコーディングエンジン (Rust)
- **Faust → Rust**: DSP・シンセサイザー・エフェクト（ビルド時にRustコード生成）
- **cpal**: クロスプラットフォームオーディオ出力
- **egui**: GUI フレームワーク
- **rdev**: グローバルキーボードキャプチャ
- **rosc / tungstenite**: OSC / WebSocket サーバー

## クレート構成

- `reactive-bgm-engine` (lib + cdylib): コアエンジン。Glicol + Faust DSP + cpal
- `reactive-bgm-server` (bin): OSC/WebSocket サーバー
- `reactive-bgm-app` (bin): egui デスクトップアプリ

## ドキュメント

設計ドキュメントは `docs/` 配下に documentation スキルのフォーマットで管理する。
