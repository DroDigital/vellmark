// Shared by the build script and the renderer through `include!`.
//
// Shader composition: a line `#import name` is replaced by the text of `name.wgsl`,
// once, wherever it first appears. That is all the composition the shaders need.

/// Expands the imports of the shader called `name`. `source_of` returns the text of a
/// shader by name.
pub fn expand_shader(
    name: &str,
    source_of: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    fn go(
        name: &str,
        source_of: &dyn Fn(&str) -> Option<String>,
        seen: &mut Vec<String>,
        stack: &mut Vec<String>,
        out: &mut String,
    ) -> Result<(), String> {
        if stack.iter().any(|s| s == name) {
            return Err(format!("shader `{name}` imports itself, through {}", stack.join(" -> ")));
        }
        if seen.iter().any(|s| s == name) {
            return Ok(());
        }
        seen.push(name.to_owned());
        stack.push(name.to_owned());
        let source = source_of(name).ok_or_else(|| format!("shader `{name}` not found"))?;
        for line in source.lines() {
            match line.trim().strip_prefix("#import ") {
                Some(import) => go(import.trim(), source_of, seen, stack, out)?,
                None => {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        stack.pop();
        Ok(())
    }
    let mut out = String::new();
    go(name, source_of, &mut Vec::new(), &mut Vec::new(), &mut out)?;
    Ok(out)
}
