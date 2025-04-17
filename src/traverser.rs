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
    type Output;

    /// Process a node and return the output
    fn process_node(&mut self, node: &Node, output: Self::Output) -> Self::Output {
        match node {
            Node::Root(root) => self.visit_root(root, output),
            Node::Paragraph(para) => self.visit_paragraph(para, output),
            Node::Heading(heading) => self.visit_heading(heading, output),
            Node::Text(text) => self.visit_text(text, output),
            Node::Strong(strong) => self.visit_strong(strong, output),
            Node::Emphasis(emphasis) => self.visit_emphasis(emphasis, output),
            Node::List(list) => self.visit_list(list, output),
            Node::ListItem(list_item) => self.visit_list_item(list_item, output),
            Node::Image(image) => self.visit_image(image, output),
            Node::Blockquote(blockquote) => self.visit_blockquote(blockquote, output),
            Node::FootnoteDefinition(def) => self.visit_footnote_definition(def, output),
            Node::MdxJsxFlowElement(elem) => self.visit_mdx_jsx_flow_element(elem, output),
            Node::MdxjsEsm(esm) => self.visit_mdxjs_esm(esm, output),
            Node::Toml(toml) => self.visit_toml(toml, output),
            Node::Yaml(yaml) => self.visit_yaml(yaml, output),
            Node::Break(break_node) => self.visit_break(break_node, output),
            Node::InlineCode(code) => self.visit_inline_code(code, output),
            Node::InlineMath(math) => self.visit_inline_math(math, output),
            Node::Delete(del) => self.visit_delete(del, output),
            Node::MdxTextExpression(expr) => self.visit_mdx_text_expression(expr, output),
            Node::FootnoteReference(ref_node) => self.visit_footnote_reference(ref_node, output),
            Node::Html(html) => self.visit_html(html, output),
            Node::ImageReference(img_ref) => self.visit_image_reference(img_ref, output),
            Node::MdxJsxTextElement(elem) => self.visit_mdx_jsx_text_element(elem, output),
            Node::Link(link) => self.visit_link(link, output),
            Node::LinkReference(link_ref) => self.visit_link_reference(link_ref, output),
            Node::Code(code) => self.visit_code(code, output),
            Node::Math(math) => self.visit_math(math, output),
            Node::MdxFlowExpression(expr) => self.visit_mdx_flow_expression(expr, output),
            Node::Table(table) => self.visit_table(table, output),
            Node::ThematicBreak(break_node) => self.visit_thematic_break(break_node, output),
            Node::TableRow(row) => self.visit_table_row(row, output),
            Node::TableCell(cell) => self.visit_table_cell(cell, output),
            Node::Definition(def) => self.visit_definition(def, output),
        }
    }
    /// Process a child node and update the result (override this if needed)
    fn process_child(&mut self, node: &Node, mut result: Self::Output) -> Self::Output {
        // Default implementation just processes the node and passes along the result
        self.process_node(node, result)
    }

    // Default implementations for container nodes that recurse over their children
    fn visit_root(&mut self, root: &Root, mut output: Self::Output) -> Self::Output {
        for child in &root.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_paragraph(&mut self, para: &Paragraph, mut output: Self::Output) -> Self::Output {
        for child in &para.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_strong(&mut self, strong: &Strong, mut output: Self::Output) -> Self::Output {
        for child in &strong.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_emphasis(&mut self, emphasis: &Emphasis, mut output: Self::Output) -> Self::Output {
        for child in &emphasis.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_list(&mut self, list: &List, mut output: Self::Output) -> Self::Output {
        for child in &list.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_list_item(&mut self, list_item: &ListItem, mut output: Self::Output) -> Self::Output {
        for child in &list_item.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_blockquote(
        &mut self,
        blockquote: &Blockquote,
        mut output: Self::Output,
    ) -> Self::Output {
        for child in &blockquote.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_link(&mut self, link: &Link, mut output: Self::Output) -> Self::Output {
        for child in &link.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_table(&mut self, table: &Table, mut output: Self::Output) -> Self::Output {
        for child in &table.children {
            output = self.process_child(child, output);
        }
        output
    }

    fn visit_table_row(&mut self, row: &TableRow, mut output: Self::Output) -> Self::Output {
        for child in &row.children {
            output = self.process_child(child, output);
        }
        output
    }

    // Default implementations for leaf nodes that have no children - return the passed output
    fn visit_text(&mut self, _text: &Text, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_heading(&mut self, _heading: &Heading, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_image(&mut self, _image: &Image, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_footnote_definition(
        &mut self,
        _def: &FootnoteDefinition,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_mdx_jsx_flow_element(
        &mut self,
        _elem: &MdxJsxFlowElement,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_mdxjs_esm(&mut self, _esm: &MdxjsEsm, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_toml(&mut self, _toml: &Toml, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_yaml(&mut self, _yaml: &Yaml, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_break(&mut self, _break_node: &Break, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_inline_code(&mut self, _code: &InlineCode, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_inline_math(&mut self, _math: &InlineMath, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_delete(&mut self, _del: &Delete, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_mdx_text_expression(
        &mut self,
        _expr: &MdxTextExpression,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_footnote_reference(
        &mut self,
        _ref_node: &FootnoteReference,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_html(&mut self, _html: &Html, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_image_reference(
        &mut self,
        _img_ref: &ImageReference,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_mdx_jsx_text_element(
        &mut self,
        _elem: &MdxJsxTextElement,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_link_reference(
        &mut self,
        _link_ref: &LinkReference,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_code(&mut self, _code: &Code, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_math(&mut self, _math: &Math, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_mdx_flow_expression(
        &mut self,
        _expr: &MdxFlowExpression,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_thematic_break(
        &mut self,
        _break_node: &ThematicBreak,
        output: Self::Output,
    ) -> Self::Output {
        output
    }

    fn visit_table_cell(&mut self, _cell: &TableCell, output: Self::Output) -> Self::Output {
        output
    }

    fn visit_definition(&mut self, _def: &Definition, output: Self::Output) -> Self::Output {
        output
    }
}
