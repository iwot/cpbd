# cpbd
Windows用クリップボード履歴ツール

APIサーバー（かつクリップボード監視）の立ち上げ。
```
cargo run -p watcher
```

`http://localhost:3030/memories` 、 `http://localhost:3030/urls` にアクセスすると、履歴がJSON形式で出力される。

`http://localhost:3030/memory/2` 、 `http://localhost:3030/url/2` にアクセスすると、この場合は履歴中のインデックスが2である内容をクリップボードに登録する。
なお、インデックスは1からの開始となる。

# TODO
- 履歴操作クライアントの作成。
- 履歴最大件数を指定できるようにする。
- APIサーバーのIP、ポートを指定できるようにする。
- コピー内容がURLだった場合の挙動を決定する。
- URLはURL用履歴にも保存する。
- URL用APIの拡充。
- 履歴からクリップボードに登録した場合、該当履歴を削除する（改めて履歴の先頭に入る）。
- 履歴の保存と復帰（SQLite？）。
