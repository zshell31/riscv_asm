use nom_span::Spanned;

pub type Span<'i> = Spanned<&'i str>;

#[derive(Debug, Clone, Copy)]
pub struct Offset {
    pub offset: usize,
    pub len: usize,
}

impl<'i> From<Span<'i>> for Offset {
    fn from(span: Span<'i>) -> Self {
        Self {
            offset: span.byte_offset(),
            len: span.len(),
        }
    }
}
