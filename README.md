# md2docx

A simple command-line tool to convert Markdown files to DOCX (Microsoft Word) format.

## Features

- Convert Markdown files to DOCX format
- Support for headings, paragraphs, and basic formatting (bold, etc.)
- Parse YAML front matter for title and author metadata
- Simple command-line interface

## Installation

```bash
cargo install --path .
```

## Usage

Convert a Markdown file to DOCX:

```bash
md2docx input.md
```

Specify a custom output file:

```bash
md2docx input.md -o custom_output.docx
```

Use the built-in sample content:

```bash
md2docx --sample
```

View help information:

```bash
md2docx --help
```

## Supported Markdown Features

- Headings (H1-H6)
- Paragraphs
- Bold text
- Line breaks
- Images with scale control
- YAML front matter (title and author)

## Images

Images can be customized by inserting JSON into the `alt` field.  Currently, `md2docx` supports:

1. `scale: float` --- Image size scaling
2. `ref: string` --- Image label

An image label can be referenced in text using the `ref` keyword inside curly brackets:

Images can also be captioned.

```
![{"scale": 0.5, "ref": "a-scaled-image"}][url "Figure Caption"]

{ref: a-scaled-image} is scaled to 50% of its true size.
```

## License

GPLv3
