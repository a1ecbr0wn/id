# ID: External IP Address Cloudflare Worker

[![CI Build](https://github.com/a1ecbr0wn/id/actions/workflows/build.yml/badge.svg)](https://github.com/a1ecbr0wn/id/actions/workflows/build.yml) [![dependency status](https://deps.rs/repo/github/a1ecbr0wn/id/status.svg)](https://deps.rs/repo/github/a1ecbr0wn/id)

[Site: id.a1ecbr0wn.com](https://id.a1ecbr0wn.com)

A simple ip address webservice using [Cloudflare's workers service](https://developers.cloudflare.com/workers/).

This project uses the [`workers-rs`](https://github.com/cloudflare/workers-rs) crate to provide a simple rust WebAssembly binary that runs as a Cloudflare worker.  This is based off the standard [`worker-rust` template](https://github.com/cloudflare/templates/tree/main/worker-rust)

## Deployment

The project is built and deployed every time it is pushed to `main` via the `build.yml` github worflow, but it can also be deployed manually if you have [`wrangler` installed](https://developers.cloudflare.com/workers/get-started/guide/#1-install-wrangler-workers-cli) by running:

```sh
npm run deploy
```

## Testing

The project can be run locally in a test instance of `wrangler` by running:

```sh
npm run dev
```

## Usage

```sh
curl id.a1ecbr0wn.com
```
