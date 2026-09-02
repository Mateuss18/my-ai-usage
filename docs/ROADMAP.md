# Roadmap

Este documento acompanha a execução do My AI Usage. Um item só deve ser marcado como concluído quando houver código ou documentação correspondente e uma verificação compatível com o risco.

## Estado atual

### Concluído

- [x] Definir produto, público, escopo e critérios de sucesso da V1 em `PRODUCT.md`.
- [x] Registrar decisões, riscos e protocolo inicial em `TECHNICAL_NOTES.md`.
- [x] Criar um spike Console em C# e .NET 8, sem dependências externas.
- [x] Iniciar `codex app-server` pelo `PATH` no Windows com `stdin`, `stdout` e `stderr` redirecionados.
- [x] Encerrar a árvore do processo filho controlado pelo aplicativo.
- [x] Implementar mensagens JSON por linha com `System.Text.Json`.
- [x] Enviar `initialize`, a notificação `initialized` e `account/rateLimits/read`.
- [x] Correlacionar respostas por `id`, ignorar notificações intermediárias e detectar EOF, JSON inválido, timeout e erro do app-server.
- [x] Ler `rateLimitsByLimitId` e usar `rateLimits` como fallback.
- [x] Exibir janelas primária e secundária com percentual usado, duração e reset no fuso local.
- [x] Tratar campos ausentes como desconhecidos, sem convertê-los silenciosamente em zero.
- [x] Criar checks executáveis para configuração do processo, ciclo de vida, JSON-RPC, erro do servidor, múltiplos buckets e resposta parcial.
- [x] Validar build sem erros ou avisos.
- [x] Validar o fluxo completo com uma sessão Codex real autenticada (evidência histórica; não reproduzida na Task 0).

### Estado do Git

`.serena/` é um artefato local não rastreado e fica fora do commit da Task 0. O commit da tarefa inclui somente o contrato documental e os checks do spike.

## Contrato fechado da V1 (Task 0)

- A distribuição V1 será em MSIX.
- O auto-start padrão é habilitado, pode ser desligado pelo usuário e inicia o app oculto quando acionado pelo Windows.
- O tray abre ou restaura a janela compacta.
- Acessibilidade básica — teclado, nomes acessíveis, contraste e não depender só de cor — é requisito V1.
- Polling serializado de 60 segundos é a decisão V1 para notificações; refresh manual e timer não podem fazer leituras simultâneas.
- O gate de desempenho é painel utilizável em até 5 s, até 100 MB de working set oculto após 5 min, até 200 MB aberto após 5 min e CPU média abaixo de 2% oculto fora do refresh.
- Instalador mais amigável, auto-update e polimento avançado ficam para a V1.1.

## Próximo passo imediato — concluir o spike

- [ ] Executar com o Codex instalado e a conta deslogada; confirmar erro identificável e acionável.
- [ ] Executar sem Codex no `PATH`; confirmar mensagem de instalação/configuração, sem stack trace para o usuário.
- [ ] Confirmar o comportamento com rede indisponível e timeout.
- [ ] Confirmar que nenhuma saída registra credenciais, e-mail ou payload completo da conta.
- [ ] Registrar o resultado do spike: o que pode ser reaproveitado e o que deve ser descartado.

Critério de saída: os três cenários principais — autenticado, deslogado e Codex ausente — têm comportamento conhecido e o contrato mínimo do futuro `CodexClient` está provado.

## V1 — quota do Codex no Windows 11

### 1. Preparar a solução

- [ ] Decidir se o spike será adaptado ou substituído; reaproveitar apenas o protocolo e o formatter que continuarem pequenos.
- [ ] Criar a solução e o aplicativo WinUI 3 pelo template oficial atual.
- [ ] Adicionar `.gitignore` para artefatos do Visual Studio e .NET.
- [ ] Manter uma única integração `CodexClient`; não criar `IProvider`, fábrica ou sistema de plugins na V1.
- [ ] Preservar o spike até o WinUI reproduzir o fluxo autenticado; removê-lo quando deixar de agregar valor.

### 2. Provar a janela WinUI

- [ ] Abrir e fechar uma janela compacta sem falha.
- [ ] Implementar e validar a janela compacta comum, aberta ou restaurada pelo tray.
- [ ] Seguir tema claro/escuro e escala do Windows usando comportamento nativo sempre que possível.
- [ ] Não implementar tray, animações ou personalização visual antes de a janela básica funcionar.

### 3. Integrar o Codex

- [ ] Mover o controle do processo e JSON-RPC para um `CodexClient` com ciclo de vida explícito.
- [ ] Inicializar o app-server uma vez e mantê-lo ativo enquanto o aplicativo estiver em execução.
- [ ] Consultar a conta quando necessário para distinguir usuário deslogado de falha temporária.
- [ ] Consultar todas as janelas retornadas por `account/rateLimits/read`.
- [ ] Consumir `account/rateLimits/updated` ou refazer a leitura quando a notificação for esparsa.
- [ ] Implementar atualização manual e polling serializado de 60 segundos; notificações do app-server são consumidas e ignoradas, e a próxima leitura ocorre no intervalo definido.
- [ ] Encerrar o processo filho ao sair do aplicativo.

### 4. Entregar o painel funcional

- [ ] Mostrar cada bucket e suas janelas disponíveis.
- [ ] Mostrar percentual usado, duração e reset no fuso local.
- [ ] Mostrar horário da última atualização.
- [ ] Implementar os estados: carregando, disponível, limite atingido, Codex ausente, conta desconectada, dados parciais e falha temporária.
- [ ] Manter o último valor somente quando acompanhado do horário da última atualização.
- [ ] Oferecer ação de tentar novamente nos erros recuperáveis.
- [ ] Nunca apresentar falha ou valor ausente como `0% usado`.

### 5. Implementar o tray

- [ ] Validar primeiro o ciclo de vida da janela sem tray.
- [ ] Implementar o tray por chamadas Win32 mínimas ou uma biblioteca pequena e mantida, conforme o menor caminho confiável.
- [ ] Abrir ou restaurar o painel pelo ícone.
- [ ] Fechar o painel sem encerrar o processo quando essa for a interação esperada.
- [ ] Disponibilizar comando explícito para sair e liberar o app-server.
- [ ] Verificar restauração do ícone após reinício do Explorer.

### 6. Confiabilidade, segurança e privacidade

- [ ] Não ler nem persistir `auth.json`, tokens ou chaves.
- [ ] Não enviar telemetria nem dados para servidores do My AI Usage.
- [ ] Não registrar payloads completos do app-server por padrão.
- [ ] Validar JSON, campos opcionais, timestamps e buckets desconhecidos.
- [ ] Exibir mensagens acionáveis para executável ausente, logout, rede, timeout, protocolo incompatível e processo encerrado.
- [ ] Manter todas as operações read-only; não consumir créditos nem alterar conta.
- [ ] Evitar sobreposição de refresh manual, polling e notificações.

### 7. Verificação da V1

- [ ] Automatizar os casos de parser: bucket único, múltiplos buckets, campos nulos, JSON inválido e erro JSON-RPC.
- [ ] Testar manualmente Codex autenticado, deslogado e ausente do `PATH`.
- [ ] Testar abertura, fechamento, tray, atualização, saída e reinício do Explorer.
- [ ] Verificar teclado, foco, leitor de tela, contraste e escalas comuns do Windows.
- [ ] Medir tempo de abertura e memória com o painel aberto e fechado; aprovar somente com painel utilizável em até 5 s, até 100 MB de working set oculto após 5 min, até 200 MB aberto após 5 min e CPU média abaixo de 2% oculto fora do refresh.
- [ ] Testar em uma instalação limpa do Windows 11 sem ambiente de desenvolvimento.
- [ ] Confirmar que falha de rede e resposta parcial não apagam silenciosamente o último valor válido.

### 8. Distribuição e conclusão da V1

- [ ] Produzir a distribuição V1 em MSIX depois de validar `PATH`, tray e instalação limpa.
- [ ] Produzir um artefato executável reproduzível e instruções de instalação/remoção.
- [ ] Documentar a dependência do Codex instalado e autenticado.
- [ ] Atualizar README, limitações conhecidas e política local-first.
- [ ] Executar a matriz final de testes e registrar resultados.
- [ ] Criar a release V1 somente quando todos os critérios abaixo estiverem satisfeitos.

## Critérios para considerar a V1 pronta

- O painel permite entender a quota em até cinco segundos após abrir.
- Todas as janelas fornecidas pelo Codex são exibidas sem inferências silenciosas.
- Codex ausente, logout, rede indisponível, timeout e resposta parcial têm estados distintos e acionáveis.
- Atualização manual, atualização automática, tray e encerramento funcionam sem deixar processos órfãos.
- Nenhuma credencial é lida, persistida ou registrada pelo aplicativo.
- O aplicativo é utilizável por teclado, leitor de tela e nas escalas suportadas do Windows 11.
- Tempo de abertura e memória medidos atendem aos limites definidos para a release.
- O artefato funciona em uma máquina limpa e possui instruções suficientes para instalação e remoção.

## V1.1 — polimento orientado por uso real

- [ ] Avaliar instalador mais amigável.
- [ ] Avaliar atualização automática.
- [ ] Avaliar notificações simples de proximidade do limite.
- [ ] Refinar visual, tray e consumo de recursos como polimento avançado, conforme medições e feedback.
- [ ] Corrigir problemas de acessibilidade avançada e compatibilidade encontrados após a V1.

Esses itens não devem atrasar a V1, salvo quando um deles for necessário para uma distribuição segura e utilizável.

## V2 — Codex e OpenCode

### 1. Descoberta da integração OpenCode

- [ ] Confirmar uma fonte oficial ou estável para quota, limite e reset do OpenCode.
- [ ] Documentar autenticação, disponibilidade por plano, formato, atualização e limitações da fonte.
- [ ] Rejeitar leitura direta de segredos ou endpoints não documentados quando houver alternativa oficial.
- [ ] Construir um spike isolado que prove os estados autenticado, deslogado, ferramenta ausente e falha de rede.
- [ ] Não iniciar a integração de produção se a fonte não puder sustentar valores confiáveis.

### 2. Adicionar OpenCode ao produto

- [ ] Implementar um cliente OpenCode independente e read-only.
- [ ] Mostrar Codex e OpenCode no mesmo painel sem misturar métricas incompatíveis.
- [ ] Permitir que um provedor continue útil quando o outro falhar.
- [ ] Identificar claramente a fonte, a última atualização e os campos indisponíveis de cada provedor.
- [ ] Reutilizar componentes visuais somente onde o conteúdo e os estados forem realmente equivalentes.

### 3. Extrair a abstração comum

- [ ] Comparar os dois clientes reais antes de definir uma interface compartilhada.
- [ ] Extrair apenas o contrato comprovadamente comum: identidade do provedor, disponibilidade, janelas, última atualização e erro apresentável.
- [ ] Manter autenticação, transporte e campos específicos dentro de cada cliente.
- [ ] Evitar fábrica, descoberta dinâmica de plugins ou configuração genérica enquanto existirem somente dois provedores fixos.

### 4. Validar e lançar a V2

- [ ] Testar combinações em que ambos, apenas um ou nenhum provedor esteja disponível.
- [ ] Testar múltiplas janelas, respostas parciais, logout, timeout e mudanças de formato de cada fonte.
- [ ] Medir o impacto do segundo processo ou integração em memória, CPU e tempo de abertura.
- [ ] Atualizar documentação, privacidade, instalação e limitações conhecidas.
- [ ] Lançar somente quando a adição do OpenCode não reduzir a confiabilidade da experiência Codex existente.

## Fora do escopo atual

- Claude, Cursor ou terceiro provedor.
- Histórico local e gráficos.
- Custo de API e faturamento.
- Conta própria do My AI Usage.
- Sincronização entre dispositivos.
- Backend, telemetria ou coleta remota.
- Sistema de plugins.
- Troca ou gerenciamento de contas dos provedores.
