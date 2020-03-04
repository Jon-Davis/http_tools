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

pub trait HttpToolsContainer<T> {
    /// This method is an interface method that allows http_tools to accept
    /// multiple different containers of items to validate http requests
    /// supported containers include &[T], Vec[T], HashSet[T], BTreeSet<T>, LinkedList<T>, Range<T>,
    /// and specially formated strings {options|seperated|by|vertical|columns}
    fn http_tools_contains(&self, item: &T) -> bool;
}
// implementation for std::hashset
impl<T : std::hash::Hash + Eq> HttpToolsContainer<T> for &std::collections::HashSet<T> {
    fn http_tools_contains(&self, item: &T) -> bool { self.contains(item) }
}
// implementation for std::hashset
impl<T : Ord> HttpToolsContainer<T> for &std::collections::BTreeSet<T> {
    fn http_tools_contains(&self, item: &T) -> bool { self.contains(item) }
}
// implementation for std::vec
impl<U, T> HttpToolsContainer<U> for &[T] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { 
        for option in *self {
            if option == item {return true}
        }
        false
    }
}
// implementation for std::LinkedList
impl<T : PartialEq<T>> HttpToolsContainer<T> for &std::collections::LinkedList<T> {
    fn http_tools_contains(&self, item: &T) -> bool { self.contains(item) }
}
// implementation for std::Range
impl<T : PartialEq<T> + std::cmp::PartialOrd + std::ops::RangeBounds<T>> HttpToolsContainer<T> for std::ops::Range<T> {
    fn http_tools_contains(&self, item: &T) -> bool { self.contains(item) }
}
// implementation for string
impl<'a, 'b : 'a, T : PartialEq<&'a str>> HttpToolsContainer<T> for &'b str {
    fn http_tools_contains(&self, item: &T) -> bool { 
        if *self == "{}" || *item == self { return true; }
        if self.starts_with('{') && self.ends_with('}') {
            if let Some(substring) = self.get(1..self.len()-1) {
                for option in substring.split("|") {
                    if item == &option {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}
// implementation for function
impl<T> HttpToolsContainer<T> for &dyn Fn(&T) -> bool {
    fn http_tools_contains(&self, item: &T) -> bool {
        self(item)
    }
}
// slice implments
impl<U, T> HttpToolsContainer<U> for [T; 1] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 2] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 3] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 4] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 5] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 6] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 7] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 8] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 9] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}
impl<U, T> HttpToolsContainer<U> for [T; 10] where T : PartialEq<U>{
    fn http_tools_contains(&self, item: &U) -> bool { (self as &[T]).http_tools_contains(item) }
}