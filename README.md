# My AI Usage

Aplicativo Tauri para Windows que mostra rapidamente o uso das quotas do Codex e seus horarios de renovacao.

## Duas contas com o T3

O painel mostra um resumo das quotas das duas contas ao mesmo tempo. Clique em uma
delas para ver os detalhes e os horarios de renovacao. `Active` indica uma leitura
atualizada; `Cached` indica dados salvos, que podem estar desatualizados.

1. Mantenha a primeira conta no Codex habitual (`~/.codex`, ou `CODEX_HOME`).
2. No menu do My AI Usage, escolha **Connect second account** e entre na segunda
   conta. Esse login usa `C:\Users\mateu\.codex-secondary` neste PC e nao substitui
   o login da primeira conta.
3. No T3, abra **Settings → Providers → Add provider → Codex** e use o nome
   **Codex 2**. Em **Config**, defina **CODEX_HOME path** como
   `C:\Users\mateu\.codex-secondary`, deixe **Shadow home path** vazio e salve.
   Esse caminho usa **Use existing CLI**, com `setupMode: existing`.
4. Selecione **Codex 2** ao iniciar uma conversa. A primeira conta continua
   disponivel nas outras conversas. Nao e necessario fechar o T3 para alternar
   entre conversas das duas contas.

Com homes independentes, cada conversa permanece no perfil em que comecou. A troca
de conta dentro da mesma conversa depende de compartilhar o estado das sessoes e
nao e configurada por este aplicativo.

O app tambem reconhece os homes de perfis CLI existentes nas configuracoes locais
do T3. Contas criadas por **Add ChatGPT account** no modo gerenciado do T3 usam
credenciais privadas dele e nao oferecem um login CLI consultavel por este app.
Use o fluxo acima para acompanhar as quotas. Nenhuma credencial e copiada entre
contas; o proprio Codex gerencia cada login no seu home. Veja a
[documentacao de autenticacao do Codex](https://learn.chatgpt.com/docs/auth?surface=cli).

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
- A consulta de quotas e somente leitura; login, logout e remocao sao acoes explicitas.
- Credenciais e dados do usuario nao passam por servidores do projeto.
- Valores indisponiveis permanecem desconhecidos; nunca sao convertidos silenciosamente em zero.

Consulte [docs/PRODUCT.md](docs/PRODUCT.md) para o escopo e [docs/TECHNICAL_NOTES.md](docs/TECHNICAL_NOTES.md) para as decisoes tecnicas.

## Licenca

[MIT](LICENSE)
