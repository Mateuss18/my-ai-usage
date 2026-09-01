# Produto

## Visão geral

My AI Usage é um pequeno aplicativo para Windows que responde, sem exigir que o usuário abra uma CLI ou dashboard: **quanto da minha quota do Codex já usei e quando ela renova?**

O projeto também serve como exercício prático de C#, .NET e desenvolvimento desktop nativo para Windows.

## Problema

Quem alterna entre agentes de programação precisa consultar interfaces diferentes para saber se ainda possui capacidade disponível. A informação costuma ficar fora do fluxo de trabalho, e “usage” pode significar coisas distintas:

- quota disponível dentro de uma janela;
- horário de renovação da janela;
- tokens consumidos ao longo do tempo;
- custo de API.

A V1 resolve apenas os dois primeiros itens. Tokens históricos podem ser adicionados depois usando uma fonte oficial; custo de API não faz parte do escopo inicial.

## Público

### Primário

Desenvolvedores Windows que usam o Codex com autenticação ChatGPT e querem conferir suas janelas de quota rapidamente.

### Secundário

Usuários de vários agentes que, no futuro, poderão comparar a disponibilidade de cada provedor em um único lugar.

## Objetivo da V1

Entregar um utilitário confiável e discreto que:

1. detecte se o Codex está disponível;
2. use a sessão já gerenciada pelo Codex;
3. apresente as janelas retornadas pelo serviço;
4. mostre percentual utilizado e próxima renovação no fuso local;
5. continue útil quando parte dos dados estiver indisponível.

## Critérios de sucesso

- O usuário entende seu estado de quota em até cinco segundos após abrir o painel.
- O aplicativo não pede que tokens ou chaves sejam copiados para ele.
- Nenhum dado do usuário é enviado a servidores do My AI Usage.
- Ausência do Codex, logout e falhas de comunicação produzem mensagens acionáveis.
- O aplicativo permanece leve quando o painel está fechado.
- Os valores exibidos correspondem à resposta do Codex, sem inferências silenciosas.

## Escopo

### Incluído

- Windows 11 inicialmente.
- Codex autenticado por uma modalidade compatível com os serviços Codex.
- Janela compacta e acesso pela bandeja.
- Uma conta ativa, a mesma usada pelo Codex local.
- Percentual usado, duração da janela e horário de renovação.
- Atualização automática moderada e botão para atualizar.
- Tema claro/escuro seguindo o Windows, se coberto pelo comportamento nativo do WinUI.

### Fora da V1

- Claude, OpenCode, Cursor ou outros provedores.
- Troca e gerenciamento de contas.
- Login próprio do My AI Usage.
- Consumo ou alteração de créditos.
- Cálculo de preços ou custo de API.
- Gráficos e histórico local.
- Notificações configuráveis.
- Sincronização entre dispositivos.
- Serviço em nuvem, telemetria e atualização automática do aplicativo.

## Experiência essencial

1. O usuário inicia o aplicativo.
2. O aplicativo inicia e inicializa o `codex app-server` local.
3. O painel consulta a conta e as quotas.
4. Cada janela disponível aparece com percentual usado e renovação.
5. O usuário fecha o painel; o aplicativo permanece acessível pela bandeja.
6. Uma atualização periódica ou manual consulta novamente os valores.

## Estados necessários

- Carregando.
- Quota disponível.
- Limite atingido.
- Codex não encontrado.
- Conta desconectada ou autenticação incompatível.
- Dados parcialmente indisponíveis.
- Falha temporária, com opção de tentar novamente.

## Roadmap orientativo

### V1 — Codex quota

Validar integração, painel e utilidade cotidiana.

### V1.1 — Polimento

Instalador, inicialização opcional com o Windows, acessibilidade e notificações simples, conforme feedback real.

### V2 — Segundo provedor

Escolher Claude ou OpenCode conforme demanda e disponibilidade de uma fonte confiável. Somente aqui extrair uma abstração comum entre provedores.

### Futuro possível

Histórico oficial de tokens, comparação entre provedores e alertas de proximidade do limite.

## Riscos de produto

- Um app para apenas um provedor pode não justificar ficar residente; a V1 deve provar uso recorrente antes de crescer.
- APIs e formatos podem mudar. O aplicativo precisa identificar dados desconhecidos e falhar de forma explícita.
- “Quota usada” e “tokens usados” não são equivalentes. A interface e a documentação não devem misturá-los.
- Um painel bonito mas pesado contraria a proposta. Tempo de inicialização e memória devem ser observados desde o primeiro protótipo.

## Referências de produto

- [llmquota](https://github.com/0xNyk/llmquota): comparação de quotas entre CLIs e apresentação explícita de fontes incertas.
- [agent-notch](https://github.com/adityarya24/agent-notch): referência de interação compacta e sempre disponível; não de arquitetura Windows.
- [end4-pC](https://github.com/eoNaho/end4-pC): referência visual para superfícies desktop personalizadas.
- [QuotaScope](https://github.com/EvanHexX/quota-scope): exemplo próximo do problema específico de bandeja no Windows.

Esses projetos são referências, não dependências. Código e identidade visual não devem ser copiados sem revisar suas licenças e objetivos.
