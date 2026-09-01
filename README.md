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

## Stack pretendida

- C# e .NET.
- WinUI 3 com Windows App SDK.
- `codex app-server` para acessar dados do Codex via JSON-RPC.

## Estado

Em descoberta e definição de produto. Ainda não há uma aplicação implementada.

Consulte [PRODUCT.md](docs/PRODUCT.md) para o escopo e [TECHNICAL_NOTES.md](docs/TECHNICAL_NOTES.md) para os achados técnicos.

## Princípios

1. Windows-first: integração e aparência coerentes com o sistema.
2. Local-first: dados e credenciais não passam por servidores do projeto.
3. Read-only: a V1 consulta e apresenta dados; não altera contas ou quotas.
4. Honestidade: valores indisponíveis são exibidos como desconhecidos, nunca estimados como fatos.
5. Escopo pequeno: uma integração funcional antes de uma arquitetura genérica.

## Licença

[MIT](LICENSE)
