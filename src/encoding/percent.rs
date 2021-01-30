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

/// PercentEncodedStr is a wrapper around a `&str` that is percent encoded, such as a URI.
/// 
/// The main use of a PercentEncodedStr is comparing it to decoded `&str`. The wrapper doesn't
/// decode the string, it just enables comparisons with &str.
/// ```
/// use http_tools::encoding::PercentEncodedStr;
///
/// assert!(PercentEncodedStr::new("hello+world") == "hello world");
/// assert!(PercentEncodedStr::new("hello%20world") == "hello world");
/// assert!(PercentEncodedStr::new("%3c!html%3e") == "<!html>");
/// // If a PercentEncodedStr matches a &str verbatim, it will also return being equal to it 
/// assert!(PercentEncodedStr::new("%3c!html%3e") == "%3c!html%3e");
/// ```
#[derive(Debug)]
pub struct PercentEncodedStr<'a>(&'a str);

impl<'a> PercentEncodedStr<'a> {
    /// Creates a new PercentEncodedStr from a &str that is percent encoded
    /// # Example
    /// ```
    /// use http_tools::encoding::PercentEncodedStr;
    ///
    /// let percent_encoded_str = PercentEncodedStr::new("hello+world");
    /// ```
    pub fn new(s : &'a str) -> Self {
        PercentEncodedStr(s)
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

#[test]
fn test_percent_encoded_str(){
    assert!(PercentEncodedStr("hello+world") == "hello world");
    assert!(PercentEncodedStr("hello%20world") == "hello world");
    assert!(PercentEncodedStr("%3c!html%3e") == "<!html>");
    assert!(PercentEncodedStr("%2c.%2f%3b%27%5b%5d%3c%3e%3f%3a%22%7b%7d!%40%23%24%25%5e%26*()_%2b-%3d") == ",./;'[]<>?:\"{}!@#$%^&*()_+-=");
    assert!(PercentEncodedStr::new("%3c!html%3e") == "%3c!html%3e");
}