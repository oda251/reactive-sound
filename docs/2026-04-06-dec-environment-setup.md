---
tags:
  - decision
  - environment
  - wsl2
  - supercollider
  - tidalcycles
---
# 環境構築

depends-on:
- [技術スタック選定](./2026-04-05-dec-tech-stack.md)

## セットアップ

自動チェックスクリプトで依存関係を確認・インストールできる:

```bash
./scripts/setup.sh
```

以下は各コンポーネントの詳細。

## 動作確認済み環境

- **OS**: Windows 11 + WSL2 (Ubuntu 24.04)
- **WSL**: v2.6.3, `networkingMode=mirrored`
- **SuperCollider**: 3.14.1 (Windows ネイティブ, winget)
- **TidalCycles**: 1.10.1 (WSL2, cabal install --lib)
- **GHC**: 9.14.1 (WSL2, mise + ghcup)
- **Rust**: stable (WSL2, mise)

## アーキテクチャ上の制約

SuperCollider は Windows ネイティブで動作させる必要がある（WSL2 にはオーディオデバイスがないため）。TidalCycles と Rust アプリは WSL2 上で動作する。

```
WSL2                          Windows
┌──────────────────┐          ┌──────────────────┐
│ Rust App         │          │                  │
│ TidalCycles/GHCi │──UDP────▶│ SuperDirt        │
│                  │ 127.0.0.1│ scsynth          │
│                  │ :57120   │ (オーディオ出力)  │
└──────────────────┘          └──────────────────┘
```

## 1. WSL2 ネットワーク設定

### networkingMode=mirrored（必須）

WSL2 のデフォルト（NAT モード）では `127.0.0.1` が WSL と Windows で異なるため、SuperDirt に接続できない。mirrored モードでは同一のネットワークスタックを共有し、`127.0.0.1` で直接通信できる（[WSL ネットワーキング公式ドキュメント](https://learn.microsoft.com/ja-jp/windows/wsl/networking)）。

```ini
# %USERPROFILE%\.wslconfig
[wsl2]
networkingMode=mirrored
```

設定変更後は `wsl --shutdown` で WSL を再起動が必要。

#### 注意点

- WSL と Windows で同じポートを同時に listen できない（ポート競合）
- Docker Desktop (WSL2 backend) との併用で Hyper-V ファイアウォール設定が必要な場合あり（未検証）
- Windows 側 VPN が WSL にも影響する

### Windows ファイアウォール

UDP 57120 を許可する必要がある。管理者権限の cmd で:

```
netsh advfirewall firewall add rule name="SuperDirt UDP" dir=in action=allow protocol=UDP localport=57120
```

## 2. SuperCollider + SuperDirt（Windows 側）

### インストール

```powershell
winget install SuperCollider.SuperCollider
```

### SuperDirt インストール

SC IDE を起動し、エディタで実行（Ctrl+Enter）:

```supercollider
Quarks.install("SuperDirt")
```

完了後 **Language → Recompile Class Library** (Ctrl+Shift+L)。

**注意**: CLI (`sclang.exe`) からの `Quarks.install` は Windows 環境で不安定なため、IDE での手動インストールを推奨。

### CLI 起動

WSL から sclang.exe を直接実行可能:

```bash
"/mnt/c/Program Files/SuperCollider-3.14.1/sclang.exe" \
  "$(wslpath -w scripts/start_superdirt.scd)"
```

### start_superdirt.scd の設定

| 設定 | 値 | 理由 |
|------|-----|------|
| `numBuffers` | 4096 | デフォルト 1024 では SuperDirt のサンプル数（219フォルダ）に不足 |
| `memSize` | 8192 * 32 (256MB) | デフォルトでは SuperDirt 起動時にメモリ不足エラー |

## 3. Linux 依存パッケージ（WSL2 側）

GHC のビルドに必要:

```bash
sudo apt-get install -y gcc g++ libgmp-dev
```

| パッケージ | 必要な理由 |
|-----------|-----------|
| `gcc` | C コンパイラ（GHC ビルド） |
| `g++` | C++ コンパイラ（GHC の configure が要求、`gcc` パッケージには含まれない） |
| `libgmp-dev` | GNU 多倍長整数ライブラリ（GHC のリンカが要求） |

## 4. Haskell ツールチェイン（WSL2 側）

mise (ghcup backend) で管理:

```bash
mise use -g ghcup@latest
mise use -g ghc@latest
mise use -g cabal@latest
```

dotfiles で管理する場合は `dot_config/mise/config.toml.tmpl` の `[tools]` セクションに追加。

## 5. TidalCycles（WSL2 側）

```bash
cabal update
cabal install --lib tidal
```

`--lib` フラグが必要（cabal v2 ではデフォルトでライブラリを GHC 環境に公開しないため）。

### BootTidal.hs

カスタム BootTidal.hs を `scripts/BootTidal.hs` に配置。mirrored モードでは `127.0.0.1:57120`（デフォルト）で SuperDirt に接続できる。

## 6. Rust ツールチェイン（WSL2 側）

mise で管理:

```bash
mise use -g rust@stable
```

## 動作確認

```bash
# ターミナル1: SuperDirt 起動
"/mnt/c/Program Files/SuperCollider-3.14.1/sclang.exe" \
  "$(wslpath -w scripts/start_superdirt.scd)"
# "SuperDirt: listening on port 57120" を待つ

# ターミナル2: Tidal から音を鳴らす
echo ':script scripts/BootTidal.hs
d1 $ sound "bd sn"
:!sleep 5
d1 $ silence
:quit' | ghci
# "Connected to SuperDirt." が出て音が鳴れば成功
```

## トラブルシューティング

### SuperDirt 起動時 "No more buffer numbers"

`s.options.numBuffers` の値が足りない。`start_superdirt.scd` で 4096 以上に設定。

### SuperDirt 起動時 "not enough free memory"

`s.options.memSize` の値が足りない。`start_superdirt.scd` で `8192 * 32`（256MB）以上に設定。

### SuperDirt 起動時 "yield was called outside of a Routine"

`SuperDirt.start` を直接呼んでいる。`s.waitForBoot { ... }` で包む必要がある。

### SuperDirt 起動時 "failed to open UDP socket: address in use"

前回の sclang/scsynth プロセスが残っている。Windows 側で:

```
taskkill /IM sclang.exe /F
taskkill /IM scsynth.exe /F
```

### Tidal "Waiting for SuperDirt" から進まない

1. SuperDirt が起動済みか確認（`SuperDirt: listening on port 57120` のログ）
2. `.wslconfig` の `networkingMode=mirrored` が設定済みか確認
3. `wsl --shutdown` 後に WSL を再起動したか確認
4. `ip addr show loopback0` で mirrored が有効か確認（loopback0 が存在すれば有効）
5. Windows ファイアウォールで UDP 57120 が許可されているか確認

### GHC インストール時 "g++: command not found"

`g++` が未インストール。`sudo apt-get install -y g++` で解決。

### cabal install 時 "cannot find -lgmp"

`libgmp-dev` が未インストール。`sudo apt-get install -y libgmp-dev` で解決。

### cabal install 時 "Installation might not be completed as desired"

`--lib` フラグを付けていない。`cabal install --lib tidal` で実行。
