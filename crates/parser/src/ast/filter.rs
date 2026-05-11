use std::fmt::Debug;

use crate::{
    CompOpDef, ParseError, ValueDef, lexer::Lexeme, parser::{Ident, Parsable, Parser}
};

#[derive(Debug, Clone)]
pub struct  FilterHolderDef {
    pub filter: FilterDef,
    pub is_only_and: bool
}
impl Parsable for FilterHolderDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        // 1. Уровень OR (Expression)
        // Грамматика: Term ( "OR" Term )*
        fn parse_filter(parser: &mut Parser, is_only_and: &mut bool) -> Result<FilterDef, ParseError> {
            let mut left = parse_term(parser, is_only_and)?;
            while let Some(Lexeme::Or) = parser.peek() {
                *is_only_and = false;
                parser.next(); // Поглощаем OR
                let right = parse_term(parser, is_only_and)?;
                
                // Собираем в дерево. Для оптимизации можно пушить в Vec, 
                // если текущий left уже Filter::Or
                left = match left {
                    FilterDef::Or(mut v) => { v.push(right); FilterDef::Or(v) },
                    _ => FilterDef::Or(vec![left, right]),
                };
            }
            Ok(left)
        }

        // 2. Уровень AND (Term)
        // Грамматика: Unary ( "AND" Unary )*
        fn parse_term(parser: &mut Parser, is_only_and: &mut bool) -> Result<FilterDef, ParseError> {
            let mut left = parse_unary(parser, is_only_and)?;

            while let Some(Lexeme::And) = parser.peek() {
                parser.next(); // Поглощаем AND
                let right = parse_unary(parser, is_only_and)?;

                left = match left {
                    FilterDef::And(mut v) => { v.push(right); FilterDef::And(v) },
                    _ => FilterDef::And(vec![left, right]),
                };
            }
            Ok(left)
        }

        // 3. Уровень NOT (Unary)
        fn parse_unary(parser: &mut Parser, is_only_and: &mut bool) -> Result<FilterDef, ParseError> {
            if let Some(Lexeme::Not) = parser.peek() {
                *is_only_and = false;
                parser.next();
                let operand = parse_unary(parser, is_only_and)?; // Рекурсия для !!condition
                Ok(FilterDef::Not(Box::new(operand)))
            } else {
                parse_primary(parser, is_only_and)
            }
        }

        // 4. Уровень Скобок и Предикатов (Primary)
        fn parse_primary(parser: &mut Parser, is_only_and: &mut bool) -> Result<FilterDef, ParseError> {
            match parser.peek() {
                // Обработка скобок
                Some(Lexeme::LParen) => {
                    parser.next();
                    let expr = parse_filter(parser, is_only_and)?; // Снова в начало приоритетов
                    parser.expect(Lexeme::RParen)?;
                    Ok(expr)
                }
                // Обработка конкретного условия (age > 10)
                _ => {
                    let predicate = PredicateDef::parse(parser)?;
                    Ok(FilterDef::Leaf(predicate))
                }
            }
        }

        let mut is_only_and = true;
        parser.expect(Lexeme::Where)?;

        let filter = parse_filter(parser, &mut is_only_and)?;

        Ok(Self { filter, is_only_and })
    }
}

#[derive(Debug, Clone)]
pub enum FilterDef {
    Leaf(PredicateDef),
    And(Vec<FilterDef>),
    Or(Vec<FilterDef>),
    Not(Box<FilterDef>),
}

// #[cfg(test)]
// mod test {
//     use crate::{ast::filter::FilterDef, parse_test};

//     #[test]
//     fn test() {
//         let input = r#"WHERE not (not(b or a > 1) and b < 1)"#;
//         let result = parse_test::<FilterDef>(input).unwrap();
//         println!("{:#?}", result)
//     }
// }


#[derive(Clone)]
pub struct PredicateDef {
    pub alias: String,
    pub attribute_name: String,
    pub comparison: Option<ComparisonDef>,
}
impl Parsable for PredicateDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let alias = Ident::parse(parser)?.to_string();
        parser.expect(Lexeme::Dot)?;
        let attr_name = Ident::parse(parser)?.to_string();
        
        let comparison = ComparisonDef::parse_probably(parser);
        Ok(Self{
            alias: alias,
            attribute_name: attr_name,
            comparison: comparison
        })
    }
}

impl Debug for PredicateDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.comparison {
            Some(c) => {
                let op = match c.op {
                    CompOpDef::Equal => "=",
                    CompOpDef::NotEqual => "!=",
                    CompOpDef::Less => "<",
                    CompOpDef::LessOrEqual => "<=",
                    CompOpDef::Greater => ">",
                    CompOpDef::GreaterOrEqual => ">=",
                };

                let value = match &c.value {
                    ValueDef::Int32(i) => i.to_string(),
                    ValueDef::Text(t) => t.to_string(),
                    ValueDef::Bool(b) => b.to_string(),
                    ValueDef::Null => "null".to_string(),
                };

                write!(f, "{} {} {}", self.attribute_name, op, value)
            },
            None => write!(f, "{}", self.attribute_name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComparisonDef {
    pub value: ValueDef,
    pub op: CompOpDef,
}
impl Parsable for ComparisonDef {
    fn parse(parser: &mut Parser) -> Result<Self, ParseError> {
        let op = CompOpDef::parse(parser)?;
        let value = ValueDef::parse(parser)?;
        Ok(Self {
            value: value,
            op: op
        })
    }
}
