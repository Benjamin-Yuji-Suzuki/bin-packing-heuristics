#!/usr/bin/env python3
"""Generate styled PDF from guia-de-estudo.md — Rust theme (dark)"""
from weasyprint import HTML
import markdown

with open('/home/ben/Área de trabalho/Onde deve rodar a IA/Analise-de-Algoritmos/Artigo/2026-09-BinPacking/guia-de-estudo.md', 'r') as f:
    md_content = f.read()

html_content = markdown.markdown(md_content, extensions=['tables', 'fenced_code'])

template = f"""
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
@page {{
    size: A4;
    margin: 2.5cm 2cm 2cm 2cm;
    @bottom-center {{
        content: counter(page);
        font-size: 10pt;
        color: #666;
    }}
}}

body {{
    font-family: "DejaVu Sans", "Segoe UI", Arial, sans-serif;
    font-size: 11pt;
    line-height: 1.6;
    color: #2a2a2a;
}}

h1 {{
    color: #c44500;
    font-size: 22pt;
    border-bottom: 2px solid #c44500;
    padding-bottom: 10pt;
    margin-top: 0;
    margin-bottom: 15pt;
}}

h2 {{
    color: #a33a00;
    font-size: 16pt;
    border-bottom: 1px solid #c44500;
    padding-bottom: 5pt;
    margin-top: 25pt;
    margin-bottom: 10pt;
}}

h3 {{
    color: #8a3000;
    font-size: 13pt;
    margin-top: 18pt;
    margin-bottom: 8pt;
}}

p {{
    margin: 0 0 10pt 0;
    text-align: justify;
}}

ul, ol {{
    margin: 0 0 10pt 0;
    padding-left: 20pt;
}}

li {{
    margin-bottom: 4pt;
}}

blockquote {{
    border-left: 4px solid #c44500;
    background: #f5f0eb;
    padding: 10pt 15pt;
    margin: 10pt 0;
    font-size: 10pt;
}}

code {{
    background: #d0d0d0;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: "JetBrains Mono", "Fira Code", "DejaVu Sans Mono", monospace;
    font-size: 9.5pt;
    color: #1a1a1a;
    font-weight: 600;
}}

pre {{
    background: #1a1a1a;
    color: #d4d4d4;
    padding: 14pt;
    border-radius: 6px;
    font-size: 9.5pt;
    line-height: 1.5;
    overflow-wrap: break-word;
    white-space: pre-wrap;
    margin: 10pt 0;
    font-family: "JetBrains Mono", "Fira Code", "DejaVu Sans Mono", monospace;
    border-left: 4px solid #c44500;
}}

strong {{
    color: #8a3000;
}}

a {{
    color: #c44500;
    text-decoration: none;
    font-weight: 500;
}}

table {{
    border-collapse: collapse;
    width: 100%;
    font-size: 10pt;
    margin: 10pt 0;
    page-break-inside: avoid;
}}

th {{
    background: #c44500;
    color: white;
    padding: 7px 9px;
    text-align: left;
    font-weight: bold;
}}

td {{
    padding: 6px 9px;
    border: 1px solid #e0e0e0;
    vertical-align: top;
}}

tr:nth-child(even) {{
    background: #fef9f5;
}}

hr {{
    border: none;
    border-top: 2px solid #c44500;
    margin: 20pt 0;
}}
</style>
</head>
<body>
{html_content}
</body>
</html>
"""

HTML(string=template).write_pdf('/home/ben/Área de trabalho/Onde deve rodar a IA/Analise-de-Algoritmos/Artigo/2026-09-BinPacking/guia-de-estudo.pdf')
print("PDF gerado com tema Rust (tons escuros)!")
