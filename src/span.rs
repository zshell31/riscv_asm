use nom::{error::VerboseError, IResult as NomResult};
use nom_span::Spanned;

pub type Span<'i> = Spanned<&'i str>;

pub type IResult<'i, O> = NomResult<Span<'i>, O, VerboseError<Span<'i>>>;
