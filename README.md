# My AI Usage

Aplicativo Tauri para Windows que mostra rapidamente o uso das quotas do Codex e seus horarios de renovacao.

## Stack

- Tauri 2 com Rust.
- Vue 3 com TypeScript e Vite.
- `codex app-server` local via JSON-RPC.

## Desenvolvimento

Requisitos: Windows x64, Node.js, Rust com o target `x86_64-pc-windows-msvc` e WebView2.

```powershell
npm.cmd ci
cargo fetch --locked --manifest-path src-tauri/Cargo.toml
npm.cmd run tauri -- dev
```

## Checks

```powershell
npm.cmd run lint
npm.cmd run typecheck
npm.cmd run test
npm.cmd run build
```

O executavel sem bundle fica em `src-tauri/target/x86_64-pc-windows-msvc/release/my-ai-usage.exe`.

## Distribuicao e instalacao

Nao gere nem instale o executavel diretamente no PC de uso. Uma compilacao local
produz um binario novo e sem reputacao, que pode ser bloqueado por heuristica do
antivirus.

- Gere o instalador em CI ou em uma maquina de build separada.
- Publique somente o instalador e o executavel assinados com certificado
  Authenticode.
- Antes de instalar, valide a assinatura com
  `Get-AuthenticodeSignature <arquivo>`; o status precisa ser `Valid`.
- Se um antivirus acusar o arquivo, pare a distribuicao e trate como possivel
  falso positivo. Nao desative a protecao e nao crie exclusoes temporarias no
  PC de uso.

## Principios

- Windows-first e local-first.
- Read-only: o aplicativo consulta dados e nao altera contas ou quotas.
- Credenciais e dados do usuario nao passam por servidores do projeto.
- Valores indisponiveis permanecem desconhecidos; nunca sao convertidos silenciosamente em zero.

Consulte [docs/PRODUCT.md](docs/PRODUCT.md) para o escopo e [docs/TECHNICAL_NOTES.md](docs/TECHNICAL_NOTES.md) para as decisoes tecnicas.

## Licenca

[MIT](LICENSE)
