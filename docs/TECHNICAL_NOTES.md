# Notas tecnicas

## Arquitetura

O aplicativo e uma aplicacao Tauri 2 para Windows. O frontend Vue/TypeScript apresenta o estado; o runtime Rust controla a bandeja, o ciclo de vida da janela, o processo local do Codex e a persistencia dos snapshots.

O frontend conversa com o runtime somente por comandos Tauri tipados. O contrato compartilhado em `src/domain/usage.ts` e `src-tauri/src/usage_contract.rs` preserva valores desconhecidos como `null`.

## Codex

O runtime inicia `codex app-server` pelo `PATH`, troca mensagens JSON por linha e consulta a conta e os limites. Payloads completos, tokens e credenciais nao sao persistidos nem registrados.

A identidade normalizada da conta e usada para separar snapshots locais. Uma falha de refresh conserva o ultimo snapshot valido como stale; uma resposta de erro nao e convertida em zero.

Cada perfil CLI usa um processo independente com `CODEX_HOME` definido apenas no
processo filho. O perfil principal preserva o ambiente atual; o secundario usa
`~/.codex-secondary`. O campo opcional `profileId` associa cada snapshot ao perfil
para direcionar login e logout. Snapshots antigos continuam legiveis.

Os perfis CLI locais do T3 sao descobertos novamente durante a atualizacao. Perfis
gerenciados pelo T3 sao ignorados: seu home nao contem uma sessao CLI equivalente.
O app nao le o armazenamento privado de credenciais do T3 nem modifica suas
configuracoes. Homes iguais sao consultados uma unica vez.

## Janela e bandeja

A janela inicia oculta e permanece residente para a bandeja. O clique alterna entre mostrar e ocultar; fechar a janela tambem oculta. O comando de saida encerra o provider e o processo do aplicativo.

O posicionamento considera o monitor e sua escala, mantendo o painel dentro da area de trabalho. Uma segunda inicializacao apenas ativa a instancia existente.

A altura inicial de 354 pixels permanece sincronizada no Rust e no arquivo Tauri.
Durante o uso, um `ResizeObserver` ajusta a altura nativa ao conteudo visivel,
incluindo resumos de contas, horarios mais longos e mensagens de erro. A janela
e reposicionada dentro da area de trabalho ao redimensionar.

## Verificacao

Os checks locais sao:

```powershell
npm.cmd run lint
npm.cmd run typecheck
npm.cmd run test
npm.cmd run build
git diff --check
```

Esses checks comprovam o codigo e o build local. Instalacao, comportamento distribuido e funcionamento em uma maquina limpa precisam ser verificados separadamente.
