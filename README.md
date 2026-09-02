# My AI Usage

Aplicativo nativo para Windows que mostra, de forma rápida, quanto da sua quota de agentes de IA já foi consumida e quando ela será renovada.

O projeto começa pequeno: a V1 acompanha somente o Codex. Outros provedores, como Claude e OpenCode, poderão ser avaliados depois que a experiência principal estiver funcionando bem.

## Proposta da V1

- Aplicativo local-first para Windows.
- Ícone na bandeja do sistema.
- Painel compacto aberto pelo ícone.
- Quotas do Codex com percentual usado e horário de renovação.
- Atualização manual e automática.
- Estados claros para carregamento, erro, Codex ausente e usuário desconectado.
- Sem servidor próprio, conta adicional ou telemetria.

## Contrato fechado da V1

- Distribuição em MSIX.
- Auto-start padrão, desligável pelo usuário; quando ativado pelo Windows, o app inicia oculto.
- O tray abre ou restaura o painel compacto.
- Acessibilidade básica — teclado, nomes acessíveis, contraste e não depender só de cor — faz parte da V1.
- Polling serializado de 60 segundos é a decisão V1 para notificações e não permite leituras simultâneas.
- O gate de desempenho é painel utilizável em até 5 s, até 100 MB de working set oculto após 5 min, até 200 MB aberto após 5 min e CPU média abaixo de 2% oculto fora do refresh.
- Instalador mais amigável, auto-update e polimento avançado ficam para a V1.1.

## Stack pretendida

- C# e .NET.
- WinUI 3 com Windows App SDK.
- `codex app-server` para acessar dados do Codex via JSON-RPC.

## Estado

Contrato V1 fechado; ainda não há uma aplicação implementada. O próximo passo é o scaffold WinUI 3.

Consulte [PRODUCT.md](docs/PRODUCT.md) para o escopo e [TECHNICAL_NOTES.md](docs/TECHNICAL_NOTES.md) para os achados técnicos.

## Princípios

1. Windows-first: integração e aparência coerentes com o sistema.
2. Local-first: dados e credenciais não passam por servidores do projeto.
3. Read-only: a V1 consulta e apresenta dados; não altera contas ou quotas.
4. Honestidade: valores indisponíveis são exibidos como desconhecidos, nunca estimados como fatos.
5. Escopo pequeno: uma integração funcional antes de uma arquitetura genérica.

## Licença

[MIT](LICENSE)
