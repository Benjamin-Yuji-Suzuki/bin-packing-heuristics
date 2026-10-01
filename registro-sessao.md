# Registro de Sessão: Correções e Desafios

**Data:** 24/09/2026
**Agente:** Hermes Agent (meituan/longcat-2.0:free) + revisões via Claude Sonnet 4.6 e Gemini 3.1 Pro

---

## 1. Correções aplicadas ao artigo (`artigo-parcial.tex`)

- Removidos jargões da disciplina ("Eixos E3/E7", "Matriz de referências") — sugestão do Gemini 3.1 Pro
- Adicionada nota A(I)/L2(I) ≥ A(I)/OPT(I) na seção de Métricas
- Tabela unificada (qualidade + tempo por linguagem) com `\resizebox` para caber na margem
- Incluídos 3 gráficos: razão por distribuição, comparação linguagens, tempo × n (log-log)
- Seção "Considerações Parciais (Fechamento do 1º Bimestre)" adicionada
- Declaração de IA reformatação em tabela com 3 ferramentas (Hermes, Claude Sonnet 4.6, Gemini 3.1 Pro)

## 2. Guia de estudo (`guia-de-estudo.md`)

- Reescrito com 15 seções: resumo decodificado, seção por seção, exemplos numéricos, 16 referências, perguntas de defesa, glossário, fala de abertura, declaração de IA
- PDF gerado com tema Rust (via weasyprint)

## 3. Configurações locais

- VSCode: extensão Markdown PDF configurada com `github.css` e margens A4
- Tema Rust (`rust-pdf-style.css`) criado em `~/.config/Code/User/`
- Script `gerar-pdf-guia.py` salvo no projeto para gerar PDF com tema consistente

## 4. Licença

- MIT License adicionada ao repositório (necessária para código aberto)

## 5. Estrutura final do projeto

```
bin-packing-heuristics/
├── artigo/
│   ├── artigo-parcial.tex   (11 páginas, SBC)
│   ├── artigo-parcial.pdf
│   ├── *.png                (3 gráficos)
│   └── referencias.bib      (16 referências)
├── bin-packing-heuristics/  (código Rust)
├── guia-de-estudo.md        (633 linhas)
├── guia-de-estudo.pdf       (tema Rust)
├── gerar-pdf-guia.py        (script Python)
├── LICENSE                  (MIT)
└── README.md                (declaração de IA)
```
