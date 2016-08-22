# WARos
コードネーム`WARos`は日本発の超軽量カーネルとして開発中の実験的プロジェクトです。アプリケーションをOSとして構築でき、超軽量な組み込み開発を可能とします。

このプロジェクトには以下の特徴があります。
- Linuxなどの既存OSをベースとしておらず、ほぼ完全にフルスクラッチで構築されています。
- 必要な機能のみを取捨選択することによりバイナリサイズ・実行速度を極限まで最適化できます。
  - 実際にRaspberry Pi B+のような貧弱な環境においても起動は一瞬です。
- Rust言語の採用により実行速度と安全性を保ったまま高効率な開発が可能です。

## 前提環境
### binutils
クロスコンパイルのため、コンパイルするターゲット環境のbinutilsが必須です。  
具体的にはi686-elf-ldといったコマンドが必要となります。

### rustc
Rustのコンパイラです。  
一部のunstableな機能を利用するために[RustのNightlyビルド](http://doc.rust-lang.org/book/nightly-rust.html)が必要です。
現在ビルドを確認しているバージョンは`rustc 1.6.0-nightly (52d95e644 2015-11-30)`です。

### grub
x86向けのビルドには起動用のディスクイメージの生成のため、grub-mkrescueコマンドを使用しています。

## ビルド
MakefileはKernelディレクトリにあります。

1. 依存モジュールのダウンロードのため、先に`make UPDATE`が必要です。
  ただし、rustcのバージョンにあったソースコードが必要なため、[手動でダウンロード](https://static.rust-lang.org/dist/)することをおすすめします。
2. Kernelディレクトリ内で`make`してください。環境が揃っていればx86向けのバイナリが`kernel.x86.bin`及び`grub.x86.iso`として出力されます。  
   ARM(Raspberry Pi)向けにビルドする際は`ARCH=arm make`としてください。この場合のバイナリは`kernel.arm.bin`です。

### 注意
Mac OS Xでのビルドにおいてリンクエラーの発生を確認しています。依存ライブラリのビルドに失敗しているだけのようで、一度Linux環境でビルドすることで再ビルドが可能となります。

## エミュレーション
デバッグにはQEMUが便利です。

### x86
以下のコマンドで起動します。

```sh
qemu-system-i386 -cdrom grub.x86.iso -vga std -m 256 -serial stdio
```

### Raspberry Pi
Raspberry Pi環境のQEMUは[Torlus/qemu](https://github.com/Torlus/qemu/tree/rpi)にあります。
ただし、メモリの容量が取得できないため`kmain`にて手動でメモリ領域を指定する必要があります。

```sh
qemu-system-arm -kernel kernel.arm.bin.elf -cpu arm1176 -m 256 -M raspi -nographic
```

この場合 `Ctrl-A X` で終了します。

## デバッグ
QEMUの起動コマンドの最後に`-s -S`を付加するとデバッガの接続を待機することができます。
ターゲット環境用のgdbを接続してデバッグが可能です。この時のgdbコマンドのサンプルが`gdb.scr`にあります。x86環境で`gdb -x gdb.scr`とすることでデバッグを開始します。
