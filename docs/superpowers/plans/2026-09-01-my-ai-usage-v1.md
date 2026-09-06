# My AI Usage V1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Entregar uma aplicação Windows 11 empacotada em MSIX que fica na bandeja e mostra as quotas atuais do Codex com estados honestos, atualização manual/automática e encerramento limpo.

**Architecture:** Uma aplicação WinUI 3 local-first referencia uma biblioteca `MyAiUsage.Core`. O `CodexClient` concreto mantém um único processo `codex app-server`, conversa por JSONL e expõe snapshots tipados; a UI só conhece snapshots e erros classificados. A V1 usa polling serializado de 60 segundos, não cria abstração de provedores e não lê credenciais locais.

**Tech Stack:** C#, .NET 8, Windows App SDK/WinUI 3, MSIX, `System.Text.Json`, `Process`/`CancellationToken`, `SemaphoreSlim` e APIs Win32 mínimas para a bandeja. Não adicionar biblioteca de tray, gráficos ou DI sem um bloqueio reproduzível.

**Spec:** `docs/PRODUCT.md`, `docs/TECHNICAL_NOTES.md` e `docs/ROADMAP.md`.

## Global Constraints

- V1 suporta somente Windows 11 e Codex.
- A distribuição V1 é MSIX; o auto-start é habilitado por padrão e pode ser desligado pelo usuário.
- O app inicia oculto quando ativado pelo Windows; clicar no tray abre ou restaura a janela comum compacta.
- A V1 mantém uma única instância do app e um único `codex app-server`.
- O produto é local-first e read-only: não lê, persiste ou registra `auth.json`, tokens, e-mails ou payloads completos.
- O app-server usa `cmd.exe /d /c codex app-server`, sem terminal visível, stdin/stdout redirecionados e stderr drenado sem armazenamento.
- `ReadRateLimitsAsync` é serializado; refresh manual e timer não podem ter duas leituras simultâneas.
- Polling é o mecanismo de atualização V1; notificações do app-server são consumidas e ignoradas, e a próxima leitura ocorre no intervalo de 60 segundos.
- Um anel representa uma janela recebida; nunca calcular média, soma ou percentual agregado.
- O anel mostra percentual usado: verde de 0–49%, amarelo de 50–79% e vermelho de 80–100%.
- Percentual nulo, não numérico ou fora de 0–100 é desconhecido e usa anel neutro; nunca converter falha em `0%`.
- Janela parcial não substitui o último snapshot completo. A UI mantém o último valor válido, marca `Dados parciais` e exibe a hora do último snapshot completo.
- Após falha temporária, a UI mantém o último snapshot completo com `Desatualizado`; sem snapshot anterior, mostra estado acionável e anéis neutros.
- O estado `Limite atingido` só aparece para uma janela cujo percentual válido seja `100`.
- Acessibilidade básica — teclado, nomes acessíveis, contraste e não depender só de cor — é requisito V1; polimento avançado fica para V1.1.
- Não adicionar OpenCode, Claude, histórico, gráficos, notificações de limite, backend, telemetria, conta própria ou sistema de plugins.
- O gate de desempenho da release é: painel utilizável em até 5 s, até 100 MB de working set oculto após 5 min, até 200 MB aberto após 5 min e CPU média abaixo de 2% oculto fora do refresh.

## Estado de partida

- O checkout contém somente o spike Console, seus checks e documentação; não há solução, projeto WinUI, manifest, tray ou MSIX.
- `spike/MyAiUsage.Console` já prova JSONL, `initialize`, `initialized`, `account/rateLimits/read`, notificações intermediárias, erro RPC, autenticação ausente, múltiplos buckets e fallback.
- Os checks ainda não cobrem JSON inválido, EOF, timeout, cancelamento, `codex` ausente do `PATH`, resposta parcial, stderr cheio ou segunda instância.
- Build e execução autenticada anteriores são evidência histórica. A Task 0 deve reproduzir o que for possível e registrar data, comando, ambiente e resultado sem credenciais.
- `.serena/` e o diretório `docs/superpowers/` não fazem parte do produto; o executor deve adicionar somente o plano quando preparar o commit desta tarefa.

## Mapa de arquivos

Arquivos existentes a alinhar:

- `README.md`: estado real do projeto, dependência do Codex e privacidade.
- `docs/PRODUCT.md`: contrato da V1, incluindo MSIX, auto-start, tray e acessibilidade básica.
- `docs/ROADMAP.md`: ordem, critérios objetivos e evidência reproduzida.
- `docs/TECHNICAL_NOTES.md`: decisões fechadas, protocolo, processo e matriz de validação.
- `spike/MyAiUsage.Console/*`: fonte de comportamento até a migração passar.

Arquivos a criar durante a implementação:

- `global.json`: SDK .NET 8 exato usado no build.
- `MyAiUsage.sln`: solução com Core, app WinUI e checks.
- `src/MyAiUsage.Core/MyAiUsage.Core.csproj`: biblioteca sem dependência de WinUI.
- `src/MyAiUsage.Core/CodexClient.cs`, `CodexClientError.cs`, `RateLimitModels.cs`, `RateLimitParser.cs`: processo, protocolo, erros e dados tipados.
- `src/MyAiUsage.App/MyAiUsage.App.csproj`, `App.xaml`, `App.xaml.cs`, `MainWindow.xaml`, `MainWindow.xaml.cs`, `QuotaRing.xaml`, `QuotaRing.xaml.cs`, `TrayIcon.cs`, `StartupTaskManager.cs`, `Package.appxmanifest`: shell WinUI, painel, tray, auto-start e ciclo de vida.
- `checks/MyAiUsage.Core.Checks/MyAiUsage.Core.Checks.csproj`, `checks/MyAiUsage.Core.Checks/Program.cs`: checks executáveis sem framework de testes.
- `.gitignore`: ignorar `artifacts/` e saídas locais do empacotamento.
- `docs/INSTALL.md`: instalação, remoção, auto-start, dependência do Codex e limitações.

## Ordem e dependências

`Task 0` alinha o contrato documental e reproduz o baseline. `Task 1` cria a solução. `Task 2` migra o protocolo para Core. `Task 3` usa os tipos de Core para renderizar e atualizar. `Task 4` conecta tray, auto-start e encerramento ao cliente já existente. `Task 5` fecha MSIX, documentação e matriz de release.

---

### Task 0: Alinhar o contrato V1 e fechar o baseline

**Files:**
- Modify: `README.md`
- Modify: `docs/PRODUCT.md`
- Modify: `docs/ROADMAP.md`
- Modify: `docs/TECHNICAL_NOTES.md`
- Read: `spike/MyAiUsage.Console/CodexAppServer.cs`
- Read: `spike/MyAiUsage.Console/RateLimitFormatter.cs`
- Test: `spike/MyAiUsage.Console.Checks/Program.cs`

**Interfaces:**
- Consumes: o escopo atual e o comportamento comprovado pelo spike.
- Produces: contrato documental único para MSIX, auto-start, tray, polling serializado, estados, desempenho e privacidade.

- [ ] **Step 1: Reproduzir o baseline do spike.** Execute:

```powershell
rtk dotnet restore spike\MyAiUsage.Console.Checks\MyAiUsage.Console.Checks.csproj
rtk dotnet build spike\MyAiUsage.Console.Checks\MyAiUsage.Console.Checks.csproj -c Release -warnaserror
rtk dotnet run --project spike\MyAiUsage.Console.Checks\MyAiUsage.Console.Checks.csproj -c Release --no-build
```

Esperado: build com 0 erros e 0 avisos e a linha `Process configuration checks passed.`. Se o build falhar, registrar a falha em `docs/TECHNICAL_NOTES.md`; não marcar o baseline como verde.

- [ ] **Step 2: Acrescentar checks dos comportamentos já prometidos.** Em `spike/MyAiUsage.Console.Checks/Program.cs`, adicionar antes da mensagem final:

```csharp
await AssertThrowsAsync<JsonException>(
    () => CodexAppServer.RequestAsync(
        TextWriter.Null, new StringReader("{invalid}\n"), 4, "test", null),
    "rejects invalid JSON"
);

await AssertThrowsAsync<EndOfStreamException>(
    () => CodexAppServer.RequestAsync(
        TextWriter.Null, new StringReader(string.Empty), 5, "test", null),
    "rejects EOF before a response"
);

using var invalidPercent = JsonDocument.Parse("""
{ "rateLimits": { "primary": { "usedPercent": 101 } } }
""");
Assert(
    RateLimitFormatter.Format(invalidPercent.RootElement, TimeZoneInfo.Utc)
        .Single().Contains("uso desconhecido", StringComparison.Ordinal),
    "does not present an out-of-range percentage as valid"
);
```

- [ ] **Step 3: Alinhar os quatro documentos.** Registrar literalmente que MSIX, auto-start padrão, início oculto, tray e acessibilidade básica são V1; deixar instalador mais amigável, auto-update e polimento avançado em V1.1. Registrar que polling serializado de 60 segundos é a decisão V1 para notificações.
- [ ] **Step 4: Registrar os limites de desempenho e a separação de evidência.** Em `TECHNICAL_NOTES.md`, criar uma tabela com cenário, comando/ação, máquina/SDK, data, resultado, limite e limitação conhecida. Identificar como histórica qualquer execução anterior que não seja reproduzida.
- [ ] **Step 5: Verificar a documentação.** Execute `rtk git diff --check` e confirme com `rtk rg -n -- 'MSIX|auto-start|60 segundos|100 MB|200 MB|2%' README.md docs`. Todos os itens fechados da V1 devem estar explícitos nos documentos.
- [ ] **Step 6: Commitar somente a documentação alinhada.** Execute:

```powershell
rtk git add README.md docs\PRODUCT.md docs\ROADMAP.md docs\TECHNICAL_NOTES.md spike\MyAiUsage.Console.Checks\Program.cs
rtk git diff --cached --check
rtk git commit -m "docs: close My AI Usage v1 contract"
```

**Saída:** os documentos deixam de se contradizer e o spike tem checks explícitos para JSON inválido, EOF e percentual fora do intervalo.

---

### Task 1: Criar a solução WinUI 3 empacotada e o Core sem UI

**Files:**
- Create: `global.json`
- Create: `MyAiUsage.sln`
- Create: `src/MyAiUsage.Core/MyAiUsage.Core.csproj`
- Create: `src/MyAiUsage.App/MyAiUsage.App.csproj`
- Create: `src/MyAiUsage.App/App.xaml`
- Create: `src/MyAiUsage.App/App.xaml.cs`
- Create: `src/MyAiUsage.App/MainWindow.xaml`
- Create: `src/MyAiUsage.App/MainWindow.xaml.cs`
- Create: `src/MyAiUsage.App/Package.appxmanifest`
- Create: `checks/MyAiUsage.Core.Checks/MyAiUsage.Core.Checks.csproj`
- Create: `checks/MyAiUsage.Core.Checks/Program.cs`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: template oficial `Blank App, Packaged (WinUI 3 in Desktop)` e o contrato da Task 0.
- Produces: solução que restaura, compila em `x64`, abre uma janela vazia e empacota em MSIX sem iniciar terminal.

- [ ] **Step 1: Fixar o SDK usado.** Execute `rtk dotnet --list-sdks`, escolha o maior SDK instalado da linha `8.0.x` e crie `global.json` com `version` igual ao valor concreto retornado, `rollForward` igual a `latestPatch` e `allowPrerelease` igual a `false`. O arquivo não pode conter versão simbólica nem SDK preview.

- [ ] **Step 2: Criar o shell pelo template oficial.** Gerar o projeto empacotado no Visual Studio com `TargetFramework` Windows, `WindowsPackageType=MSIX`, plataforma inicial `x64` e a versão de Windows App SDK emitida pelo template. Adicionar o projeto `MyAiUsage.Core` como class library `net8.0` e o projeto de checks com referência somente ao Core. Acrescentar `artifacts/` ao `.gitignore`.
- [ ] **Step 3: Escrever o check mínimo da janela.** Em `checks/MyAiUsage.Core.Checks/Program.cs`, deixar este primeiro check executável:

```csharp
Console.WriteLine("Core checks passed.");
```

Esperado: `Core checks passed.` sem carregar assemblies WinUI.
- [ ] **Step 4: Implementar a janela vazia.** `App.xaml.cs` deve criar uma única `MainWindow`; `MainWindow.xaml` deve conter somente um `Grid` e um título acessível. Não iniciar `CodexClient`, tray ou timer ainda.
- [ ] **Step 5: Validar restauração e build.** Execute:

```powershell
rtk dotnet restore MyAiUsage.sln
rtk dotnet build MyAiUsage.sln -c Debug -p:Platform=x64 -warnaserror
```

Esperado: exit code 0, 0 erros e 0 avisos.
- [ ] **Step 6: Validar o shell manualmente.** Inicie o app pelo Visual Studio, feche a janela e confirme que nenhum `cmd.exe` ou `codex app-server` foi criado. Não marcar MSIX instalado como validado nesta etapa.
- [ ] **Step 7: Commitar o shell.** Execute `rtk git diff --check`, depois:

```powershell
rtk git add global.json MyAiUsage.sln src checks .gitignore
rtk git diff --cached --check
rtk git commit -m "feat: scaffold packaged WinUI app"
```

**Saída:** solução WinUI x64 restaurável e compilável, com uma janela vazia e Core independente da UI.

---

### Task 2: Migrar o spike para `CodexClient` e parser tipado

**Files:**
- Create: `src/MyAiUsage.Core/CodexClient.cs`
- Create: `src/MyAiUsage.Core/CodexClientError.cs`
- Create: `src/MyAiUsage.Core/RateLimitModels.cs`
- Create: `src/MyAiUsage.Core/RateLimitParser.cs`
- Modify: `checks/MyAiUsage.Core.Checks/Program.cs`
- Modify: `src/MyAiUsage.Core/MyAiUsage.Core.csproj`
- Read until migration passes: `spike/MyAiUsage.Console/CodexAppServer.cs`, `spike/MyAiUsage.Console/RateLimitFormatter.cs`

**Interfaces:**
- Consumes: JSONL e processo validados pelo spike.
- Produces: os contratos públicos abaixo; a UI não chama `Process`, `JsonDocument` ou `TextReader` diretamente.

```csharp
namespace MyAiUsage.Core;

public enum CodexClientErrorKind
{
    ExecutableNotFound,
    AuthenticationRequired,
    EndOfStream,
    InvalidJson,
    Timeout,
    ProtocolError,
    PartialData,
    Cancelled
}

public sealed class CodexClientException : Exception
{
    public CodexClientErrorKind Kind { get; }
    public CodexClientException(CodexClientErrorKind kind, string message, Exception? inner = null)
        : base(message, inner) => Kind = kind;
}

public sealed record RateLimitWindow(
    string Key,
    int? UsedPercent,
    long? WindowDurationMins,
    DateTimeOffset? ResetsAt);

public sealed record RateLimitBucket(
    string Id,
    string DisplayName,
    IReadOnlyList<RateLimitWindow> Windows);

public sealed record RateLimitSnapshot(
    IReadOnlyList<RateLimitBucket> Buckets,
    DateTimeOffset RetrievedAt,
    bool IsPartial);

public sealed class CodexClient : IAsyncDisposable
{
    public Task StartAsync(CancellationToken cancellationToken = default);
    public Task<RateLimitSnapshot> ReadRateLimitsAsync(CancellationToken cancellationToken = default);
    public ValueTask DisposeAsync();
}
```

- [ ] **Step 1: Escrever checks que falham para o contrato.** Adicionar checks para: janela válida; todos os objetos de janela do bucket, incluindo uma chave desconhecida; `usedPercent` nulo, não numérico e fora de 0–100; duração ausente ou não positiva; timestamp inválido; resposta parcial; erro RPC de autenticação; EOF; timeout e cancelamento.
- [ ] **Step 2: Implementar o parser mínimo.** Em `RateLimitParser.Parse(JsonElement, TimeZoneInfo)`, percorrer `rateLimitsByLimitId`; quando vazio, usar `rateLimits`. Para cada propriedade objeto que contenha campos de janela, criar `RateLimitWindow` com valores inválidos como `null` e marcar `IsPartial=true`. Propriedades de metadados string não são janelas. Se nenhuma janela puder ser criada, lançar `CodexClientException(PartialData, "Não foi possível ler as janelas de quota do Codex.")`.
- [ ] **Step 3: Implementar o processo sem expor stderr.** Reaproveitar `cmd.exe` com:

```csharp
var info = new ProcessStartInfo("cmd.exe")
{
    UseShellExecute = false,
    CreateNoWindow = true,
    RedirectStandardInput = true,
    RedirectStandardOutput = true,
    RedirectStandardError = true
};
info.ArgumentList.Add("/d");
info.ArgumentList.Add("/c");
info.ArgumentList.Add("codex");
info.ArgumentList.Add("app-server");
```

Iniciar um leitor assíncrono que drena `StandardError` em blocos e descarta cada bloco. Nunca guardar ou registrar stderr.
- [ ] **Step 4: Implementar a sequência JSON-RPC.** `StartAsync` inicia o processo e envia `initialize`; depois envia a notificação `initialized`. `ReadRateLimitsAsync` envia primeiro `account/read` com `{"refreshToken":false}` e depois `account/rateLimits/read` com `params: null`. Uma única leitura pendente é permitida por vez com `SemaphoreSlim(1, 1)`; linhas sem o `id` esperado são notificações e são ignoradas.
- [ ] **Step 5: Implementar classificação e limpeza.** Mapear EOF com exit code `9009` para `ExecutableNotFound`; erro de autenticação para `AuthenticationRequired`; JSON inválido, timeout, cancelamento e erro RPC para seus respectivos tipos. `DisposeAsync` deve ser idempotente, cancelar o leitor, fechar stdin, matar a árvore com `Kill(entireProcessTree: true)` quando ainda ativa e aguardar o processo; capturar a corrida em que ele já encerrou.
- [ ] **Step 6: Executar os checks em red-green.** Rode primeiro `rtk dotnet run --project checks\MyAiUsage.Core.Checks\MyAiUsage.Core.Checks.csproj -c Debug`; confirme que os novos checks falham antes da implementação, implemente o menor código e rode novamente até aparecer `Core checks passed.`.
- [ ] **Step 7: Validar o fluxo real sem guardar payload.** Com uma conta Codex já autenticada, execute o app Console e registre somente presença de buckets, percentual, duração e reset. Teste também estado deslogado e `codex` ausente em processo isolado; nunca pedir logout da conta ativa nem copiar resposta JSON.
- [ ] **Step 8: Manter o spike até a migração passar.** Só depois dos checks e do fluxo real reproduzidos remover ou arquivar `spike/`; se ainda for útil para comparação, mantê-lo sem duplicar lógica usada pela UI.
- [ ] **Step 9: Commitar Core e checks.** Execute:

```powershell
rtk git add src\MyAiUsage.Core checks\MyAiUsage.Core.Checks
rtk git diff --cached --check
rtk git commit -m "feat: add typed Codex quota client"
```

**Saída:** Core obtém um snapshot tipado ou erro seguro, drena stderr, encerra o processo e não expõe JSON-RPC à UI.

---

### Task 3: Renderizar quotas, estados e refresh serializado

**Files:**
- Create: `src/MyAiUsage.App/QuotaRing.xaml`
- Create: `src/MyAiUsage.App/QuotaRing.xaml.cs`
- Modify: `src/MyAiUsage.App/MainWindow.xaml`
- Modify: `src/MyAiUsage.App/MainWindow.xaml.cs`

**Interfaces:**
- Consumes: `RateLimitSnapshot`, `RateLimitBucket`, `RateLimitWindow` e `CodexClientException` da Task 2.
- Produces: painel com um anel por janela, estado textual, hora do último snapshot completo, refresh manual e timer de 60 segundos.

- [x] **Step 1: Escrever checks de apresentação.** Antes do XAML final, criar no check project funções puras para título e cor e verificar:

```csharp
Assert(WindowTitle(300) == "Janela de 5 horas", "formats hours");
Assert(WindowTitle(10080) == "Janela de 7 dias", "formats days");
Assert(UsageColor(49) == "green", "uses green below 50");
Assert(UsageColor(80) == "red", "uses red at 80");
Assert(UsageColor(null) == "neutral", "uses neutral for unknown");
```

- [x] **Step 2: Implementar `QuotaRing`.** Usar um `Ellipse` neutro e um traço de progresso calculado por `StrokeDashArray`/`StrokeDashOffset`; expor apenas `RateLimitWindow Window`, `string BucketName` e `string AccessibleDescription`. Percentual nulo usa o traço neutro. O título deriva de minutos: dias quando divisível por 1440, horas quando divisível por 60 e minutos nos demais casos; duração ausente usa `Janela sem duração informada`.
- [x] **Step 3: Implementar a grade e os textos equivalentes.** `MainWindow.xaml` deve renderizar todos os buckets/janelas sem agregação. Cada anel deve anunciar bucket, título, percentual ou `uso desconhecido`, reset ou `reset desconhecido` e estado. Mostrar sempre texto fora do anel; cor não pode ser a única informação.
- [x] **Step 4: Implementar os estados.** Mapear `CodexClientErrorKind` para `Codex ausente`, `Desconectado`, `Falha temporária`, `Dados parciais` e `Atualização cancelada`. Exibir `Carregando` no primeiro refresh, `Disponível` com snapshot completo e `Limite atingido` por janela em 100%.
- [x] **Step 5: Implementar o refresh único.** Centralizar botão e timer em `RefreshAsync` e proteger com `SemaphoreSlim(1, 1)`:

```csharp
private async Task RefreshAsync(CancellationToken cancellationToken)
{
    await _refreshGate.WaitAsync(cancellationToken);
    try
    {
        SetState("Carregando");
        var snapshot = await _client.ReadRateLimitsAsync(cancellationToken);
        if (snapshot.IsPartial)
        {
            Render(_lastGoodSnapshot ?? snapshot);
            SetState("Dados parciais");
            return;
        }
        _lastGoodSnapshot = snapshot;
        Render(snapshot);
        SetState("Disponível");
    }
    catch (CodexClientException error)
    {
        SetState(error.Kind == CodexClientErrorKind.Cancelled
            ? "Atualização cancelada"
            : MapError(error.Kind));
        Render(_lastGoodSnapshot);
    }
    finally { _refreshGate.Release(); }
}
```

`WaitAsync(cancellationToken)` aguarda e enfileira refreshes concorrentes do botão e do timer; não descarta uma chamada, e o `finally` só libera o semáforo depois de uma aquisição bem-sucedida. Para uma resposta parcial, `Render(_lastGoodSnapshot ?? snapshot)` preserva o último snapshot completo e sua hora quando ele existe; sem snapshot completo anterior, renderiza os campos utilizáveis do parcial e placeholders para os inválidos. O `RetrievedAt` parcial nunca é exibido como hora do último snapshot completo, nem o parcial é atribuído a `_lastGoodSnapshot`. Se houver falha com snapshot anterior, manter anéis e hora anterior e acrescentar `Desatualizado`; sem snapshot anterior, renderizar anéis neutros. Não atualizar `RetrievedAt` em erro ou parcial.
- [x] **Step 6: Configurar o timer sem custo oculto.** O timer de 60 segundos roda somente enquanto a janela estiver visível; ao ocultar, parar o timer; ao restaurar, iniciar e atualizar se o snapshot tiver mais de 60 segundos. O app-server permanece controlado pelo app e não há polling enquanto o painel estiver oculto.
- [x] **Step 7: Validar UI e checks.** Execute os checks Core, `rtk dotnet build MyAiUsage.sln -c Debug -p:Platform=x64 -warnaserror`, e teste manualmente todos os buckets, campos ausentes, 49/50/79/80/100%, resposta parcial e falha após snapshot válido.
- [ ] **Step 8: Commitar o painel.** Execute `rtk git diff --check`, depois:

```powershell
rtk git add src\MyAiUsage.App checks\MyAiUsage.Core.Checks
rtk git diff --cached --check
rtk git commit -m "feat: show Codex quota states"
```

**Saída:** o painel apresenta cada janela com informação equivalente para teclado/leitor de tela e nunca apaga valor válido por causa de erro ou resposta parcial.

#### Remediation

##### Round 1 review

- [x] [CRITICAL] Executar o build obrigatório da Task 3 com acesso aos dados de auditoria de vulnerabilidades do NuGet e registrar o resultado aprovado. - `docs/superpowers/plans/2026-09-01-my-ai-usage-v1.md:341` (`rtk dotnet build MyAiUsage.sln -c Debug -p:Platform=x64 -warnaserror`: 4 projetos, 0 erros, 0 avisos, 2026-09-03)
- [x] [CRITICAL] Executar e registrar todos os cenários manuais exigidos da Task 3: buckets, campos ausentes, 49/50/79/80/100%, resposta parcial e falha após snapshot válido. - `docs/superpowers/plans/2026-09-01-my-ai-usage-v1.md:341`
- [x] [NON_CRITICAL] Corrigir literais visíveis com UTF-8 corrompido em `MainWindow.xaml`, `MainWindow.xaml.cs` e `QuotaRing.xaml.cs`.
- [x] [NON_CRITICAL] Fazer os checks executáveis chamarem a lógica de apresentação de produção e cobrirem estados disponível, limite atingido e desconhecido.
- [x] [NON_CRITICAL] Executar e registrar a validação manual de buckets, campos ausentes, limites, resposta parcial e falha após snapshot válido.
  - Validado pelo usuário (2026-09-03): buckets, campos ausentes, 49/50/79/80/100%, resposta parcial e falha após snapshot válido.

#### Delivery

- [x] Final review: PASS
- [x] Merge request created - https://github.com/Mateuss18/my-ai-usage/pull/11

---

### Task 4: Conectar tray, auto-start, instância única e encerramento

**Files:**
- Create: `src/MyAiUsage.App/TrayIcon.cs`
- Create: `src/MyAiUsage.App/StartupTaskManager.cs`
- Modify: `src/MyAiUsage.App/App.xaml.cs`
- Modify: `src/MyAiUsage.App/MainWindow.xaml.cs`
- Modify: `src/MyAiUsage.App/Package.appxmanifest`

**Interfaces:**
- Consumes: `CodexClient.DisposeAsync`, `MainWindow.RefreshAsync` e o shell da Task 1.
- Produces: `void OpenOrRestoreWindow()`, `void HideWindow()`, `Task ExitApplicationAsync()`, `Task<StartupTaskState> GetStateAsync()` e `Task<StartupTaskState> SetEnabledAsync(bool enabled)`.

```csharp
public enum StartupTaskState { Enabled, DisabledByUser, DisabledByPolicy, Unavailable }
```

- [ ] **Step 1: Escrever a aceitação do ciclo de vida.** Registrar checks manuais que falhem se: fechar a janela matar o app, sair deixar `codex app-server`, iniciar duas instâncias, ou reiniciar o Explorer remover permanentemente o ícone.
- [ ] **Step 2: Impedir segunda instância.** No início de `App.OnLaunched`, chamar `AppInstance.FindOrRegisterForKey("my-ai-usage")`. A instância secundária deve redirecionar a ativação para a instância principal e terminar; a principal deve abrir/restaurar a janela.
- [ ] **Step 3: Implementar o tray com APIs Win32 mínimas.** `TrayIcon.cs` deve usar o HWND de `WindowNative.GetWindowHandle`, `Shell_NotifyIconW`/`NOTIFYICONDATAW`, callback `WM_APP`, `CreatePopupMenu`, `AppendMenuW`, `TrackPopupMenuEx`, `DestroyMenu` e destruição do `HICON`. Registrar `RegisterWindowMessage("TaskbarCreated")` e reenviar `NIM_ADD` ao receber a mensagem. Liberar todos os handles em `Dispose`.
- [ ] **Step 4: Implementar ocultar versus sair.** Interceptar `AppWindow.Closing`, marcar `args.Cancel = true` e chamar `AppWindow.Hide()` quando o usuário fechar a janela. Menu `Sair` deve marcar `_isExiting`, parar o timer, aguardar `CodexClient.DisposeAsync`, liberar tray e chamar `Application.Current.Exit()`; `DisposeAsync` deve ser seguro se chamado duas vezes.
- [ ] **Step 5: Declarar auto-start no manifest.** Adicionar namespace `uap5` e a extensão `windows.startupTask` com `TaskId`, `Executable="$targetnametoken$.exe"`, `EntryPoint="Windows.FullTrustApplication"`, `Enabled="true"` e nome visível. Manter o XML gerado pelo template e alterar somente a extensão necessária.
- [ ] **Step 6: Implementar o toggle nativo.** `StartupTaskManager` deve usar `StartupTask.GetAsync()`, expor estados `Enabled`, `DisabledByUser`, `DisabledByPolicy` e solicitar `RequestEnableAsync`/`Disable`. Não escrever Registry manualmente. Se a tarefa estiver indisponível fora do pacote, mostrar configuração desabilitada e motivo.
- [ ] **Step 7: Testar o ciclo completo.** Instalar o pacote de desenvolvimento, habilitar/desabilitar auto-start, reiniciar sessão, confirmar início oculto, abrir pelo tray, fechar sem sair, sair pelo menu, iniciar uma segunda instância e reiniciar o Explorer. Verificar em cada saída que não existe processo filho órfão.
- [ ] **Step 8: Commitar o ciclo de vida.** Execute:

```powershell
rtk git add src\MyAiUsage.App
rtk git diff --cached --check
rtk git commit -m "feat: add tray and startup lifecycle"
```

**Saída:** o app é residente de forma previsível, tem uma instância, o tray se recupera do Explorer e o encerramento libera o app-server.

#### Issue #6: instância única e ciclo de vida do tray

Escopo desta issue: Task 4, Steps 1–4 e os checks de ciclo de vida aplicáveis da Step 7. Auto-start, `StartupTaskManager` e alterações de manifest das Steps 5–6 ficam fora desta issue.

##### Acceptance criteria

- [x] Ao iniciar uma segunda instância, `AppInstance.FindOrRegisterForKey("my-ai-usage")` identifica a instância principal, encaminha a ativação, restaura/ativa a janela principal e encerra a secundária.
- [x] O app cria um único ícone de tray com as APIs Win32 mínimas da Task 4, oferece as ações de abrir/restaurar e `Sair`, e libera menu, ícone e demais handles nativos no descarte.
- [x] Fechar a janela cancela o fechamento, oculta a janela e mantém o processo e o ícone de tray residentes; abrir/restaurar pelo tray torna a mesma janela visível novamente.
- [x] Ao receber a mensagem registrada `TaskbarCreated`, o app registra novamente o ícone de tray sem criar uma segunda instância do aplicativo.
- [x] A ação `Sair` é segura contra chamadas repetidas, para o timer, cancela o trabalho pendente, aguarda `CodexClient.DisposeAsync`, descarta o tray e encerra o aplicativo sem deixar processo `codex app-server` órfão.
- [x] A implementação desta issue não adiciona auto-start, `StartupTaskManager` nem altera `Package.appxmanifest` para startup.

##### Verification

- [x] `rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` termina com exit code 0, sem erros nem avisos.
- [x] `rtk proxy powershell -NoProfile -Command "& '.\checks\MyAiUsage.Core.Checks\bin\x64\Release\net8.0\MyAiUsage.Core.Checks.exe'; exit $LASTEXITCODE"` termina com exit code 0 e imprime `Core checks passed.`.
- [x] `rtk git diff --check` termina com exit code 0.
- [ ] Check manual no Windows: com a primeira instância minimizada ou oculta, iniciar o app novamente restaura a primeira janela e o Gerenciador de Tarefas continua mostrando somente uma instância do app.
- [ ] Check manual no Windows: fechar a janela mantém o ícone de tray e o processo residentes; a ação de abrir/restaurar do tray reexibe a mesma janela.
- [ ] Check manual no Windows: reiniciar o Windows Explorer remove temporariamente o tray e o ícone reaparece após `TaskbarCreated`.
- [ ] Check manual no Windows: escolher `Sair` remove o ícone, encerra o app e confirma no Gerenciador de Tarefas que não restou processo `codex app-server` iniciado pelo app.

##### Remediation

###### Round 1 review

- [x] [CRITICAL] Decodificar somente o `LOWORD` de `lParam` nas notificações `NOTIFYICON_VERSION_4` para que os cliques esquerdo e direito acionem abrir/menu, com check executável do valor empacotado. - `src/MyAiUsage.App/TrayIcon.cs:142` (check executável imprime `Tray callback check passed.` antes dos checks Core)
- [x] [CRITICAL] Encaminhar `AppInstance.Activated` para a `DispatcherQueue` da UI antes de acessar `Window` ou `AppWindow`. - `src/MyAiUsage.App/App.xaml.cs:66` (build Release x64 sem erros/avisos)
- [ ] [CRITICAL] Executar e registrar os quatro checks manuais de Windows: segunda instância, fechar/restaurar pelo tray, recuperação após reiniciar o Explorer e saída sem `codex app-server` órfão. - `docs/superpowers/plans/2026-09-01-my-ai-usage-v1.md:422`

###### Round 2 review

- [x] [CRITICAL] Restaurar explicitamente o `OverlappedPresenter` quando a janela estiver minimizada; `AppWindow.Show()` cobre somente a janela oculta. - `src/MyAiUsage.App/App.xaml.cs` (`Restore()` condicionado a `Minimized`, compartilhado pelo tray e pela segunda instância; preserva janela maximizada)
- [ ] [CRITICAL] Executar e registrar os quatro checks manuais de Windows contra o build revisado. - `docs/superpowers/plans/2026-09-01-my-ai-usage-v1.md:422`

Validação da correção em 2026-09-04: `rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` passou com 0 erros e 0 avisos. Os checks manuais permanecem pendentes: `Get-AppxPackage *MyAiUsage*` não encontrou pacote instalado neste Windows. Na validação de restauração, minimizar a janela e reabrir tanto pelo tray quanto por uma segunda instância; repetir com a janela oculta e maximizada, confirmando que a maximização é preservada.

##### Delivery

Entrega direta autorizada pelo usuário após a correção: build Release x64 com warnings como erro e runner x64 passaram; MSIX corrigido em `src/MyAiUsage.App/artifacts/msix-fixed/MyAiUsage.App_0.1.0.0_x64_Test/MyAiUsage.App_0.1.0.0_x64.msix` foi assinado, verificado e instalado com exit code 0 e Status `Ok`, substituindo a instalação anterior. A abertura do PR foi solicitada explicitamente; o histórico do loop abaixo permanece preservado, sem atribuir PASS independente à correção posterior.

- [ ] Final review: PASS
- [x] Merge request created: https://github.com/Mateuss18/my-ai-usage/pull/12

---

#### Issue #7: auto-start nativo e configurável no Windows

##### Confirmed scope

- Objetivo: concluir o auto-start nativo, configurável e habilitado por padrão no Windows, fiel ao estado efetivo do sistema, iniciado oculto via `StartupTask` e sem regredir instância única, tray ou ciclo de vida da Issue #6.
- Entregas: uma única `uap5:Extension` `windows.startupTask` com `TaskId="MyAiUsageStartup"`, `Executable="$targetnametoken$.exe"`, `EntryPoint="Windows.FullTrustApplication"`, `Enabled="true"` e nome visível; preservar `windows.fullTrustProcess` e `runFullTrust`.
- Entregas: `StartupTaskManager` mínimo diretamente sobre `StartupTask.GetAsync()`, `RequestEnableAsync()` e `Disable()`, expondo `Enabled`, `EnabledByPolicy`, `Disabled`, `DisabledByUser`, `DisabledByPolicy` e `Unavailable`, sem interface ou abstração adicional.
- Entregas: toggle acessível na janela existente, alterável somente em `Enabled` e `Disabled`, com estado e motivo visíveis; estados de usuário/política ficam somente leitura e fora do MSIX ou falha de acesso resulta em `Unavailable` sem encerrar o app.
- Entregas: ativação `StartupTask` cria a infraestrutura residente com janela oculta; ativações comuns e redirecionadas continuam abrindo/restaurando a janela única e preservam tray, `Sair` e encerramento do app-server.
- Fora de escopo: Registry, serviço/background task próprio, nova página, abstrações, dependências, release/instalador final da Issue #9 e pendências gerais da Issue #6.
- Restrições: corrigir os dois findings do loop anterior, preservar todo o dirty work inicial, não promover evidência antiga a atual e não instalar certificado/pacote nem reiniciar a sessão sem autorização específica.
- Escopo confirmado explicitamente pelo usuário para um novo `$eloop`; perguntas usadas: 0/6.

##### Loop state

- Phase: blocked.
- Clarification questions used: 0/6.
- Round: 2/2.
- Implementation verdict: REVISE.
- Nota: o blocker abaixo registra o snapshot pré-restart; a evidência humana posterior encerra esse blocker para implementação.
- Current blocker and next action: round 2/2 review returned `REVISE`; the two-round limit is exhausted. Start a new `$eloop` to remove the production diagnostic marker, regenerate/revalidate the MSIX and reconcile the remaining acceptance checkboxes before delivery.
- Blocker and next action: a causa do token foi corrigida no pipeline por `ResolveStartupTaskExecutable`/`XmlPoke` após `AfterGenerateAppxManifest`; a fonte mantém `$targetnametoken$.exe`, e `dotnet msbuild` termina com exit code 0, gerando o MSIX padrão. O deployment unpackaged/debug exato foi resolvido como `PackageFullName=MyAiUsage_0.1.0.0_x64__5n9q33krdf9q0`, `FamilyName=MyAiUsage_5n9q33krdf9q0`, `Status=Ok`, sem processo `MyAiUsage.App` ativo, e removido com `Remove-AppxPackage` exit code 0. O certificado não exportável permanece em `CurrentUser\My`; o público foi confirmado em `CurrentUser\TrustedPeople`, `CurrentUser\Root` e, após autorização elevada, `LocalMachine\TrustedPeople`, sem chave privada no store de máquina. O MSIX padrão foi instalado com dependência e está `Status=Ok`; os cenários de toggle, acessibilidade, instância única, fechamento/restore pelo tray e `Sair` foram observados. Permanecem pendentes a ativação real de `StartupTask` após logout/reinício (não autorizado), os estados de política não reproduzidos sem forçar política e a confirmação visual de `Unavailable` no executável unpackaged (o runner comprova o manager). Próxima ação: autorizar especificamente logout/reinício para validar o auto-start real; não remover pacote/certificado antes disso.
- Baseline Git: `main == origin/main == c6439da458e8bb7753424d390e6e3c9de0c660c4`; stage vazio.
- Baseline dirty paths: `checks/MyAiUsage.Core.Checks/MyAiUsage.Core.Checks.csproj` (`72D8484EB007B3DB71CCBCF9D5B3798EF186C0BA36B8B0FDF7D7F6F24ACD3268`), `checks/MyAiUsage.Core.Checks/Program.cs` (`D69064ED6CAE0187F71BE80B440FA8B50C57E2F4211EE34DFE13D457B452D4A2`), este plano antes da atualização de spec (`8A46A2AF64ABECC1D1AF81E7E40183AA9C016659B554C9D39C9E4F08B81676F9`), `src/MyAiUsage.App/App.xaml.cs` (`F6129E819E780FAEDC9D509302C1429F17E3278A90EA1738DF60B7DA256C6185`), `MainWindow.xaml` (`282DC3A5B1829F9115F3FF788650BF5C21BE2D599E8C30F956FE8BE4032ED7BD`), `MainWindow.xaml.cs` (`D43C1AF55BC3995EBADB526837A07B5AFF92190571D5A7A453BAED216CD9EB8A`) e `Package.appxmanifest` (`4401D8874CF0DB7024A81009E396E8D4C69E23C10F121815394024BA9D7AFB6E`); `StartupTaskManager.cs` está untracked (`01F17E42B351AFAA43FF9CF51C8D1D5F34EAB206ED7E87937CE75D3BA5AD76E1`).
- Evidence snapshot (2026-09-05): `dotnet restore MyAiUsage.sln` passou; `dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` passou com 4 projetos, 0 erros e 0 avisos; o runner Release passou com `Tray callback check passed.` e `Core checks passed.`; `git diff --check` passou. O `dotnet msbuild` padrão termina com exit code 0 e gera `artifacts/msix-standard-fixed/MyAiUsage.App_0.1.0.0_x64_Test/MyAiUsage.App_0.1.0.0_x64.msix`; houve apenas o aviso opcional de `mspdbcmf.exe` ausente. O pacote foi assinado com `signtool sign` exit 0 e `signtool verify /pa /all` terminou com exit code 0; o MSIX assinado tem SHA-256 `5D7D74AEF8E80E65C68F48DABEB92A5A449206CEE550971796E2DD5318CC24DD`, e `makeappx unpack` terminou com exit code 0; manifest desempacotado SHA-256 `EA2D05F1C6CFEB7356F1F64DBED5C37B279DCCDC921E663CD2B4B7EC69F4EB40`, com `MyAiUsage.App.exe` resolvido na extensão. O certificado público está em `artifacts/msix-standard-fixed/MyAiUsage.Development.cer`, SHA-256 `EF60132FD4C7761EDDDB1B9838C07535340CFE4799C0850B67E410D818152E62`, thumbprint `0A4683BEDF7E1105EDF80F88446DFE8C252BC03C`, subject `CN=MyAiUsage`, validade `2026-09-05` a `2027-09-05`, sem chave privada em `TrustedPeople`/`Root`; `certutil -user -verify` passou. A remoção do deployment antigo foi exit code 0 com os identificadores acima. As tentativas de `Add-AppxPackage` do MSIX assinado, com e sem a dependência `Microsoft.WindowsAppRuntime.2.msix`, terminaram com exit code 1 e `0x800B0109`; `Get-AppxPackage -Name MyAiUsage` confirmou nenhum pacote instalado depois. A tentativa unsigned de diagnóstico terminou com exit code 1 e `0x80073CFB` antes da remoção porque havia registro unpackaged/debug. A execução direta do Release não empacotado terminou com `0xE0434352`/`REGDB_E_CLASSNOTREG` por ausência de inicialização do Windows App Runtime, e não é evidência nativa do pacote. Hashes atuais dos paths de código: checks csproj `72D8484EB007B3DB71CCBCF9D5B3798EF186C0BA36B8B0FDF7D7F6F24ACD3268`; checks Program `34F681FB90EEC3F1B0818B0600B0322AF970CFAC902EFA453B210FEB3E795A64`; App `F6129E819E780FAEDC9D509302C1429F17E3278A90EA1738DF60B7DA256C6185`; MainWindow.xaml `282DC3A5B1829F9115F3FF788650BF5C21BE2D599E8C30F956FE8BE4032ED7BD`; MainWindow.xaml.cs `63CEE19C1E528310C6FA8E57732CBE150D227BE0BE265D6681E117718F56A111`; Package.appxmanifest `015E993A091283A26638F645B49EB8DD27A1BC43AE5E602A36922D3675412AEF`; StartupTaskManager `EBB5B3FE48067AD0E53C6A64BF465259DA13A1C3A60D96F264AEDA45A68D65DE`; MyAiUsage.App.csproj `C31525E09E27F1216DFF662BC2CED83428A7627BDF9FC673FE2E9B8BFC0C1C69`.

Pós-autorização (2026-09-05): `Import-Certificate` elevado do único `.cer` esperado em `Cert:\LocalMachine\TrustedPeople` terminou exit code 0; o certificado confirmou thumbprint `0A4683BEDF7E1105EDF80F88446DFE8C252BC03C`, subject `CN=MyAiUsage` e `HasPrivateKey=False`. `Add-AppxPackage` do MSIX atual com a dependência x64 terminou exit code 0 e instalou `MyAiUsage_0.1.0.0_x64__5n9q33krdf9q0`/Family `MyAiUsage_5n9q33krdf9q0`, versão `0.1.0.0`, Status `Ok`. UI Automation observou lançamento normal responsivo, segunda ativação com um único PID, `WindowPattern.Close` ocultando a janela, `Shell_NotifyIconGetRect` encontrando o tray e callback nativo restaurando-a; teclado real alternou `On/Enabled` → `Off/Disabled` → `On/Enabled` e `Sair` encerrou app + `cmd.exe` + Node, sem PIDs filhos restantes. O runner unpackaged terminou exit code 0 e confirmou `Unavailable`/não mutabilidade fora do pacote. Startup real ainda requer logout/reinício, não autorizado; estados de política não foram forçados.

Pós-restart (2026-09-05): sem abrir/restaurar antes da captura, `LastBootUpTime=2026-09-05T19:05:36.5000000-03:00`; `Microsoft-Windows-Shell-Core/Operational` 62408 e `AppModel-Runtime/Admin` 201 registraram a execução de `MyAiUsage.App.exe` às `19:08:26.678`/`19:08:26.816`, PID `17252`, parent `sihost.exe`; o processo único estava responsivo, com janela nativa visível (`HWND=67236`, título `My AI Usage`) e tray encontrado (`Shell_NotifyIconGetRect=0x00000000`, `1658,1032–1690,1080`). UIA encontrou `My AI Usage` e `StartupToggle` `On`, habilitado; o pacote permaneceu `MyAiUsage_0.1.0.0_x64__5n9q33krdf9q0`, versão `0.1.0.0`, Status `Ok`. A árvore observada foi app `17252` → `cmd.exe 6272` → Node `3868` → `codex.exe`; após `WindowPattern.Close`, o mesmo PID ficou oculto (`HWND=0`), callback esquerdo do tray com o HWND preservado restaurou a janela visível, e `Sair` resultou em `AppProcessCount=0`; somente o Node `24724` do host Codex, parent `26452` e fora da árvore do app, permaneceu. Como a janela já estava visível na captura, não há prova suficiente para atribuir a ativação a `ExtendedActivationKind.StartupTask` nem para confirmar que o usuário não a abriu depois do login; esse critério permanece pendente. Próxima ação: perguntar se ela apareceu sozinha (tray/janela) ou se foi aberta manualmente; não reiniciar novamente nesta rodada.

Confirmação humana pós-login (2026-09-05): o usuário informou que não viu a janela aparecer após o reinício/login e observou o processo `MyAiUsage.App.exe` residente no Gerenciador de Tarefas. Combinada com os eventos técnicos de launch pós-boot, uma única instância, tray identificado e o ciclo callback/restauração já observado, essa confirmação sustenta auto-start residente com janela não apresentada no login; ela não expõe diretamente o enum `ExtendedActivationKind.StartupTask`, portanto essa limitação de instrumentação permanece explícita. A pendência de atribuição/manual-open do item anterior fica encerrada por este depoimento; não foram criados estados de política.

##### Fresh evidence — Round 2/2 (2026-09-05)

- A sequência reproduzível terminou com exit code 0 em cada comando: `rtk dotnet restore MyAiUsage.sln`; `rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` (4 projetos, 0 erros, 0 avisos); `rtk proxy powershell -NoProfile -Command "& '.\checks\MyAiUsage.Core.Checks\bin\x64\Release\net8.0-windows10.0.22000.0\MyAiUsage.Core.Checks.exe'"` (imprimiu `Tray callback check passed.` e `Core checks passed.`); `rtk git diff --check`. O executável x64 explícito elimina a divergência anterior com `bin\Release`.
- O pacote revisado foi gerado com `rtk dotnet msbuild MyAiUsage.sln /p:Configuration=Release /p:Platform=x64 /p:UapAppxPackageBuildMode=SideloadOnly /p:AppxBundle=Never /p:AppxPackageDir=artifacts\msix-round2\ /p:GenerateAppxPackageOnBuild=true` (exit code 0; somente o aviso opcional de `mspdbcmf.exe` ausente), assinado (exit code 0), verificado por `signtool verify /pa /all` (exit code 0) e desempacotado por `makeappx unpack` (exit code 0). MSIX: `src/MyAiUsage.App/artifacts/msix-round2/MyAiUsage.App_0.1.0.0_x64_Test/MyAiUsage.App_0.1.0.0_x64.msix`, SHA-256 `E55D124F015C1F31096D5E6566F4BF1F58B2BAC3D08F33915D474356188365A9`; manifest desempacotado SHA-256 `EA2D05F1C6CFEB7356F1F64DBED5C37B279DCCDC921E663CD2B4B7EC69F4EB40`, com `MyAiUsage.App.exe` resolvido.
- Hashes atuais dos arquivos de código alterados no novo loop: `checks/MyAiUsage.Core.Checks/Program.cs` `D257A4657FF065CFCBCA2761E0D2709864B685B5924F9106FBA6CFB18BA57FAC`; `src/MyAiUsage.App/StartupTaskManager.cs` `D8421A2DA609D801D6EDB7CBD356551FC4BE448E2937B8160659ED649ECCBE46`.
- Para o cenário permitido de diagnóstico, o marcador temporário `MyAiUsage.StartupTask.Unavailable` foi criado no `%TEMP%` e no `TempState` do pacote, e removido após a captura. UI Automation observou a janela WinUI empacotada real: processo `31040`, `Responding=True`, janela habilitada, status `Disponível`, `StartupToggle` com nome acessível `Iniciar My AI Usage com o Windows`, `ToggleState=Off`, `IsEnabled=False`, `Estado: indisponível` e motivo visível `O auto-start está indisponível neste cenário de diagnóstico.`; a UI de quotas permaneceu funcional. O pacote foi removido/reinstalado somente para executar este snapshot; não houve alteração de certificado ou política.

##### Acceptance criteria

- [x] O manifest fonte e o empacotado permanecem válidos e instaláveis, preservam `runFullTrust` e contêm exatamente uma extensão `windows.startupTask` com `TaskId="MyAiUsageStartup"`, `Executable="$targetnametoken$.exe"` na fonte e `MyAiUsage.App.exe` resolvido no pacote, `EntryPoint="Windows.FullTrustApplication"`, `Enabled="true"` e nome visível.
- [x] `StartupTaskManager` usa diretamente as APIs nativas e representa sem colapsar `Enabled`, `EnabledByPolicy`, `Disabled`, `DisabledByUser`, `DisabledByPolicy` e `Unavailable`.
- [x] Em `Disabled`, o toggle alterável chama `RequestEnableAsync()` e reflete o estado retornado; em `Enabled`, chama `Disable()` e relê/reflete o estado efetivo. No pacote, UIA observou `On/Enabled` → `Off/Enabled` → `On/Enabled` → `Off/Enabled`, com textos de estado/motivo atualizados, e restauração final para `On/Enabled`.
- [x] Em `EnabledByPolicy`, `DisabledByUser` e `DisabledByPolicy`, o toggle é somente leitura e o motivo/orientação fica visível; nenhuma tentativa inválida de mudança é feita. Evidência: check executável dos estados e inspeção de `ApplyStartupTaskState`/`OnStartupToggled`; reprodução nativa desses estados limitada conforme Verification, sem política artificial.
- [x] Fora do pacote ou se `StartupTask.GetAsync()` falhar, a UI mostra `Unavailable`, explica o motivo, desabilita o toggle e mantém o restante do app funcional. A captura UIA real do cenário de diagnóstico observou `Estado: indisponível`, motivo completo, `IsEnabled=False`, status `Disponível` e janela responsiva.
- [x] Toggle, estado e motivo têm nome acessível, estado perceptível e operação por teclado na janela existente; nenhuma página nova é criada. UIA encontrou `AutomationId=StartupToggle`, nome `Iniciar My AI Usage com o Windows`, `TogglePattern` e texto de estado/motivo; uma tecla `Space` real alternou `On/Enabled` → `Off/Enabled` → `On/Enabled`.
- [x] Uma ativação `ExtendedActivationKind.StartupTask` registra/reutiliza a instância única, cria tray e janela sem exibi-la nem ativá-la. Após reinício/login, o usuário não observou janela e observou o processo residente; a captura técnica confirmou PID único, tray e posterior restauração pelo callback. O enum de ativação não é exposto pelos eventos coletados e fica como limitação de observação, sem extrapolação.
- [x] Ativações comuns e redirecionadas continuam abrindo/restaurando a mesma janela; fechar continua ocultando, o tray restaura e `Sair` encerra app e app-server sem órfão. Evidência: capturas nativas e árvore de processos registradas em Verification e no pós-restart; código de lifecycle não alterado nesta correção.
- [x] A implementação permanece mínima e não adiciona Registry, serviço/background task, abstração, dependência, nova página ou trabalho da Issue #9.

##### Verification

- [x] `rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror` termina com exit code 0, zero erros e zero avisos no snapshot revisado.
- [x] O check mínimo executável cobre o mapeamento dos estados, a mutabilidade somente de `Enabled`/`Disabled` e o comportamento `Unavailable`, termina com exit code 0 e imprime `Core checks passed.`.
- [x] `rtk git diff --check` e a inspeção final de `git status`/diff terminam sem erro, preservam os dirty paths iniciais e as mudanças adicionais desta issue ficam limitadas ao hook mínimo de empacotamento no `.csproj`, ao novo `StartupTaskManager.cs` e ao spec.
- [x] Gerar e inspecionar o manifest empacotado/MSIX; registrar caminho, SHA-256 e resultado contra o mesmo snapshot de código, distinguindo o artefato removido de qualquer artefato realmente usado. O `dotnet msbuild` padrão termina com exit code 0; `signtool sign` e `signtool verify /pa /all` passaram; `makeappx unpack` terminou com sucesso e confirmou o executável resolvido. A geração opcional de símbolos avisou sobre `mspdbcmf.exe` ausente; após trust elevado, a instalação do pacote assinado terminou com exit code 0.
- [x] Instalar o pacote de desenvolvimento após autorização específica; registrar certificado/procedimento, comando, exit code, versão e identidade efetivamente instalada. O `.cer` público esperado foi importado via UAC em `Cert:\LocalMachine\TrustedPeople` (exit code 0), e `Add-AppxPackage` com `Microsoft.WindowsAppRuntime.2.msix` terminou com exit code 0; identidade `MyAiUsage_0.1.0.0_x64__5n9q33krdf9q0`, versão `0.1.0.0`, Family `MyAiUsage_5n9q33krdf9q0`, Status `Ok`.
- [x] No pacote instalado, registrar `Disabled` → `Enabled` → `Disabled`, incluindo texto, disponibilidade do toggle e estado final retornado pelo Windows. UI Automation observou `Off/Estado: desativado` → `On/Estado: ativado` → `Off/Estado: desativado`, sempre com toggle habilitado; uma restauração final por teclado deixou `On/Estado: ativado`.
- [x] Registrar estados de usuário/política quando reproduzíveis. Se `EnabledByPolicy`, `DisabledByUser` ou `DisabledByPolicy` não puderem ser produzidos no ambiente, registrar ambiente, estado observado e limitação honesta; a aceitação não exige criar política artificial. Neste Windows 10 Home, apenas `Enabled`/`Disabled` foram reproduzidos; nenhum estado de política foi forçado.
- [x] Executar fora do pacote ou reproduzir falha de acesso e confirmar `Unavailable`, motivo visível, toggle desabilitado e app funcional. O marcador temporário reproduziu a falha no pacote revisado e foi removido ao final.
- [x] Após autorização específica para reiniciar a sessão, confirmar que auto-start habilitado inicia uma única instância, um único tray e janela oculta; abrir pelo tray restaura a mesma janela. O usuário confirmou processo residente sem janela apresentada no login; a captura pós-restart registrou uma instância/tray e o ciclo de restauração, e `Sair` deixou zero processos do app.
- [x] Confirmar que início comum exibe/restaura; segunda ativação redireciona; fechar oculta; tray restaura; `Sair` encerra app e o `codex app-server` filho sem órfão. O lançamento normal ficou responsivo; segunda ativação manteve um PID; `WindowPattern.Close` ocultou sem encerrar; o ícone nativo foi encontrado por `Shell_NotifyIconGetRect` e o callback restaurou a janela; `Sair` encerrou o app e seus filhos `cmd.exe`/Node, com zero PIDs restantes.

##### Remediation

###### Histórico do loop anterior — Round 1 review

- [x] [CRITICAL] `StartupTaskManager.cs:81` colapsava `EnabledByPolicy` em `Enabled`; no snapshot atual o check executável mapeia `EnabledByPolicy` separadamente e verifica `CanChange=false`.

###### Histórico do loop anterior — Round 2 review

- [x] [NON_CRITICAL] `Package.appxmanifest:31` usava `MyAiUsage.App.exe`; restaurar exatamente `$targetnametoken$.exe` e revalidar fonte e empacotado. A fonte agora usa o token exigido; o hook `ResolveStartupTaskExecutable` resolve somente o manifest intermediário, e a inspeção do MSIX padrão confirmou `MyAiUsage.App.exe` resolvido.
- [x] [NON_CRITICAL] Separar a evidência do MSIX removido de `AppPackages` (`E1955AA22E4573E7D411A8238E35474D168A73BD298E02DD80BE75195A5DBE43`) do artefato em `artifacts/msix` usado na tentativa de instalação (`CA29F3F20226F0052B4DF30A3604E01CD7983E035B9C3DD0857E5088A70FE5E8`). O artefato atual assinado é `artifacts/msix-standard-fixed/MyAiUsage.App_0.1.0.0_x64_Test/MyAiUsage.App_0.1.0.0_x64.msix`, SHA-256 `5D7D74AEF8E80E65C68F48DABEB92A5A449206CEE550971796E2DD5318CC24DD`; os dois hashes históricos não foram reutilizados como evidência.

Snapshot histórico inválido para o novo PASS: `E1955...BE43` foi removido após inspeção e `CA29...FE5E8` foi usado na tentativa de instalação que falhou com `0x800B0100` por ausência de certificado. O pacote padrão atual foi assinado e separado, instalado após trust autorizado em `LocalMachine\TrustedPeople`, e os cenários nativos possíveis foram observados; permanecem pendentes somente a ativação real após logout/reinício, estados de política não forçados e a confirmação visual de `Unavailable` no executável unpackaged.

###### Round 1 review — novo loop

- [x] [CRITICAL] Produzir evidência executada da UI WinUI real em cenário unpackaged/falha de acesso: motivo `Unavailable` visível, toggle desabilitado e restante do app funcional; a captura UIA real de 2026-09-05 observou todos os estados exigidos.
- [x] [NON_CRITICAL] Tornar a sequência do runner reproduzível: o build x64 emite `checks\MyAiUsage.Core.Checks\bin\x64\Release`; a sequência final executou o `.exe` nesse caminho explícito e registrou exit code 0 e `Core checks passed.`.

###### Round 2 review — novo loop

- [ ] [CRITICAL] `src/MyAiUsage.App/StartupTaskManager.cs:19,32-36`: remover do build de produção o marcador previsível `%TEMP%\MyAiUsage.StartupTask.Unavailable`; manter qualquer injeção de falha somente em código de teste/compilação condicional e regenerar/revalidar o MSIX.
- [ ] [NON_CRITICAL] `docs/superpowers/plans/2026-09-01-my-ai-usage-v1.md:493,497`: reconciliar os dois critérios de aceitação ainda desmarcados com a evidência permitida registrada ou tratá-los explicitamente como não atendidos.

##### Delivery

Correção direta autorizada pelo usuário após o encerramento do loop: removidos completamente o marcador de diagnóstico e sua leitura do `StartupTaskManager`; o runner agora exercita a falha nativa real fora do pacote, sem criar arquivos de controle. Checkboxes de política e lifecycle reconciliados com as evidências acima. O verdict `REVISE` abaixo é histórico do loop encerrado; não representa revisão da correção direta. A evidência UIA com marcador é histórica e não deve ser apresentada como uma nova execução visual do pacote corrigido.

- Review verdict and evidence: `REVISE` no round 2/2; marcador de diagnóstico em produção e checkboxes inconsistentes permanecem acionáveis.
- Delivery status: blocked pelo limite de duas rodadas; nenhuma branch, commit, push ou PR foi criada.
- Pull request para `main`: pending.

---

### Task 5: Validar, empacotar e documentar a release V1

**Files:**
- Create: `docs/INSTALL.md`
- Modify: `README.md`
- Modify: `docs/ROADMAP.md`
- Modify: `docs/TECHNICAL_NOTES.md`
- Create during build only: `artifacts/msix/` (não versionar)

**Interfaces:**
- Consumes: a solução funcional e os critérios globais das Tasks 0–4.
- Produces: MSIX instalável, instruções reproduzíveis, matriz de evidência e decisão explícita de V1 pronta ou bloqueada.

- [ ] **Step 1: Executar a validação automatizada final.** Rode sem `--no-restore` na primeira execução:

```powershell
rtk dotnet restore MyAiUsage.sln
rtk dotnet build MyAiUsage.sln -c Release -p:Platform=x64 -warnaserror
rtk proxy powershell -NoProfile -Command "& '.\checks\MyAiUsage.Core.Checks\bin\x64\Release\net8.0-windows10.0.22000.0\MyAiUsage.Core.Checks.exe'"
rtk git diff --check
```

Esperado: todos os comandos terminam com exit code 0 e o check imprime `Core checks passed.`.
- [ ] **Step 2: Gerar o MSIX.** Em um Developer PowerShell que tenha `msbuild` no `PATH`, execute:

```powershell
rtk proxy msbuild MyAiUsage.sln /t:Restore
rtk proxy msbuild MyAiUsage.sln /p:Configuration=Release /p:Platform=x64 /p:UapAppxPackageBuildMode=SideloadOnly /p:AppxBundle=Never /p:AppxPackageDir=artifacts\msix\ /p:GenerateAppxPackageOnBuild=true
```

Esperado: um `.msix` em `artifacts/msix/`, sem certificado, chave privada ou payload de conta no repositório. Se `msbuild` não estiver disponível, executar os mesmos parâmetros no Developer PowerShell do Visual Studio instalado e registrar a versão usada.
- [ ] **Step 3: Verificar instalação limpa e o PATH.** Em uma VM ou máquina Windows 11 sem o app instalado, instalar o pacote e certificado de desenvolvimento conforme a instrução gerada pelo template. Confirmar que o processo gráfico encontra o `codex` real no `PATH`, que o app-server não abre console e que desinstalar/reinstalar remove a associação anterior sem deixar startup task órfã.
- [ ] **Step 4: Executar a matriz manual.** Registrar cada caso em `docs/TECHNICAL_NOTES.md`:

| Caso | Resultado obrigatório |
|---|---|
| Codex autenticado | Todos os buckets/janelas válidos, percentual e reset local |
| Sem autenticação | `Desconectado`, ação para autenticar no Codex, sem payload bruto |
| `codex` ausente | `Codex ausente`, sem stack trace |
| Rede/timeout | `Falha temporária`; último snapshot permanece com hora |
| JSON inválido/EOF | erro classificado; processo encerrado sem órfão |
| Resposta parcial | `Dados parciais`; último snapshot completo não é apagado |
| 0/49/50/79/80/100% | cores e textos nos limites definidos; 100% mostra limite atingido |
| Dois buckets e janela extra | tudo aparece, sem soma/agregação |
| Teclado/Narrator | refresh, tray/configuração e cada anel têm nome e estado |
| Escala 100/125/150% e tema claro/escuro | sem corte, contraste legível, layout utilizável |
| Segunda instância | ativação é encaminhada à primeira |
| Explorer reiniciado | ícone é registrado novamente |
| Painel oculto por 5 min | working set ≤100 MB, CPU média <2%, sem polling |
| Painel aberto por 5 min | working set ≤200 MB e refresh sem sobreposição |

- [ ] **Step 5: Documentar instalação e limites.** Em `docs/INSTALL.md`, explicar Windows 11, Codex instalado/autenticado, instalação MSIX, certificado de desenvolvimento, auto-start, tray, atualizar, sair, desinstalar e solução para Codex ausente. Explicar que o app não envia dados a servidores próprios, mas o Codex local pode acessar os serviços dele.
- [ ] **Step 6: Atualizar o roadmap somente com evidência.** Marcar itens concluídos apenas quando a matriz tiver resultado registrado; manter pendente qualquer caso não reproduzido e listar bloqueios com causa e próximo comando.
- [ ] **Step 7: Fazer a revisão final de segurança e escopo.** Execute:

```powershell
rtk rg -n -- 'auth\.json|token|email|payload|secret|password' src checks README.md docs
rtk git status --short
```

Confirmar que ocorrências são apenas documentação de proibição/checagem, que nenhum artefato de build é rastreado e que `.serena/` permanece fora do commit.
- [ ] **Step 8: Commitar a release documental.** Execute:

```powershell
rtk git add README.md docs\INSTALL.md docs\ROADMAP.md docs\TECHNICAL_NOTES.md
rtk git diff --cached --check
rtk git diff --cached --name-status
rtk git commit -m "docs: record My AI Usage v1 release evidence"
```

**Saída:** a release só é chamada de pronta quando os checks, o MSIX, a instalação limpa e a matriz manual têm evidência registrada; caso contrário, a documentação deixa o bloqueio explícito.

## Fora desta execução

- Segundo provedor, histórico, custo de API, gráficos e notificações configuráveis.
- Backend, telemetria, sincronização, banco de dados, login próprio e múltiplas contas.
- Sistema de plugins, `IProvider`, fábrica ou abstração genérica de transporte.
- Auto-update do aplicativo e instalador comercial; a V1 entrega o MSIX reproduzível e instruções de instalação.
- Garantia absoluta contra qualquer descendente externo: a V1 usa `Kill(entireProcessTree: true)` e registra a limitação; Job Object só entra se um teste reproduzir órfãos.

## Critério de parada

Se o shell WinUI, o `CodexClient`, o parser, o tray ou o encerramento falhar, corrigir esse bloco e repetir sua validação antes de adicionar polimento visual, instalação comercial ou qualquer segundo provedor.
