# My AI Usage — rewrite Tauri + Vue

Scaffold isolado da reescrita do My AI Usage. O aplicativo C#/WinUI no restante do repositório continua sendo a referência legada; este diretório não migra integrações, tray, UI final ou empacotamento.

## Localização e stack

- Diretório: `rewrite/`.
- Frontend: Vue 3.5.42, TypeScript 6.0.3 e Vite 8.2.2.
- Desktop: Tauri CLI 2.11.4, Tauri 2.x resolvido em `src-tauri/Cargo.lock`.
- Pacote: npm, com versões fixadas em `package.json` e `package-lock.json`.

## Pré-requisitos Windows x64

O ambiente verificado para este scaffold foi Windows 11 x64 (build `10.0.26200`), com Node.js `v24.13.1`, npm `11.10.0`, Rust `1.93.0` MSVC, Cargo `1.93.0` e o target `x86_64-pc-windows-msvc` instalado.

Para repetir o build, instale também:

- Visual Studio Build Tools com o workload **Desktop development with C++**;
- Microsoft Edge WebView2 Runtime;
- Rustup com o target `x86_64-pc-windows-msvc`.

## Instalação

No diretório `rewrite/`:

```powershell
npm.cmd ci
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
```

## Desenvolvimento

```powershell
npm.cmd run tauri -- dev
```

A página exibe um botão nativo e acessível por teclado. Ele chama o comando Rust `greet`, que responde `Hello from Rust!`. Se a ponte falhar, a mensagem aparece na própria página.

## Checks

Todos os comandos abaixo executam uma vez e terminam:

```powershell
npm.cmd run lint
npm.cmd run typecheck
npm.cmd run test
npm.cmd run build
```

`lint` executa ESLint no frontend, `cargo fmt --check` e Clippy. `typecheck` executa `vue-tsc`. `test` executa o check Vitest mockado da ponte e o teste Rust. `build` gera o frontend e um executável Windows x64 sem installer, bundle ou assinatura.

## Saída

O executável sem bundle fica em:

```text
src-tauri/target/x86_64-pc-windows-msvc/release/rewrite.exe
```

`dist/`, `node_modules/` e `src-tauri/target/` são saídas geradas e estão no `.gitignore`. Os fontes e os dois lockfiles permanecem rastreáveis.
