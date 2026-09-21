# Assinatura dos releases Windows

## Problema

O build local produz um executável e um MSI válidos, mas ambos ficam sem assinatura (`NotSigned`). Isso permite que antivírus e mecanismos de reputação tratem o artefato como desconhecido. Usuários finais não devem precisar desativar proteção para instalar o aplicativo.

## Desenho aprovado para implementação

- Criar um certificado autoassinado de código no armazenamento do usuário atual para validar a instalação nesta máquina.
- Criar um script PowerShell único para preparar o certificado, configurar o Tauri para usar o `signtool.exe` nativo durante o bundle e verificar o executável e o MSI resultantes.
- Adicionar um comando npm de release que gere o MSI, assine os artefatos e falhe se a assinatura não puder ser validada.
- Manter a chave privada fora do Git. O modo local usa o certificado do Windows; o modo público poderá usar um certificado `.pfx` fornecido por segredo de CI, sem mudar o fluxo.
- O release público só será considerado pronto com certificado Authenticode de uma autoridade reconhecida. O certificado autoassinado é apenas para teste local e não será tratado como solução pública.

## Fora de escopo

- Comprar ou inventar um certificado público.
- Desativar ou enfraquecer antivírus.
- Alterar a arquitetura do aplicativo ou migrar de MSI para outro formato.

## Verificação

- O build deve produzir o MSI x64.
- O `.exe` e o `.msi` devem conter assinatura.
- `Get-AuthenticodeSignature` deve retornar `Valid` na máquina que confia no certificado local.
- O artefato final deve permanecer acessível e o estado do Git deve mostrar somente as mudanças deste fluxo.
