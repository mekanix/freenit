const bootstrap = document.currentScript
const { default: init } = await import(bootstrap.dataset.module)

await init(bootstrap.dataset.wasm)
