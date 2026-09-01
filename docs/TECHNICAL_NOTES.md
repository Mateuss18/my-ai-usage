# Notas técnicas

## Decisão recomendada

Construir a V1 em C# e .NET com WinUI 3, usando o processo oficial `codex app-server` como fonte dos dados.

Essa escolha atende ao objetivo de aprendizado e mantém o produto Windows-first. WPF teria um caminho mais maduro para alguns comportamentos de bandeja, mas trocar de stack antes de encontrar um bloqueio real reduziria o valor de aprendizado pretendido.

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
7. Continuar lendo notificações até o encerramento do aplicativo.
8. Encerrar o processo filho de maneira controlada.

Uma consulta periódica simples, por exemplo a cada 60 segundos, é suficiente como fallback. Antes de fixar o intervalo, verificar se as notificações cobrem as mudanças relevantes sem polling contínuo.

## Bandeja e janela

WinUI 3 não oferece toda a experiência de bandeja como um controle pronto de alto nível. A implementação provavelmente precisará das APIs Win32 de notification area ou de uma pequena biblioteca já consolidada.

Decisão para o primeiro spike:

1. provar a chamada ao app-server em um processo de console C#;
2. provar uma janela WinUI que abre e fecha corretamente;
3. só então implementar o ícone de bandeja pelo menor caminho confiável.

Não escolher uma dependência de tray antes desse spike. Se poucas chamadas Win32 resolverem com segurança, não adicionar pacote; se o ciclo de vida ficar complexo, usar uma biblioteca pequena e mantida é mais barato que possuir código nativo frágil.

## Distribuição

Para desenvolvimento, começar com o modelo padrão gerado pelas ferramentas atuais do WinUI 3. A escolha entre aplicação empacotada e não empacotada deve ser feita após validar:

- instalação e atualização;
- localização do `codex` no `PATH` real do processo gráfico;
- inicialização com o Windows;
- comportamento do tray após reinício do Explorer;
- tamanho e simplicidade da distribuição.

MSIX, instalador alternativo e auto-update não pertencem ao primeiro spike.

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
- Medição simples de memória quando o painel estiver fechado.
- Verificação de teclado, leitor de tela, contraste e escala do Windows antes de chamar a V1 de pronta.

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

## Questões para a próxima decisão

1. O painel será uma janela comum compacta ou um flyout ancorado ao ícone da bandeja?
2. A V1 suporta somente Windows 11 ou também Windows 10?
3. O app deve iniciar com o Windows já na V1 ou isso fica para o polimento?
4. Qual métrica de leveza será adotada para memória e tempo de abertura?

Essas decisões devem ser fechadas antes do scaffold, não antecipadas por uma arquitetura genérica.
