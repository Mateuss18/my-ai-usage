# Notas técnicas

## Decisão recomendada

Construir a V1 em C# e .NET com WinUI 3, usando o processo oficial `codex app-server` como fonte dos dados.

Essa escolha atende ao objetivo de aprendizado e mantém o produto Windows-first. WPF teria um caminho mais maduro para alguns comportamentos de bandeja, mas trocar de stack antes de encontrar um bloqueio real reduziria o valor de aprendizado pretendido.

## Contrato fechado da V1

- A distribuição é MSIX.
- O auto-start padrão é habilitado e pode ser desligado pelo usuário; quando ativado pelo Windows, o app inicia oculto.
- O tray abre ou restaura a janela compacta.
- Acessibilidade básica — teclado, nomes acessíveis, contraste e não depender só de cor — pertence à V1.
- Polling serializado de 60 segundos é a decisão V1 para notificações. Refresh manual e timer compartilham a mesma leitura e não podem sobrepor requisições; notificações do app-server são consumidas e ignoradas.
- Instalador mais amigável, auto-update e polimento avançado pertencem à V1.1.
- Limites de release: painel utilizável em até 5 s, até 100 MB de working set oculto após 5 min, até 200 MB aberto após 5 min e CPU média abaixo de 2% oculto fora do refresh.

## Fonte de dados do Codex

O Codex App Server expõe uma interface JSON-RPC bidirecional. O cliente inicia `codex app-server` e conversa com o processo por `stdin` e `stdout`, com uma mensagem JSON por linha.

Para a V1, as operações relevantes são:

- `initialize`: inicializa a conexão antes das demais chamadas.
- `account/read`: informa o estado da conta atual.
- `account/rateLimits/read`: retorna as janelas de quota do ChatGPT.
- `account/rateLimits/updated`: notifica mudanças nas quotas enquanto o processo está ativo.

A resposta de rate limits pode incluir:

- `rateLimits`: visão compatível de um único bucket;
- `rateLimitsByLimitId`: vários buckets identificados pelo serviço;
- `usedPercent`: percentual já utilizado;
- `windowDurationMins`: duração da janela;
- `resetsAt`: Unix timestamp em segundos;
- `planType`, créditos e estado de limite, quando disponíveis.

Há também `account/usage/read` para resumo e buckets diários de tokens. Ele é tecnicamente possível, mas fica fora da V1 para não misturar histórico de tokens com quota corrente.

Documentação oficial: [Codex App Server](https://developers.openai.com/codex/app-server).

## Por que não ler `auth.json`

Projetos de referência obtêm quota lendo credenciais locais e acessando endpoints do provedor. Isso pode funcionar, mas cria responsabilidades desnecessárias:

- manipulação direta de segredos;
- formatos locais não documentados;
- renovação de tokens;
- dependência de endpoints que podem mudar.

O app-server já pertence ao Codex e encapsula autenticação e comunicação com os serviços. A V1 deve reutilizá-lo e nunca registrar mensagens que possam conter dados de conta ou autenticação.

## Arquitetura mínima

```text
WinUI 3 UI
    |
    v
CodexClient
    |
    v
codex app-server
    |
    v
Serviços Codex
```

Componentes suficientes para começar:

- `MainWindow`: mostra o painel e seus estados.
- `CodexClient`: controla o processo, envia JSON-RPC e recebe respostas/notificações.
- Modelos de resposta contendo apenas os campos usados pela interface.

Não criar agora `IProvider`, fábrica, plugin system, banco, servidor local ou camada genérica de transporte. Um segundo provedor mostrará qual abstração realmente existe.

## Fluxo de dados

1. Localizar o executável `codex` no ambiente do usuário.
2. Iniciar `codex app-server` sem janela de console.
3. Enviar `initialize` e aguardar a resposta.
4. Consultar `account/read` e `account/rateLimits/read`.
5. Converter timestamps para o fuso local apenas na camada de apresentação.
6. Atualizar a interface no dispatcher do WinUI.
7. Consumir e ignorar notificações do app-server até o encerramento; o polling serializado faz a próxima leitura a cada 60 segundos.
8. Encerrar o processo filho de maneira controlada.

O polling serializado de 60 segundos é a decisão V1 para notificações. Uma leitura manual ou do timer deve aguardar a leitura em andamento; nunca executar duas consultas simultâneas. O intervalo poderá mudar somente com evidência de uso e custo.

## Bandeja e janela

WinUI 3 não oferece toda a experiência de bandeja como um controle pronto de alto nível. A implementação provavelmente precisará das APIs Win32 de notification area ou de uma pequena biblioteca já consolidada.

Decisão para o primeiro spike:

1. provar a chamada ao app-server em um processo de console C#;
2. provar uma janela WinUI que abre e fecha corretamente;
3. implementar o tray da V1 pelo menor caminho confiável.

Não escolher uma dependência de tray antes desse spike. Se poucas chamadas Win32 resolverem com segurança, não adicionar pacote; se o ciclo de vida ficar complexo, usar uma biblioteca pequena e mantida é mais barato que possuir código nativo frágil.

## Distribuição

Para desenvolvimento, começar com o modelo padrão gerado pelas ferramentas atuais do WinUI 3. A distribuição da V1 será em MSIX, após validar:

- instalação e atualização;
- localização do `codex` no `PATH` real do processo gráfico;
- auto-start padrão e início oculto quando ativado pelo Windows;
- comportamento do tray após reinício do Explorer;
- tamanho e simplicidade da distribuição.

Instalador mais amigável, auto-update e polimento avançado não são gates da V1; ficam para a V1.1. O primeiro spike não prova a distribuição, o auto-start ou o tray.

## Segurança e privacidade

- Não ler, copiar ou persistir credenciais do Codex.
- Não enviar telemetria.
- Não registrar payloads completos por padrão.
- Tratar toda saída do processo como entrada não confiável: validar JSON e campos opcionais.
- Executar somente o comando esperado, sem montar linha de comando a partir de dados remotos.
- Manter a V1 read-only: não expor operações que consumam créditos ou alterem a conta.
- Documentar claramente que o app inicia um processo local do Codex e que o Codex pode acessar seus próprios serviços.

## Erros esperados

- Executável `codex` ausente.
- Processo falha ao iniciar ou encerra inesperadamente.
- Protocolo incompatível após uma atualização.
- Conta sem login ou modo de autenticação sem suporte aos endpoints de uso.
- Resposta parcial, bucket desconhecido ou campo nulo.
- Rede indisponível.
- Timeout.

O painel deve manter o último valor apenas se ele estiver marcado com o horário da última atualização. Sem valor anterior, mostrar estado desconhecido; nunca converter falha em `0% usado`.

## Verificação mínima

- Teste do parser com uma resposta contendo um bucket e outra com múltiplos buckets.
- Teste dos estados de campo ausente e JSON inválido.
- Teste manual com Codex autenticado, deslogado e ausente do `PATH`.
- Medição de abertura, working set e CPU contra os limites: painel utilizável em até 5 s, até 100 MB oculto após 5 min, até 200 MB aberto após 5 min e CPU média abaixo de 2% oculto fora do refresh.
- Verificação de teclado, nomes acessíveis, contraste, não depender só de cor e escala do Windows antes de chamar a V1 de pronta.

## Matriz de evidências e limites

| Cenário | Comando/ação | Máquina/SDK | Data | Resultado | Limite | Limitação conhecida |
| --- | --- | --- | --- | --- | --- | --- |
| Restore do baseline | `rtk dotnet restore spike\MyAiUsage.Console.Checks\MyAiUsage.Console.Checks.csproj` | Windows 10.0.26200.0 / .NET SDK 10.0.400 | 2026-09-01 | Reproduzido: 2 projetos, 0 erros, 0 avisos | Restore sem erros ou avisos | Não prova o comportamento da aplicação WinUI, MSIX ou produção |
| Build do baseline | `rtk dotnet build spike\MyAiUsage.Console.Checks\MyAiUsage.Console.Checks.csproj -c Release -warnaserror` | Windows 10.0.26200.0 / .NET SDK 10.0.400 | 2026-09-01 | Reproduzido: 2 projetos, 0 erros, 0 avisos | 0 erros e 0 avisos | Compila somente o spike Console e seus checks |
| Checks executáveis antes da extensão | `rtk dotnet run --project spike\MyAiUsage.Console.Checks\MyAiUsage.Console.Checks.csproj -c Release --no-build` | Windows 10.0.26200.0 / .NET SDK 10.0.400 | 2026-09-01 | Reproduzido: `Process configuration checks passed.` | Linha de sucesso do spike | Não prova tray físico, auto-start, MSIX, acessibilidade ou desempenho de release |
| Fluxo Codex autenticado | Execução anterior com sessão Codex real autenticada | Ambiente histórico; SDK e máquina não reproduzidos nesta sessão | Histórica; não reproduzida na Task 0 | Evidência histórica, não confirmada agora | Reproduzir autenticado, deslogado e Codex ausente do `PATH` antes da release | Não usar como evidência atual nem registrar credenciais, e-mail ou payload |
| Desempenho da aplicação | Medir após o scaffold com painel oculto e aberto por 5 min | Não aplicável ao spike atual | Pendente | Não medido | Painel em até 5 s; working set até 100 MB oculto e 200 MB aberto; CPU média abaixo de 2% oculto fora do refresh | O spike não tem UI WinUI, tray, MSIX ou ciclo de vida da aplicação |

O build e a execução autenticada anteriores que não foram reproduzidos são históricos. Nesta Task 0, somente as três linhas de baseline acima foram reproduzidas; nenhum resultado de produção é inferido a partir delas.

## Achados nas referências

### llmquota

Boa referência para distinguir provedores e mostrar percentual, reset e incerteza. Ele próprio alerta que algumas fontes não são oficiais e podem mudar. Para Codex, o My AI Usage deve preferir o app-server oficial em vez de reproduzir a leitura de `~/.codex/auth.json`.

### agent-notch

O valor relevante é a ideia de uma superfície pequena, contextual e rapidamente acessível. A implementação é voltada a macOS e não deve orientar a arquitetura Windows.

### end4-pC

Serve como inspiração de personalização visual do desktop. A V1 não deve copiar sua complexidade visual antes de provar legibilidade, tray e consumo de recursos.

### Projetos Windows semelhantes

Já existem utilitários Windows para Codex e Claude. Isso valida o problema, mas também significa que o diferencial não pode ser apenas “mostrar duas barras”. Os diferenciais iniciais mais defensáveis são:

- integração oficial do Codex;
- aplicação realmente Windows-first;
- código pequeno, compreensível e útil para colaboradores C#;
- comportamento local-first e transparente.

## Decisões fechadas antes do scaffold

1. O painel será uma janela comum compacta, aberta ou restaurada pelo tray.
2. A V1 suporta Windows 11.
3. O auto-start padrão e o início oculto fazem parte da V1.
4. O desempenho será avaliado pelos limites registrados na matriz: abertura em até 5 s, working set de até 100 MB oculto e 200 MB aberto após 5 min, e CPU média abaixo de 2% oculto fora do refresh.
