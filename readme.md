<div align="center">
  <h1><code>neatar</code></h1>
  <img src="https://raw.githubusercontent.com/neatar/dapp/main/src/web/asset/logo.svg" alt="neatar logo" />
  <p>
    <strong>The Web3 avatar like as Gravatar</strong>
  </p>
  <p>
    <a href="https://neatar.github.io/">neatar.github.io</a>
  </p>
</div>

# Docs API

See an example of integration for a third-party service: https://replit.com/@neatar/demo#index.html

## Getting avatar

```http request
POST https://rest.nearapi.org/view
Content-Type: application/json

{
  "contract": "alpha.neatar.testnet",
  "method": "avatar_of",
  "params": {
    "account_id": "ilyar.testnet"
  },
  "rpc_node": "https://rpc.testnet.near.org"
}
```

**Via curl**

```shell
curl https://rest.nearapi.org/view -H 'content-type: application/json' \
--data-raw '{
  "contract": "alpha.neatar.testnet",
  "method": "avatar_of",
  "params": {
    "account_id": "ilyar.testnet"
  },
  "rpc_node": "https://rpc.testnet.near.org"
}'
```

**Example response:**

```text
data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgMSAxIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxyZWN0IHg9IjAiIHk9IjAiIHdpZHRoPSIxIiBoZWlnaHQ9IjEiIGZpbGw9IiMwMDAiIHN0cm9rZT0iIzAwMCIvPjwvc3ZnPg0=
```
