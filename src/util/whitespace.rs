/// Remove all white spaces and line breaks
///
/// Alternative:
/// fn `remove_whitespace_mut(s`: &mut String) {
///   s.retain(|c| !`c.is_whitespace()`);
/// }
///
/// [Source]: https://stackoverflow.com/questions/57063777/remove-all-whitespace-from-a-string
pub fn remove_all(s: &str) -> String {
  s.chars().filter(|c| !c.is_whitespace()).collect()
}
