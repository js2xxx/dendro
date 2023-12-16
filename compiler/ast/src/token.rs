use dendro_span::{span::Span, symbol::Symbol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Expression-operator symbols.
    /// `=`
    Eq,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `==`
    EqEq,
    /// `!=`
    Ne,
    /// `>=`
    Ge,
    /// `>`
    Gt,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `!`
    Not,
    /// `~`
    Tilde,

    BinOp(BinOpToken),
    BinOpEq(BinOpToken),

    // Structural symbols
    /// `@`
    At,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `...`
    DotDotDot,
    /// `..=`
    DotDotEq,
    /// `,`
    Comma,
    /// `;`
    Semi,
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `->`
    RArrow,
    /// `<-`
    LArrow,
    /// `=>`
    FatArrow,
    /// `#`
    Pound,
    /// `$`
    Dollar,
    /// `?`
    Question,
    /// Used by proc macros for representing lifetimes, not generated by lexer
    /// right now.
    SingleQuote,
    /// An opening delimiter (e.g., `{`).
    OpenDelim(Delimiter),
    /// A closing delimiter (e.g., `}`).
    CloseDelim(Delimiter),

    // Literals
    Literal(Lit),

    /// Identifier token.
    /// Do not forget about `NtIdent` when you want to match on identifiers.
    /// It's recommended to use
    /// `Token::(ident,uninterpolate,uninterpolated_span)` to treat regular
    /// and interpolated identifiers in the same way.
    Ident(Symbol, /* is_raw */ bool),
    /// Lifetime identifier token.
    /// Do not forget about `NtLifetime` when you want to match on lifetime
    /// identifiers. It's recommended to use
    /// `Token::(lifetime,uninterpolate,uninterpolated_span)` to
    /// treat regular and interpolated lifetime identifiers in the same way.
    Lifetime(Symbol),

    /// A doc comment token.
    /// `Symbol` is the doc comment's data excluding its "quotes" (`///`, `/**`,
    /// etc) similarly to symbols in string literal tokens.
    DocComment(CommentKind, AttrStyle, Symbol),
}
pub use TokenKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentKind {
    Line,
    Block,
}
pub use CommentKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttrStyle {
    Outer,
    Inner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpToken {
    Plus,
    Minus,
    Star,
    Slash,
    BackSlash,
    Percent,
    Caret,
    And,
    Or,
    Shl,
    Shr,
}
pub use BinOpToken::*;

/// Describes how a sequence of token trees is delimited.
/// Cannot use `proc_macro::Delimiter` directly because this
/// structure should implement some additional traits.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Delimiter {
    /// `( ... )`
    Parenthesis,
    /// `{ ... }`
    Brace,
    /// `[ ... ]`
    Bracket,
}
pub use Delimiter::*;

// Note that the suffix is *not* considered when deciding the `LitKind` in this
// type. This means that float literals like `1f32` are classified by this type
// as `Int`. Only upon conversion to `ast::LitKind` will such a literal be
// given the `Float` kind.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LitKind {
    Bool, // AST only, must never appear in a `Token`
    Byte,
    Char,
    Integer, // e.g. `1`, `1u8`, `1f32`
    Float,   // e.g. `1.`, `1.0`, `1e3f32`
    Str,
    ByteStr,
    CStr,
}
pub use LitKind::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: Symbol,
    pub suffix: Option<Symbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }

    pub fn glue(&self, joint: &Self) -> Option<Self> {
        let kind = match self.kind {
            Eq => match joint.kind {
                Eq => EqEq,
                Gt => FatArrow,
                _ => return None,
            },
            Lt => match joint.kind {
                Eq => Le,
                Lt => BinOp(Shl),
                Le => BinOpEq(Shl),
                BinOp(Minus) => LArrow,
                _ => return None,
            },
            Gt => match joint.kind {
                Eq => Ge,
                Gt => BinOp(Shr),
                Ge => BinOpEq(Shr),
                _ => return None,
            },
            Not => match joint.kind {
                Eq => Ne,
                _ => return None,
            },
            BinOp(op) => match joint.kind {
                Eq => BinOpEq(op),
                BinOp(And) if op == And => AndAnd,
                BinOp(Or) if op == Or => OrOr,
                Gt if op == Minus => RArrow,
                _ => return None,
            },
            Dot => match joint.kind {
                Dot => DotDot,
                DotDot => DotDotDot,
                _ => return None,
            },
            DotDot => match joint.kind {
                Dot => DotDotDot,
                Eq => DotDotEq,
                _ => return None,
            },
            Colon => match joint.kind {
                Colon => ColonColon,
                _ => return None,
            },
            SingleQuote => match joint.kind {
                Ident(name, false) => Lifetime(Symbol::new_owned(format!("'{name}"))),
                _ => return None,
            },

            Le | EqEq | Ne | Ge | AndAnd | OrOr | Tilde | BinOpEq(..) | At | DotDotDot
            | DotDotEq | Comma | Semi | ColonColon | RArrow | LArrow | FatArrow | Pound
            | Dollar | Question | OpenDelim(..) | CloseDelim(..) | Literal(..) | Ident(..)
            | Lifetime(..) | DocComment(..) => return None,
        };

        Some(Token::new(kind, self.span.to(&joint.span)))
    }
}
