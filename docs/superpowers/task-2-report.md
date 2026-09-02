# Relatório da Task 2 — Core do Codex

## Escopo

Migração do protocolo JSONL do spike para `MyAiUsage.Core`, com contratos tipados para janelas de quota, classificação segura de erros, processo `codex app-server`, leitura serializada, drenagem de `stderr` sem armazenamento e encerramento idempotente.

## Arquivos

- `src/MyAiUsage.Core/CodexClient.cs`: inicia `cmd.exe /d /c codex app-server`, executa `initialize`, `initialized`, `account/read` e `account/rateLimits/read`, ignora notificações, serializa leituras, classifica falhas e encerra a árvore do processo.
- `src/MyAiUsage.Core/CodexClientError.cs`: enumeração e exceção pública dos erros do cliente.
- `src/MyAiUsage.Core/RateLimitModels.cs`: modelos públicos de snapshot, bucket e janela.
- `src/MyAiUsage.Core/RateLimitParser.cs`: parser de `rateLimitsByLimitId` com fallback para `rateLimits`, validação de campos e marcação de dados parciais.
- `checks/MyAiUsage.Core.Checks/Program.cs`: checks executáveis sem framework para o contrato da Task 2.
- `docs/superpowers/task-2-report.md`: este relatório.

O spike permanece preservado para comparação até a validação autenticada real. `.serena/` e `.worktrees/` não foram alterados nem incluídos.

## Decisões verificadas

- O Core expõe somente snapshots tipados e `CodexClientException`; a UI não precisa conhecer `Process`, `JsonDocument` ou `TextReader`.
- O parser cria todas as janelas objeto que contêm campos de janela, inclusive chaves desconhecidas; metadados string são ignorados.
- Percentual nulo, não numérico ou fora de `0..100`, duração ausente/não positiva e timestamp inválido tornam o campo desconhecido e marcam o snapshot como parcial.
- Uma resposta sem janela utilizável lança `CodexClientException` com `PartialData`, sem fabricar `0%`.
- `stderr` é drenado em blocos e descartado.
- A leitura de quotas usa um único `SemaphoreSlim`; notificações sem o `id` esperado são consumidas e ignoradas.
- A criação e publicação do processo/streams é protegida por `_disposeLock` e revalida o estado descartado, evitando que `DisposeAsync` conclua antes de observar um processo recém-criado.
- `DisposeAsync` cancela a leitura, fecha stdin, mata a árvore ainda ativa e aguarda o processo; chamadas repetidas reutilizam a mesma tarefa de descarte.
- O check de corrida inspeciona somente o `Process` privado publicado pelo cliente e, em caso de falha, limpa apenas essa instância conhecida; não enumera nem encerra `cmd.exe` global do sistema.

## Verificações

Ambiente local: Windows, SDK .NET `10.0.400` (não há SDK `8.0.x` instalado). O projeto continua direcionado a `net8.0`; os comandos abaixo foram executados com o SDK disponível.

| Comando | Resultado |
| --- | --- |
| `rtk dotnet --list-sdks` | `10.0.400` |
| `rtk dotnet build checks\MyAiUsage.Core.Checks\MyAiUsage.Core.Checks.csproj -c Debug -warnaserror` | Passou: 2 projetos, 0 erros, 0 avisos |
| `rtk dotnet run --project checks\MyAiUsage.Core.Checks\MyAiUsage.Core.Checks.csproj -c Debug` | Passou após o fix round 1: `Core checks passed.`; inclui a disputa `StartAsync`/`DisposeAsync` |
| `rtk dotnet restore MyAiUsage.sln` | Passou: 4 projetos, 0 erros, 0 avisos reportados |
| `rtk dotnet build MyAiUsage.sln -c Debug -p:Platform=x64 -warnaserror` | Passou: 4 projetos, 0 erros, 0 avisos |
| `rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` | Passou: 4 projetos, 0 erros, 0 avisos |
| `rtk dotnet run --project checks\MyAiUsage.Core.Checks\MyAiUsage.Core.Checks.csproj -c Release --no-build` | Falhou: após build x64, procura o executável em `bin\Release` |
| `rtk dotnet run --project checks\MyAiUsage.Core.Checks\MyAiUsage.Core.Checks.csproj -c Release -p:Platform=x64 --no-build` | Passou: `Core checks passed.` |
| `rtk git diff --check` | Passou sem ocorrências |

Os checks usam processos falsos locais para confirmar a sequência JSON-RPC, notificações intermediárias, drenagem de stderr, duas leituras sem sobreposição, a disputa entre inicialização e descarte e as classificações de autenticação, EOF, JSON inválido, timeout, cancelamento e executável ausente. A resposta parcial é coberta diretamente pelo check do parser.

## Limitações

- Não foi possível reproduzir o fluxo com uma conta Codex autenticada nesta sessão; a autenticação disponível no ambiente não é válida para esse cenário. Nenhum `auth.json`, token, e-mail ou payload completo foi lido, copiado ou registrado.
- O check de executável ausente usa um `PATH` isolado e o check autenticado usa um `codex.cmd` temporário; isso valida o contrato do processo local, mas não substitui a execução com o binário Codex real.
- Não foram validados nesta Task o Visual Studio, o runtime de uma instalação MSIX, a máquina limpa, o tray, a UI ou os gates de desempenho; eles pertencem às Tasks 3–5.
