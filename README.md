# これは何? 
ネットワーク経由でddする簡易的なツールです｡

# 使い方
## サーバー側
サーバーはクライアントからの接続を待ち受け､接続が来たらファイルを送信します｡  
サーバーを実行するには以下のようにします｡  
```
netdd if=/dev/sda of=netdd://0.0.0.0:4545
```

## クライアント側
クライアントはサーバーに接続しデータを受け取り､ファイルに書き込みます｡  
クライアントを実行するには以下のようにします｡  
```
netdd if=netdd://192.168.0.10:4545 of=/dev/sda
```

## プロキシ
多分
```
netdd if=netdd://192.168.0.10:4545 of=netdd://0.0.0.0:4545
```
とかやればプロキシとして機能すると思います｡  

# 注意点
暗号化などは一切行っていないのでLANなど信頼できる環境下でのみご使用ください｡