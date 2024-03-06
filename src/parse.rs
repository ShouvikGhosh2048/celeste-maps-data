// https://github.com/iSkLz/celestial-compass/blob/master/article.md

#[derive(Debug, Clone)]
pub enum Attribute {
    Bool(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Long(i64),
    Double(f64),
}

impl Attribute {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Byte(x) => Some(*x as i64),
            Self::Short(x) => Some(*x as i64),
            Self::Int(x) => Some(*x as i64),
            Self::Long(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_real(&self) -> Option<f64> {
        match self {
            Self::Float(x) => Some(*x as f64),
            Self::Double(x) => Some(*x),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(x) = self {
            Some(*x)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, Attribute)>,
    pub children: Vec<Element>,
}

impl Element {
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.0 == name)
            .map(|attribute| &attribute.1)
    }

    pub fn get_child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub package_name: String,
    pub root: Element,
}

// Parse string with 7-bit length encoding.
fn parse_string(bytes: &[u8], curr: &mut usize) -> Result<String, &'static str> {
    let mut string_length: usize = 0; // Can cause a problem on 32 bit.
    let mut bits_added = 0;

    loop {
        if bits_added == 35 {
            return Err("Bad string length");
        }

        let Some(&byte) = bytes.get(*curr) else {
            return Err("Bad string length");
        };
        *curr += 1;

        string_length |= ((byte & 0x7F) as usize) << bits_added;
        bits_added += 7;

        if byte & 0x80 == 0 {
            break;
        }
    }

    if *curr + string_length > bytes.len() {
        return Err("String length exceeds beyond map");
    }
    *curr += string_length;
    String::from_utf8(bytes[*curr - string_length..*curr].into()).map_err(|_| "Bad string")
}

fn parse_short(bytes: &[u8], curr: &mut usize) -> Result<i16, &'static str> {
    if *curr + 2 > bytes.len() {
        Err("Expected short")
    } else {
        let res = i16::from_le_bytes([bytes[*curr], bytes[*curr + 1]]);
        *curr += 2;
        Ok(res)
    }
}

fn parse_lookup_string(
    bytes: &[u8],
    curr: &mut usize,
    lookup_table: &[String],
) -> Result<String, &'static str> {
    let index = parse_short(bytes, curr)?;
    if index < 0 {
        return Err("Lookup index is negative");
    }

    if let Some(string) = lookup_table.get(index as usize) {
        Ok(string.clone())
    } else {
        Err("Lookup index exceeds the table length")
    }
}

fn parse_element(
    bytes: &[u8],
    curr: &mut usize,
    lookup_table: &[String],
) -> Result<Element, &'static str> {
    let name = parse_lookup_string(bytes, curr, lookup_table)?;

    let Some(&attribute_count) = bytes.get(*curr) else {
        return Err("Expected byte");
    }; // TODO: Should this be a i8 or u8?
    *curr += 1;

    let mut attributes = vec![];
    for _ in 0..attribute_count {
        let attribute_name = parse_lookup_string(bytes, curr, lookup_table)?;

        let encoded_value_type = bytes[*curr];
        *curr += 1;
        let attribute = match encoded_value_type {
            0 => {
                let Some(&byte) = bytes.get(*curr) else {
                    return Err("Expected boolean");
                };
                *curr += 1;
                let boolean = byte != 0;
                Attribute::Bool(boolean)
            }
            1 => {
                let Some(&byte) = bytes.get(*curr) else {
                    return Err("Expected byte");
                };
                *curr += 1;
                Attribute::Byte(byte)
            }
            2 => {
                let short = parse_short(bytes, curr)?;
                Attribute::Short(short)
            }
            3 => {
                if *curr + 4 > bytes.len() {
                    return Err("Expected int");
                }
                let int = i32::from_le_bytes([
                    bytes[*curr],
                    bytes[*curr + 1],
                    bytes[*curr + 2],
                    bytes[*curr + 3],
                ]);
                *curr += 4;
                Attribute::Int(int)
            }
            4 => {
                if *curr + 4 > bytes.len() {
                    return Err("Expected float");
                }
                let float = f32::from_le_bytes([
                    bytes[*curr],
                    bytes[*curr + 1],
                    bytes[*curr + 2],
                    bytes[*curr + 3],
                ]);
                *curr += 4;
                Attribute::Float(float)
            }
            5 => Attribute::String(parse_lookup_string(bytes, curr, lookup_table)?),
            6 => Attribute::String(parse_string(bytes, curr)?),
            7 => {
                let string_length = parse_short(bytes, curr)?;
                if string_length < 0 {
                    return Err("Negative string length");
                }
                if string_length % 2 == 1 {
                    return Err("Length encoded string has odd length");
                }
                if *curr + string_length as usize > bytes.len() {
                    return Err("String length exceeds beyond map");
                }

                let mut string = String::new();
                for _ in 0..string_length / 2 {
                    let repeat_count = bytes[*curr];
                    let character: char = bytes[*curr + 1].into();
                    for _ in 0..repeat_count {
                        string.push(character);
                    }
                    *curr += 2;
                }
                Attribute::String(string)
            }
            8 => {
                if *curr + 8 > bytes.len() {
                    return Err("Expected long");
                }
                let long = i64::from_le_bytes([
                    bytes[*curr],
                    bytes[*curr + 1],
                    bytes[*curr + 2],
                    bytes[*curr + 3],
                    bytes[*curr + 4],
                    bytes[*curr + 5],
                    bytes[*curr + 6],
                    bytes[*curr + 7],
                ]);
                *curr += 8;
                Attribute::Long(long)
            }
            9 => {
                if *curr + 8 > bytes.len() {
                    return Err("Expected double");
                }
                let double = f64::from_le_bytes([
                    bytes[*curr],
                    bytes[*curr + 1],
                    bytes[*curr + 2],
                    bytes[*curr + 3],
                    bytes[*curr + 4],
                    bytes[*curr + 5],
                    bytes[*curr + 6],
                    bytes[*curr + 7],
                ]);
                *curr += 8;
                Attribute::Double(double)
            }
            _ => {
                return Err("Unspecified encoded value.");
            }
        };

        attributes.push((attribute_name, attribute));
    }

    let child_count = parse_short(bytes, curr)?;
    if child_count < 0 {
        return Err("Negative child count");
    }

    let mut children = vec![];
    for _ in 0..child_count {
        children.push(parse_element(bytes, curr, lookup_table)?);
    }

    Ok(Element {
        name,
        attributes,
        children,
    })
}

pub fn parse(map: &[u8]) -> Result<Map, &'static str> {
    if map.len() < 12 || map[0] != 11 || &map[1..12] != "CELESTE MAP".as_bytes() {
        return Err("Map should start with CELESTE MAP");
    }

    let mut curr = 12;

    let package_name = parse_string(map, &mut curr)?;

    let Ok(lookup_table_size) = parse_short(map, &mut curr) else {
        return Err("Bad lookup table size");
    };
    if lookup_table_size < 0 {
        return Err("Lookup table size is negative");
    }

    let mut lookup_table = vec![];
    for _ in 0..lookup_table_size {
        lookup_table.push(parse_string(map, &mut curr)?);
    }

    let root = parse_element(map, &mut curr, &lookup_table)?;

    if curr != map.len() {
        return Err("Extra bytes after parsing root");
    }

    Ok(Map { package_name, root })
}
