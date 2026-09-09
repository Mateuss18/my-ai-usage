# Produto

My AI Usage e um pequeno aplicativo Tauri para Windows que responde rapidamente quanto da quota do Codex ja foi usada e quando ela renova.

## Objetivo

Entregar um utilitario discreto e confiavel que:

- detecte se o Codex local esta disponivel;
- use a sessao ja gerenciada pelo Codex;
- apresente as janelas retornadas pelo servico;
- mostre percentual usado e proxima renovacao no fuso local;
- continue util quando parte dos dados estiver indisponivel.

## Escopo atual

- Windows 11 inicialmente.
- Aplicativo Tauri com frontend Vue e runtime Rust.
- Janela compacta acessivel pela bandeja.
- Login e logout do Codex pelo fluxo local suportado.
- Uma conta ativa por leitura e snapshots stale de contas ja vistas.
- Atualizacao manual e automatica serializada.
- Nenhum servidor, telemetria ou armazenamento de credenciais do projeto.

## Fora do escopo

- Outros provedores.
- Troca ou gerenciamento de contas.
- Historico de tokens, graficos e custo de API.
- Sincronizacao entre dispositivos.
- Backend e coleta remota.

## Principios

1. Windows-first: integracao coerente com o sistema.
2. Local-first: dados e credenciais permanecem locais.
3. Read-only: quotas e contas nao sao alteradas pelo aplicativo.
4. Honestidade: campos ausentes permanecem desconhecidos.
5. Escopo pequeno: adicionar somente o que o fluxo real exigir.
