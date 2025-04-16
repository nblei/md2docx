use markdown::mdast::{
    Blockquote, Break, Code, Definition, Delete, Emphasis, FootnoteDefinition, FootnoteReference,
    Heading, Html, Image, ImageReference, InlineCode, InlineMath, Link, LinkReference, List,
    ListItem, Math, MdxFlowExpression, MdxJsxFlowElement, MdxJsxTextElement, MdxTextExpression,
    MdxjsEsm, Node, Paragraph, Root, Strong, Table, TableCell, TableRow, Text, ThematicBreak, Toml,
    Yaml,
};

/// A trait for traversing Markdown AST nodes
pub trait MarkdownNodeTraverser {
    /// The type that will be produced during traversal
    type Output: Default;

    /// Process a node and return the output
    fn process_node(&mut self, node: &Node) -> Self::Output {
        match node {
            Node::Root(root) => self.visit_root(root),
            Node::Paragraph(para) => self.visit_paragraph(para),
            Node::Heading(heading) => self.visit_heading(heading),
            Node::Text(text) => self.visit_text(text),
            Node::Strong(strong) => self.visit_strong(strong),
            Node::Emphasis(emphasis) => self.visit_emphasis(emphasis),
            Node::List(list) => self.visit_list(list),
            Node::ListItem(list_item) => self.visit_list_item(list_item),
            Node::Image(image) => self.visit_image(image),
            Node::Blockquote(blockquote) => self.visit_blockquote(blockquote),
            Node::FootnoteDefinition(def) => self.visit_footnote_definition(def),
            Node::MdxJsxFlowElement(elem) => self.visit_mdx_jsx_flow_element(elem),
            Node::MdxjsEsm(esm) => self.visit_mdxjs_esm(esm),
            Node::Toml(toml) => self.visit_toml(toml),
            Node::Yaml(yaml) => self.visit_yaml(yaml),
            Node::Break(break_node) => self.visit_break(break_node),
            Node::InlineCode(code) => self.visit_inline_code(code),
            Node::InlineMath(math) => self.visit_inline_math(math),
            Node::Delete(del) => self.visit_delete(del),
            Node::MdxTextExpression(expr) => self.visit_mdx_text_expression(expr),
            Node::FootnoteReference(ref_node) => self.visit_footnote_reference(ref_node),
            Node::Html(html) => self.visit_html(html),
            Node::ImageReference(img_ref) => self.visit_image_reference(img_ref),
            Node::MdxJsxTextElement(elem) => self.visit_mdx_jsx_text_element(elem),
            Node::Link(link) => self.visit_link(link),
            Node::LinkReference(link_ref) => self.visit_link_reference(link_ref),
            Node::Code(code) => self.visit_code(code),
            Node::Math(math) => self.visit_math(math),
            Node::MdxFlowExpression(expr) => self.visit_mdx_flow_expression(expr),
            Node::Table(table) => self.visit_table(table),
            Node::ThematicBreak(break_node) => self.visit_thematic_break(break_node),
            Node::TableRow(row) => self.visit_table_row(row),
            Node::TableCell(cell) => self.visit_table_cell(cell),
            Node::Definition(def) => self.visit_definition(def),
        }
    }

    /// Process a child node and update the result (override this if needed)
    fn process_child(&mut self, node: &Node, _result: &mut Self::Output) {
        // Default implementation just processes the node and ignores the result
        self.process_node(node);
    }

    // Default implementations for container nodes that recurse over their children
    fn visit_root(&mut self, root: &Root) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &root.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_paragraph(&mut self, para: &Paragraph) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &para.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_strong(&mut self, strong: &Strong) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &strong.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_emphasis(&mut self, emphasis: &Emphasis) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &emphasis.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_list(&mut self, list: &List) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &list.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_list_item(&mut self, list_item: &ListItem) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &list_item.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_blockquote(&mut self, blockquote: &Blockquote) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &blockquote.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_link(&mut self, link: &Link) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &link.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_table(&mut self, table: &Table) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &table.children {
            self.process_child(child, &mut result);
        }
        result
    }

    fn visit_table_row(&mut self, row: &TableRow) -> Self::Output {
        let mut result = Self::Output::default();
        for child in &row.children {
            self.process_child(child, &mut result);
        }
        result
    }

    // Default implementations for leaf nodes that have no children - return default output
    fn visit_text(&mut self, _text: &Text) -> Self::Output {
        Self::Output::default()
    }

    fn visit_heading(&mut self, _heading: &Heading) -> Self::Output {
        Self::Output::default()
    }

    fn visit_image(&mut self, _image: &Image) -> Self::Output {
        Self::Output::default()
    }

    fn visit_footnote_definition(&mut self, _def: &FootnoteDefinition) -> Self::Output {
        Self::Output::default()
    }

    fn visit_mdx_jsx_flow_element(&mut self, _elem: &MdxJsxFlowElement) -> Self::Output {
        Self::Output::default()
    }

    fn visit_mdxjs_esm(&mut self, _esm: &MdxjsEsm) -> Self::Output {
        Self::Output::default()
    }

    fn visit_toml(&mut self, _toml: &Toml) -> Self::Output {
        Self::Output::default()
    }

    fn visit_yaml(&mut self, _yaml: &Yaml) -> Self::Output {
        Self::Output::default()
    }

    fn visit_break(&mut self, _break_node: &Break) -> Self::Output {
        Self::Output::default()
    }

    fn visit_inline_code(&mut self, _code: &InlineCode) -> Self::Output {
        Self::Output::default()
    }

    fn visit_inline_math(&mut self, _math: &InlineMath) -> Self::Output {
        Self::Output::default()
    }

    fn visit_delete(&mut self, _del: &Delete) -> Self::Output {
        Self::Output::default()
    }

    fn visit_mdx_text_expression(&mut self, _expr: &MdxTextExpression) -> Self::Output {
        Self::Output::default()
    }

    fn visit_footnote_reference(&mut self, _ref_node: &FootnoteReference) -> Self::Output {
        Self::Output::default()
    }

    fn visit_html(&mut self, _html: &Html) -> Self::Output {
        Self::Output::default()
    }

    fn visit_image_reference(&mut self, _img_ref: &ImageReference) -> Self::Output {
        Self::Output::default()
    }

    fn visit_mdx_jsx_text_element(&mut self, _elem: &MdxJsxTextElement) -> Self::Output {
        Self::Output::default()
    }

    fn visit_link_reference(&mut self, _link_ref: &LinkReference) -> Self::Output {
        Self::Output::default()
    }

    fn visit_code(&mut self, _code: &Code) -> Self::Output {
        Self::Output::default()
    }

    fn visit_math(&mut self, _math: &Math) -> Self::Output {
        Self::Output::default()
    }

    fn visit_mdx_flow_expression(&mut self, _expr: &MdxFlowExpression) -> Self::Output {
        Self::Output::default()
    }

    fn visit_thematic_break(&mut self, _break_node: &ThematicBreak) -> Self::Output {
        Self::Output::default()
    }

    fn visit_table_cell(&mut self, _cell: &TableCell) -> Self::Output {
        Self::Output::default()
    }

    fn visit_definition(&mut self, _def: &Definition) -> Self::Output {
        Self::Output::default()
    }
}

