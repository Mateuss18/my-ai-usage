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

O build normal continua produzindo um executavel sem assinatura para desenvolvimento.
Para testar uma instalacao neste PC, use o build assinado local:

```powershell
npm.cmd run build:windows:signed:local
```

Esse comando cria, se necessario, o certificado `My AI Usage Local Code Signing`
no usuario atual do Windows, confia nele apenas neste PC, gera o MSI x64 e valida
as assinaturas do executavel e do instalador. Esse certificado autoassinado nao e
uma identidade publica e nao deve ser usado para distribuir o aplicativo.

Para um release publico, importe um certificado Authenticode publico em
`Cert:\CurrentUser\My`, defina `MY_AI_USAGE_SIGNING_THUMBPRINT` com o thumbprint
dele e `MY_AI_USAGE_TIMESTAMP_URL` com o servico de timestamp fornecido pela
autoridade certificadora, e execute:

```powershell
npm.cmd run build:windows:signed
```

- Gere releases publicos em CI ou em uma maquina de build separada, mantendo a
  chave privada fora do Git e protegida pelo ambiente de release.
- Publique somente o instalador e o executavel cujo
  `Get-AuthenticodeSignature <arquivo>` retorne `Valid`.
- Se um antivirus acusar um release publico, pare a distribuicao e trate como
  possivel falso positivo. Usuarios finais nao devem desativar a protecao nem
  criar exclusoes temporarias no PC de uso.

## Principios

- Windows-first e local-first.
- Read-only: o aplicativo consulta dados e nao altera contas ou quotas.
- Credenciais e dados do usuario nao passam por servidores do projeto.
- Valores indisponiveis permanecem desconhecidos; nunca sao convertidos silenciosamente em zero.

Consulte [docs/PRODUCT.md](docs/PRODUCT.md) para o escopo e [docs/TECHNICAL_NOTES.md](docs/TECHNICAL_NOTES.md) para as decisoes tecnicas.

## Licenca

[MIT](LICENSE)
