use std::str;
use std::str::FromStr;

use *;

use nom::{
    line_ending,
    not_line_ending,
    is_digit,
    is_oct_digit,
    is_hex_digit,
};

named!(whitespace, take_while!(is_whitespace));


named!(pub id<String>, map!(map_res!(verify!(take_while!(is_digit), |x:&[u8]| x.len() > 0), str::from_utf8), String::from));

named!(pub file_name<FileName>, map_res!(map_res!(take_while!(is_allowed_in_file_name), str::from_utf8), FileName::from_str));

named!(comment<Comment>, complete!(do_parse!(
    hash: tag!("#") >>
    text: map_res!(not_line_ending, str::from_utf8) >>
    (Comment(String::from(text)))
)));

named!(directive<Directive>, map_res!(map_res!(do_parse!(_tag: tag!("@") >> name: take_while!(is_allowed_in_directive_name) >> (name)), str::from_utf8), Directive::from_str));

named!(service_response_marker<ServiceResponseMarker>, do_parse!(_srm: tag!("---") >> (ServiceResponseMarker{})));

named!(literal<Lit>, alt!(
    complete!(do_parse!(_value: tag!("true") >> (Lit::Bool(true)) )) |
    complete!(do_parse!(_value: tag!("false") >> (Lit::Bool(false)) )) |
    complete!(do_parse!(_format: tag!("0x") >> value: map_res!(take_while!(is_hex_digit), str::from_utf8) >> (Lit::Hex{sign: Sign::Implicit, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("+0x") >> value: map_res!(take_while!(is_hex_digit), str::from_utf8) >> (Lit::Hex{sign: Sign::Positive, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("-0x") >> value: map_res!(take_while!(is_hex_digit), str::from_utf8) >> (Lit::Hex{sign: Sign::Negative, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("0b") >> value: map_res!(take_while!(is_bin_digit), str::from_utf8) >> (Lit::Bin{sign: Sign::Implicit, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("+0b") >> value: map_res!(take_while!(is_bin_digit), str::from_utf8) >> (Lit::Bin{sign: Sign::Positive, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("-0b") >> value: map_res!(take_while!(is_bin_digit), str::from_utf8) >> (Lit::Bin{sign: Sign::Negative, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("0o") >> value: map_res!(take_while!(is_oct_digit), str::from_utf8) >> (Lit::Oct{sign: Sign::Implicit, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("+0o") >> value: map_res!(take_while!(is_oct_digit), str::from_utf8) >> (Lit::Oct{sign: Sign::Positive, value: String::from(value)}))) |
    complete!(do_parse!(_format: tag!("-0o") >> value: map_res!(take_while!(is_oct_digit), str::from_utf8) >> (Lit::Oct{sign: Sign::Negative, value: String::from(value)}))) |
    complete!(do_parse!(achar: delimited!(tag!("'"), map_res!(take_until!("'"), str::from_utf8), tag!("'")) >> (Lit::Char(String::from(achar))))) |
    complete!(do_parse!(_sign: tag!("-") >> value: map_res!(verify!(take_while!(is_allowed_in_const), |x:&[u8]| x.iter().all(|x| is_digit(*x))), str::from_utf8) >> (Lit::Dec{sign: Sign::Negative, value: String::from(value)}))) |
    complete!(do_parse!(_sign: tag!("+") >> value: map_res!(verify!(take_while!(is_allowed_in_const), |x:&[u8]| x.iter().all(|x| is_digit(*x))), str::from_utf8) >> (Lit::Dec{sign: Sign::Positive, value: String::from(value)}))) |
    complete!(do_parse!(value: map_res!(verify!(take_while!(is_allowed_in_const), |x:&[u8]| x.iter().all(|x| is_digit(*x))), str::from_utf8) >> (Lit::Dec{sign: Sign::Implicit, value: String::from(value)}))) |
    complete!(do_parse!(_sign: tag!("-") >> value: map_res!(verify!(take_while!(is_allowed_in_const), |x:&[u8]| x.iter().all(|x| is_allowed_in_float(*x))), str::from_utf8) >> (Lit::Float{sign: Sign::Negative, value: String::from(value)}))) |
    complete!(do_parse!(_sign: tag!("+") >> value: map_res!(verify!(take_while!(is_allowed_in_const), |x:&[u8]| x.iter().all(|x| is_allowed_in_float(*x))), str::from_utf8) >> (Lit::Float{sign: Sign::Positive, value: String::from(value)}))) |
    complete!(do_parse!(value: map_res!(take_while!(is_allowed_in_float), str::from_utf8) >> (Lit::Float{sign: Sign::Implicit, value: String::from(value)})))
));

named!(cast_mode<CastMode>, map_res!(map_res!(
    alt!(
        complete!(tag!("saturated")) |
        complete!(tag!("truncated")) 
    ), str::from_utf8), CastMode::from_str)
);

named!(field_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_field_name), |x:&[u8]| x.len() >= 1 && is_lowercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(const_name<Ident>, map!(map_res!(
    verify!(take_while!(is_allowed_in_const_name), |x:&[u8]| x.len() >= 1 && is_uppercase_char(x[0])),
    str::from_utf8), Ident::from)
);

named!(composite_type_name<CompositeType>, map_res!(map_res!(
    take_while!(is_allowed_in_composite_type_name),
    str::from_utf8), CompositeType::from_str)
);

named!(primitive_type<PrimitiveType>, map_res!(map_res!(take_while!(is_allowed_in_primitive_type_name), str::from_utf8), PrimitiveType::from_str));

named!(type_name<Ty>, alt!(
    map!(complete!(primitive_type), Ty::from) |
    map!(complete!(composite_type_name), Ty::from)
));

named!(array_info<ArrayInfo>, alt!(
    complete!(do_parse!(intro: tag!("[<=") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::DynamicLeq(u64::from_str(num).unwrap())))) |
    complete!(do_parse!(intro: tag!("[<") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::DynamicLess(u64::from_str(num).unwrap())))) |
    complete!(do_parse!(intro: tag!("[") >> num: map_res!(take_while!(is_digit), str::from_utf8) >> exit: tag!("]") >> (ArrayInfo::Static(u64::from_str(num).unwrap()))))
));





named!(void_definition<FieldDefinition>, sep!(whitespace, 
                                             do_parse!(
                                                 type_name: verify!(primitive_type, |x:PrimitiveType| x.is_void()) >>
                                                     (FieldDefinition{cast_mode: None, field_type: Ty::Primitive(type_name), array: None, name: None}))
));


named!(field_definition<FieldDefinition>, sep!(whitespace, do_parse!(
    cast_mode: opt!(cast_mode) >>
        field_type: type_name >>
        array: opt!(array_info) >>
        name: field_name >>
        (FieldDefinition{cast_mode: cast_mode, field_type: field_type, array: array, name: Some(name)})
)));


named!(const_definition<ConstDefinition>, sep!(whitespace, do_parse!(
    cast_mode: opt!(cast_mode) >>
        field_type: type_name >>
        name: const_name >>
        _eq: tag!("=") >>
        literal: literal >>
        (ConstDefinition{cast_mode: cast_mode, field_type: field_type, name: name, literal: literal})
)));





named!(attribute_definition<AttributeDefinition>, complete!(sep!(whitespace, alt!(
    map!(const_definition, AttributeDefinition::from) |
    map!(field_definition, AttributeDefinition::from) |
    map!(void_definition, AttributeDefinition::from)
))));



named!(line<Line>, sep!(whitespace, alt!(
    do_parse!(
        definition: attribute_definition >>
            comment: opt!(comment) >>
            _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Definition{definition, comment})
    ) |
    do_parse!(
        directive: directive >>
            comment: opt!(comment) >>
            _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Directive{directive, comment})
    ) |
    do_parse!(
        comment: comment >>
            _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Comment(comment))
    ) |
    do_parse!(
        _eol: verify!(not_line_ending, |x:&[u8]| x.len() == 0) >>
            (Line::Empty)
    )
)));                       

named!(lines<Vec<Line>>, many0!(sep!(whitespace, do_parse!(line: line >> _eol: opt!(complete!(line_ending)) >> (line)))));

named!(message_definition<MessageDefinition>, do_parse!(lines: lines >> (MessageDefinition(lines))));

named!(service_definition<ServiceDefinition>, sep!(whitespace, do_parse!(
    request: message_definition >>
        _srm: service_response_marker >>
        response: message_definition >>
        (ServiceDefinition{request: request, response: response})
)));

named!(pub type_definition<TypeDefinition>, alt!(
    map!(complete!(service_definition), TypeDefinition::from) |
    map!(complete!(message_definition), TypeDefinition::from)
));




fn is_whitespace(chr: u8) -> bool {
    chr == b' ' || chr == b'\t'
}


fn is_lowercase_char(chr: u8) -> bool {
    chr >= b'a' && chr <= b'z'
}

fn is_uppercase_char(chr: u8) -> bool {
    chr >= b'A' && chr <= b'Z'
}

fn is_bin_digit(chr: u8) -> bool {
    chr == b'0' || chr == b'1'
}

fn is_allowed_in_float(chr: u8) -> bool {
    is_digit(chr) || chr == b'.' || chr == b'E' || chr == b'e' || chr == b'-' || chr == b'+'
}

fn is_allowed_in_const(chr: u8) -> bool {
    is_digit(chr) || is_uppercase_char(chr) || is_lowercase_char(chr) || chr == b'.' || chr == b'E' || chr == b'e' || chr == b'-' || chr == b'+'
}

fn is_allowed_in_field_name(chr: u8) -> bool {
    is_lowercase_char(chr) || is_digit(chr) || chr == b'_'
}
    
fn is_allowed_in_primitive_type_name(chr: u8) -> bool {
    is_lowercase_char(chr) || is_digit(chr)
}
    
fn is_allowed_in_const_name(chr: u8) -> bool {
    is_uppercase_char(chr) || is_digit(chr) || chr == b'_'
}
    
fn is_allowed_in_composite_type_name(chr: u8) -> bool {
    is_uppercase_char(chr) || is_lowercase_char(chr) || is_digit(chr) || chr == b'.'
}
    
fn is_allowed_in_directive_name(chr: u8) -> bool {
    is_lowercase_char(chr)
}
    
fn is_allowed_in_file_name(chr: u8) -> bool {
    is_lowercase_char(chr) || is_uppercase_char(chr) || is_digit(chr) || chr == b'.'
}
    
        


#[cfg(test)]
mod tests {
    use super::*;

    use nom::{
        IResult,
    };

    #[test]
    fn parse_file_name() {
        assert_eq!(file_name(&b"NodeStatus.uavcan"[..]), IResult::Done(&b""[..], FileName{id: None, namespace: String::from(""), name: String::from("NodeStatus"), version: None}));
        assert_eq!(file_name(&b"NodeStatus.uavcan"[..]), IResult::Done(&b""[..], FileName{id: None, namespace: String::from(""), name: String::from("NodeStatus"), version: None}));
        assert_eq!(file_name(&b"protocol.NodeStatus.uavcan"[..]), IResult::Done(&b""[..], FileName{id: None, namespace: String::from("protocol"), name: String::from("NodeStatus"), version: None}));
        assert_eq!(file_name(&b"uavcan.protocol.NodeStatus.uavcan"[..]), IResult::Done(&b""[..], FileName{id: None, namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: None}));
        assert_eq!(file_name(&b"protocol.341.NodeStatus.uavcan"[..]), IResult::Done(&b""[..], FileName{id: Some(String::from("341")), namespace: String::from("protocol"), name: String::from("NodeStatus"), version: None}));
        assert_eq!(file_name(&b"uavcan.protocol.341.NodeStatus.uavcan"[..]), IResult::Done(&b""[..], FileName{id: Some(String::from("341")), namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: None}));
        assert_eq!(file_name(&b"protocol.341.NodeStatus.0.1.uavcan"[..]), IResult::Done(&b""[..], FileName{id: Some(String::from("341")), namespace: String::from("protocol"), name: String::from("NodeStatus"), version: Some(Version{minor: 1, major: 0})}));
        assert_eq!(file_name(&b"uavcan.protocol.341.NodeStatus.0.1.uavcan"[..]), IResult::Done(&b""[..], FileName{id: Some(String::from("341")), namespace: String::from("uavcan.protocol"), name: String::from("NodeStatus"), version: Some(Version{minor: 1, major: 0})}));
    }
    
    #[test]
    fn parse_directive() {
        assert_eq!(directive(&b"@union"[..]), IResult::Done(&b""[..], Directive::Union));
    }
    
    #[test]
    fn parse_comment() {
        assert_eq!(comment(&b"#This is a comment\n"[..]), IResult::Done(&b"\n"[..], Comment(String::from("This is a comment"))));
        assert_eq!(comment(&b"#This is a comment"[..]), IResult::Done(&b""[..], Comment(String::from("This is a comment"))));
        assert_eq!(comment(&b"#This is a comment\r\n"[..]), IResult::Done(&b"\r\n"[..], Comment(String::from("This is a comment"))));
        assert_eq!(comment(&b"#This is a longer comment"[..]), IResult::Done(&b""[..], Comment(String::from("This is a longer comment"))));
        assert_eq!(comment(&b"# This is a comment"[..]), IResult::Done(&b""[..], Comment(String::from(" This is a comment"))));
        assert_eq!(comment(&b"#"[..]), IResult::Done(&b""[..], Comment(String::from(""))));   
    }

    #[test]
    fn parse_literal() {
        assert_eq!(literal(&b"12354"[..]), IResult::Done(&b""[..], Lit::Dec{sign: Sign::Implicit, value: String::from("12354")}));
        assert_eq!(literal(&b"-12"[..]), IResult::Done(&b""[..], Lit::Dec{sign: Sign::Negative, value: String::from("12")}));
        assert_eq!(literal(&b"+12"[..]), IResult::Done(&b""[..], Lit::Dec{sign: Sign::Positive, value: String::from("12")}));
        
        assert_eq!(literal(&b"0x123"[..]), IResult::Done(&b""[..], Lit::Hex{sign: Sign::Implicit, value: String::from("123")}));
        assert_eq!(literal(&b"-0x12"[..]), IResult::Done(&b""[..], Lit::Hex{sign: Sign::Negative, value: String::from("12")}));
        assert_eq!(literal(&b"+0x123"[..]), IResult::Done(&b""[..], Lit::Hex{sign: Sign::Positive, value: String::from("123")}));
        
        assert_eq!(literal(&b"0b1101"[..]), IResult::Done(&b""[..], Lit::Bin{sign: Sign::Implicit, value: String::from("1101")}));
        assert_eq!(literal(&b"-0b101101"[..]), IResult::Done(&b""[..], Lit::Bin{sign: Sign::Negative, value: String::from("101101")}));
        assert_eq!(literal(&b"+0b101101"[..]), IResult::Done(&b""[..], Lit::Bin{sign: Sign::Positive, value: String::from("101101")}));
        
        assert_eq!(literal(&b"-0o123"[..]), IResult::Done(&b""[..], Lit::Oct{sign: Sign::Negative, value: String::from("123")}));
        assert_eq!(literal(&b"0o777"[..]), IResult::Done(&b""[..], Lit::Oct{sign: Sign::Implicit, value: String::from("777")}));
        assert_eq!(literal(&b"+0o777"[..]), IResult::Done(&b""[..], Lit::Oct{sign: Sign::Positive, value: String::from("777")}));
        
        assert_eq!(literal(&b"15.75"[..]), IResult::Done(&b""[..], Lit::Float{sign: Sign::Implicit, value: String::from("15.75")}));
        assert_eq!(literal(&b"1.575E1"[..]), IResult::Done(&b""[..], Lit::Float{sign: Sign::Implicit, value: String::from("1.575E1")}));
        assert_eq!(literal(&b"1575e-2"[..]), IResult::Done(&b""[..], Lit::Float{sign: Sign::Implicit, value: String::from("1575e-2")}));
        assert_eq!(literal(&b"-2.5e-3"[..]), IResult::Done(&b""[..], Lit::Float{sign: Sign::Negative, value: String::from("2.5e-3")}));
        assert_eq!(literal(&b"+25e-4"[..]), IResult::Done(&b""[..], Lit::Float{sign: Sign::Positive, value: String::from("25e-4")}));
        
        assert_eq!(literal(&b"true"[..]), IResult::Done(&b""[..], Lit::Bool(true)));
        assert_eq!(literal(&b"false"[..]), IResult::Done(&b""[..], Lit::Bool(false)));
        
        assert_eq!(literal(&b"'a'"[..]), IResult::Done(&b""[..], Lit::Char(String::from("a"))));
        assert_eq!(literal(&[39,92,b'x',b'6',b'1', 39]), IResult::Done(&b""[..], Lit::Char(String::from("\\x61"))));
        assert_eq!(literal(&[39,92,b'n', 39]), IResult::Done(&b""[..], Lit::Char(String::from("\\n"))));
    }

    #[test]
    fn parse_field_name() {
        assert_eq!(field_name(&b"variable23"[..]), IResult::Done(&b""[..], Ident(String::from("variable23"))));
        assert_eq!(field_name(&b"var_iable23"[..]), IResult::Done(&b""[..], Ident(String::from("var_iable23"))));
        assert!(field_name(&b"2variable23"[..]).is_err());
    }

    #[test]
    fn parse_const_name() {
        assert_eq!(const_name(&b"CONST"[..]), IResult::Done(&b""[..], Ident(String::from("CONST"))));
        assert_eq!(const_name(&b"CONST23"[..]), IResult::Done(&b""[..], Ident(String::from("CONST23"))));
        assert_eq!(const_name(&b"CON_ST"[..]), IResult::Done(&b""[..], Ident(String::from("CON_ST"))));
        assert_eq!(const_name(&b"CON_ST1_2345"[..]), IResult::Done(&b""[..], Ident(String::from("CON_ST1_2345"))));
        assert!(const_name(&b"2CON"[..]).is_err());
    }

    #[test]
    fn parse_composite_type_name() {
        assert_eq!(composite_type_name(&b"TypeName"[..]), IResult::Done(&b""[..], CompositeType{namespace: None, name: Ident(String::from("TypeName")) } ));
        assert_eq!(composite_type_name(&b"TypeName1234"[..]), IResult::Done(&b""[..], CompositeType{namespace: None, name: Ident(String::from("TypeName1234")) } ));

        assert_eq!(composite_type_name(&b"uavcan.protocol.TypeName"[..]), IResult::Done(&b""[..], CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName"))}));
        assert_eq!(composite_type_name(&b"uavcan.protocol.TypeName1234"[..]), IResult::Done(&b""[..], CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName1234"))}));
    }

    #[test]
    fn parse_primitive_type() {
        assert_eq!(primitive_type(&b"uint2"[..]), IResult::Done(&b""[..], PrimitiveType::Uint2));
        assert_eq!(primitive_type(&b"uint3"[..]), IResult::Done(&b""[..], PrimitiveType::Uint3));
        assert_eq!(primitive_type(&b"uint16"[..]), IResult::Done(&b""[..], PrimitiveType::Uint16));
        assert_eq!(primitive_type(&b"uint32"[..]), IResult::Done(&b""[..], PrimitiveType::Uint32));
        
        assert!(primitive_type(&b"2variable23"[..]).is_err());
    }

    #[test]
    fn parse_type_name() {
        assert_eq!(type_name(&b"uint2"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint2)));
        assert_eq!(type_name(&b"uint3"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint3)));
        assert_eq!(type_name(&b"uint16"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint16)));
        assert_eq!(type_name(&b"uint32"[..]), IResult::Done(&b""[..], Ty::Primitive(PrimitiveType::Uint32)));

        assert_eq!(type_name(&b"TypeName"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: None, name: Ident(String::from("TypeName"))})));
        assert_eq!(type_name(&b"TypeName1234"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: None, name: Ident(String::from("TypeName1234"))})));

        assert_eq!(type_name(&b"uavcan.protocol.TypeName"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName"))})));
        assert_eq!(type_name(&b"uavcan.protocol.TypeName1234"[..]), IResult::Done(&b""[..], Ty::Composite(CompositeType{namespace: Some(Ident(String::from("uavcan.protocol"))), name: Ident(String::from("TypeName1234"))})));
        
    }

    #[test]
    fn parse_cast_mode() {
        assert_eq!(cast_mode(&b"saturated"[..]), IResult::Done(&b""[..], CastMode::Saturated));
        assert_eq!(cast_mode(&b"truncated"[..]), IResult::Done(&b""[..], CastMode::Truncated));
        
        assert!(cast_mode(&b"2variable23"[..]).is_err());
        assert!(cast_mode(&b""[..]).is_err());
    }

    #[test]
    fn parse_array_info() {
        assert_eq!(array_info(&b"[<=4]"[..]), IResult::Done(&b""[..], ArrayInfo::DynamicLeq(u64::from_str("4").unwrap())));
        assert_eq!(array_info(&b"[<5]"[..]), IResult::Done(&b""[..], ArrayInfo::DynamicLess(u64::from_str("5").unwrap())));
        
        assert_eq!(array_info(&b"[<=128]"[..]), IResult::Done(&b""[..], ArrayInfo::DynamicLeq(u64::from_str("128").unwrap())));
        assert_eq!(array_info(&b"[<129]"[..]), IResult::Done(&b""[..], ArrayInfo::DynamicLess(u64::from_str("129").unwrap())));

        assert_eq!(array_info(&b"[4]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(u64::from_str("4").unwrap())));
        assert_eq!(array_info(&b"[5]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(u64::from_str("5").unwrap())));
        assert_eq!(array_info(&b"[128]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(u64::from_str("128").unwrap())));
        assert_eq!(array_info(&b"[129]"[..]), IResult::Done(&b""[..], ArrayInfo::Static(u64::from_str("129").unwrap())));
        
    }







    #[test]
    fn parse_void_definition() {
        assert_eq!(
            void_definition(&b"void2"[..]),
            IResult::Done(&b""[..], FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Void2),
                array: None,
                name: None,
            })
        );        
    }

    #[test]
    fn parse_field_definition() {
        assert_eq!(
            field_definition(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: None,
                name: Some(Ident(String::from("uptime_sec"))),
            })
        );

        
    }

    #[test]
    fn parse_const_definition() {
        assert_eq!(
            const_definition(&b"uint2 HEALTH_OK              = 0"[..]),
            IResult::Done(&b""[..], ConstDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint2),
                name: Ident(String::from("HEALTH_OK")),
                literal: Lit::Dec{sign: Sign::Implicit, value: String::from("0")},
            })
        );

        
    }





    #[test]
    fn parse_attribute_definition() {
        assert_eq!(
            attribute_definition(&b"void2"[..]),
            IResult::Done(&b""[..], AttributeDefinition::Field(FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Void2),
                array: None,
                name: None,
            }))
        );

        assert_eq!(
            attribute_definition(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], AttributeDefinition::Field(FieldDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint32),
                array: None,
                name: Some(Ident(String::from("uptime_sec"))),
            }))
        );
        
        assert_eq!(
            attribute_definition(&b"uint2 HEALTH_OK              = 0"[..]),
            IResult::Done(&b""[..], AttributeDefinition::Const(ConstDefinition{
                cast_mode: None,
                field_type: Ty::Primitive(PrimitiveType::Uint2),
                name: Ident(String::from("HEALTH_OK")),
                literal: Lit::Dec{sign: Sign::Implicit, value: String::from("0")},
            }))
        );

        
    }





    #[test]
    fn parse_line() {
        assert_eq!(
            line(&b"# Test comment"[..]),
            IResult::Done(&b""[..], Line::Comment(
                Comment(String::from(" Test comment"))
            ))
        );
        
        assert_eq!(
            line(&b"void2\n"[..]),
            IResult::Done(&b"\n"[..], Line::Definition {
                definition: AttributeDefinition::Field(FieldDefinition {
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Void2),
                    array: None,
                    name: None,
                }),
                comment: None
            })
        );
        
        assert_eq!(
            line(&b"void3"[..]),
            IResult::Done(&b""[..], Line::Definition {
                definition: AttributeDefinition::Field(FieldDefinition {
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Void3),
                    array: None,
                    name: None,
                }),
                comment: None
            })
        );

        assert_eq!(
            line(&b"void2 # test comment\n"[..]),
            IResult::Done(&b"\n"[..], Line::Definition {
                definition: AttributeDefinition::Field(FieldDefinition {
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Void2),
                    array: None,
                    name: None
                }),
                comment: Some(Comment(String::from(" test comment")))
            })
        );

        assert_eq!(
            line(&b"uint32 uptime_sec"[..]),
            IResult::Done(&b""[..], Line::Definition {
                definition: AttributeDefinition::Field(FieldDefinition {
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Uint32),
                    array: None,
                    name: Some(Ident(String::from("uptime_sec"))),
                }),
                comment: None,
            })
        );
        
        assert_eq!(
            line(&b"uint2 HEALTH_OK              = 0"[..]),
            IResult::Done(&b""[..], Line::Definition {
                definition: AttributeDefinition::Const(ConstDefinition {
                    cast_mode: None,
                    field_type: Ty::Primitive(PrimitiveType::Uint2),
                    name: Ident(String::from("HEALTH_OK")),
                    literal: Lit::Dec { sign: Sign::Implicit, value: String::from("0") },
                }),
                comment: None,
            })
        );
        
    }

    
    #[test]
    fn parse_lines() {
        assert_eq!(
            lines(&b"void2
# test comment
void3

void2 # test comment"[..]),
            IResult::Done(&b""[..], vec!(
                Line::Definition{definition: AttributeDefinition::Field(FieldDefinition{cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None}), comment: None},
                Line::Comment(Comment(String::from(" test comment"))),
                Line::Definition{definition: AttributeDefinition::Field(FieldDefinition{cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void3), array: None, name: None}), comment: None},
                Line::Empty,
                Line::Definition{definition: AttributeDefinition::Field(FieldDefinition{cast_mode: None, field_type: Ty::Primitive(PrimitiveType::Void2), array: None, name: None}), comment: Some(Comment(String::from(" test comment")))},
            ))  
        );
        
        
    }
    
    
    
}
