<div align="center">
  <h1><code>neatar</code></h1>
  <p>
    <strong>The Web3 avavar</strong>
  </p>
</div>

## Develop

```shell
make fix 
make qa
make build
make clean
```

## Deploy test

```shell
make build
near dev-deploy
contractName=$(cat neardev/dev-account)
near state $contractName
```

## Usage

```shell
accountId=ilyar.testnet
contractName=$(cat neardev/dev-account)
near view $contractName get_num
near call $contractName increment --accountId $accountId
near view $contractName get_num
near call $contractName decrement --accountId $accountId
near view $contractName get_num
near delete $contractName $accountId
```
