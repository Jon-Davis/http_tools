// MIT License
// 
// Copyright (c) 2020 Jonathon Davis
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software. 
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub struct PercentEncodedStr<'a>(&'a str);

impl<'a> PercentEncodedStr<'a> {
    pub fn new(s : &'a str) -> Self {
        PercentEncodedStr(s)
    }

    pub fn inner(&self) -> &str {
        self.0
    }
}

fn hex_to_byte(hex : u8) -> Option<u8> {
    match hex {
        hex if (48..=57).contains(&hex) => Some(hex - 48),
        hex if (65..=70).contains(&hex) => Some(hex - 55),
        hex if (97..=102).contains(&hex) => Some(hex - 87),
        _ => None,
    }
}

impl<'a> PartialEq<&str> for PercentEncodedStr<'a> {
    fn eq(&self, rhs: &&str) -> bool { 
        if self.0 == *rhs {
            return true;
        }
        
        let mut encoded = self.0.bytes();
        let mut unencoded = rhs.bytes();
        
        loop {
            let equal = match (encoded.next(), unencoded.next()) {
                (Some(ebyte), Some(ubyte)) if ebyte == b'%' => {
                    let first_byte = encoded.next().and_then(hex_to_byte);
                    let second_byte = encoded.next().and_then(hex_to_byte);
                    match (first_byte, second_byte) {
                        (Some(fb), Some(sb)) => ubyte == fb * 0x10 + sb,
                        _ => false,
                    }
                }
                (Some(ebyte), Some(ubyte)) if ebyte == ubyte => true,
                (Some(43), Some(32)) => true,
                (None, None) => return true,
                _ => false,
            };
            if !equal { return false; }
        }

    }
}