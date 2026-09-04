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

- [ ] **Step 1: Escrever checks de apresentação.** Antes do XAML final, criar no check project funções puras para título e cor e verificar:

```csharp
Assert(WindowTitle(300) == "Janela de 5 horas", "formats hours");
Assert(WindowTitle(10080) == "Janela de 7 dias", "formats days");
Assert(UsageColor(49) == "green", "uses green below 50");
Assert(UsageColor(80) == "red", "uses red at 80");
Assert(UsageColor(null) == "neutral", "uses neutral for unknown");
```

- [ ] **Step 2: Implementar `QuotaRing`.** Usar um `Ellipse` neutro e um traço de progresso calculado por `StrokeDashArray`/`StrokeDashOffset`; expor apenas `RateLimitWindow Window`, `string BucketName` e `string AccessibleDescription`. Percentual nulo usa o traço neutro. O título deriva de minutos: dias quando divisível por 1440, horas quando divisível por 60 e minutos nos demais casos; duração ausente usa `Janela sem duração informada`.
- [ ] **Step 3: Implementar a grade e os textos equivalentes.** `MainWindow.xaml` deve renderizar todos os buckets/janelas sem agregação. Cada anel deve anunciar bucket, título, percentual ou `uso desconhecido`, reset ou `reset desconhecido` e estado. Mostrar sempre texto fora do anel; cor não pode ser a única informação.
- [ ] **Step 4: Implementar os estados.** Mapear `CodexClientErrorKind` para `Codex ausente`, `Desconectado`, `Falha temporária`, `Dados parciais` e `Atualização cancelada`. Exibir `Carregando` no primeiro refresh, `Disponível` com snapshot completo e `Limite atingido` por janela em 100%.
- [ ] **Step 5: Implementar o refresh único.** Centralizar botão e timer em `RefreshAsync` e proteger com `SemaphoreSlim(1, 1)`:

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
- [ ] **Step 6: Configurar o timer sem custo oculto.** O timer de 60 segundos roda somente enquanto a janela estiver visível; ao ocultar, parar o timer; ao restaurar, iniciar e atualizar se o snapshot tiver mais de 60 segundos. O app-server permanece controlado pelo app e não há polling enquanto o painel estiver oculto.
- [ ] **Step 7: Validar UI e checks.** Execute os checks Core, `rtk dotnet build MyAiUsage.sln -c Debug -p:Platform=x64 -warnaserror`, e teste manualmente todos os buckets, campos ausentes, 49/50/79/80/100%, resposta parcial e falha após snapshot válido.
- [ ] **Step 8: Commitar o painel.** Execute `rtk git diff --check`, depois:

```powershell
rtk git add src\MyAiUsage.App checks\MyAiUsage.Core.Checks
rtk git diff --cached --check
rtk git commit -m "feat: show Codex quota states"
```

**Saída:** o painel apresenta cada janela com informação equivalente para teclado/leitor de tela e nunca apaga valor válido por causa de erro ou resposta parcial.

#### Remediation

##### Round 1 review

- [x] [NON_CRITICAL] Corrigir literais visíveis com UTF-8 corrompido em `MainWindow.xaml`, `MainWindow.xaml.cs` e `QuotaRing.xaml.cs`.
- [x] [NON_CRITICAL] Fazer os checks executáveis chamarem a lógica de apresentação de produção e cobrirem estados disponível, limite atingido e desconhecido.
- [ ] [NON_CRITICAL] Executar e registrar a validação manual de buckets, campos ausentes, limites, resposta parcial e falha após snapshot válido.
  - Parcial (2026-09-03): o usuário confirmou que o app MSIX abriu pelo Visual Studio; os cenários de dados e falha ainda não foram exercitados individualmente.

#### Delivery

- [ ] Final review: PASS
- [ ] Merge request created

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
rtk dotnet run --project checks\MyAiUsage.Core.Checks\MyAiUsage.Core.Checks.csproj -c Release --no-build
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
