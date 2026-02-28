use std::fs;

use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = kaspa::stub_info()?;
    stub.generate()?;

    post_process_stub_file("kaspa.pyi");

    Ok(())
}

fn post_process_stub_file(path: &str) {
    let content = fs::read_to_string(path).unwrap();

    let content = strip_py_prefix_from_enums(content);
    let content = rename_none_enum_variant(content);
    let content = fix_rpc_method_signatures(content);
    let content = remove_duplicate_default_none(content);
    let content = fix_utxo_processor_event_listener_overloads(content);
    let content = append_rpc_types(content);

    fs::write(path, content).unwrap();
}

/// Appends the contents of kaspa_rpc.pyi to the stub file.
/// This includes manually typed RPC request/response TypedDicts.
fn append_rpc_types(content: String) -> String {
    let rpc_types_path = "kaspa_rpc.pyi";

    match fs::read_to_string(rpc_types_path) {
        Ok(rpc_content) => {
            format!(
                "{}\n\n\
                # =============================================================================\n\
                # RPC Types (from {})\n\
                # =============================================================================\n\n\
                {}",
                content.trim_end(),
                rpc_types_path,
                rpc_content.trim()
            )
        }
        Err(e) => {
            eprintln!("Warning: Could not read {}: {}", rpc_types_path, e);
            content
        }
    }
}

/// Removes the "Py" prefix from enum class names and all their references.
/// e.g., `class PyNetworkType(enum.Enum)` becomes `class NetworkType(enum.Enum)`
fn strip_py_prefix_from_enums(content: String) -> String {
    let mut enum_names: Vec<String> = Vec::new();

    for line in content.lines() {
        if let Some(start) = line.find("class Py")
            && line.contains("(enum.Enum)")
        {
            let after_class = &line[start + 6..];
            if let Some(paren_pos) = after_class.find('(') {
                let class_name = &after_class[..paren_pos];
                if class_name.starts_with("Py") {
                    enum_names.push(class_name.to_string());
                }
            }
        }
    }

    let mut result = content;
    for py_name in &enum_names {
        if let Some(stripped) = py_name.strip_prefix("Py") {
            result = result.replace(py_name, stripped);
        }
    }

    result
}

/// Renames `None = ...` to `_None = ...` for enum variants.
/// This is necessary because `None` is a reserved keyword in Python.
fn rename_none_enum_variant(content: String) -> String {
    content.replace("    None = ...", "    _None = ...")
}

/// Removes duplicate `= None` that incorrectly follows another default value.
/// e.g., `language: str | Language = Language.English = None` becomes
///       `language: str | Language = Language.English`
fn remove_duplicate_default_none(content: String) -> String {
    use regex::Regex;

    // Match patterns like `= SomeValue = None` where SomeValue is an identifier
    // (possibly with dots like `Language.English` or `Encoding.Borsh`)
    let re = Regex::new(r"(= \w+(?:\.\w+)?) = None\b").unwrap();
    re.replace_all(&content, "$1").to_string()
}

/// Inserts overload stubs for UtxoProcessor event listener APIs.
///
/// The Rust/PyO3 signature supports two call patterns:
/// - add_event_listener(callback, *args, **kwargs)
/// - add_event_listener(targets, callback, *args, **kwargs)
///
/// pyo3-stub-gen currently emits a single signature with `callback: Optional[Any] = None`,
/// which makes the `callback` appear optional in all cases. Overloads improve type checking
/// without changing runtime behavior.
fn fix_utxo_processor_event_listener_overloads(content: String) -> String {
    let add_impl = "    def add_event_listener(self, event_or_callback: typing.Any, callback: typing.Optional[typing.Any] = None, *args: typing.Any, **kwargs: typing.Any) -> None:";
    let add_overloads = concat!(
        "    @typing.overload\n",
        "    def add_event_listener(self, callback: typing.Callable[..., typing.Any], *args: typing.Any, **kwargs: typing.Any) -> None: ...\n",
        "    @typing.overload\n",
        "    def add_event_listener(self, event_or_callback: builtins.str | UtxoProcessorEvent | typing.Sequence[builtins.str | UtxoProcessorEvent], callback: typing.Callable[..., typing.Any], *args: typing.Any, **kwargs: typing.Any) -> None: ...\n",
    );

    let remove_impl = "    def remove_event_listener(self, event_or_callback: typing.Any, callback: typing.Optional[typing.Any] = None) -> None:";
    let remove_overloads = concat!(
        "    @typing.overload\n",
        "    def remove_event_listener(self, event_or_callback: typing.Callable[..., typing.Any]) -> None: ...\n",
        "    @typing.overload\n",
        "    def remove_event_listener(self, event_or_callback: builtins.str | UtxoProcessorEvent | typing.Sequence[builtins.str | UtxoProcessorEvent], callback: typing.Optional[typing.Callable[..., typing.Any]] = None) -> None: ...\n",
    );

    let mut out = content;

    if out.contains(add_impl) {
        out = out.replace(add_impl, &format!("{add_overloads}{add_impl}"));
    } else {
        eprintln!("Warning: Could not find UtxoProcessor.add_event_listener signature to overload");
    }

    if out.contains(remove_impl) {
        out = out.replace(remove_impl, &format!("{remove_overloads}{remove_impl}"));
    } else {
        eprintln!(
            "Warning: Could not find UtxoProcessor.remove_event_listener signature to overload"
        );
    }

    out
}

/// Converts a snake_case string to PascalCase.
/// e.g., "get_block_count" -> "GetBlockCount"
fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Fixes RPC method signatures in the stub file to use proper TypedDict types.
///
/// Transforms:
/// - `def method_name(self, request: typing.Optional[dict] = None) -> typing.Any: ...`
///   to `def method_name(self, request: MethodNameRequest | None = None) -> MethodNameResponse: ...`
/// - `def method_name(self, request: dict) -> typing.Any: ...`
///   to `def method_name(self, request: MethodNameRequest) -> MethodNameResponse: ...`
fn fix_rpc_method_signatures(content: String) -> String {
    let mut result = String::with_capacity(content.len());

    for line in content.lines() {
        let transformed = transform_rpc_method_line(line);
        result.push_str(&transformed);
        result.push('\n');
    }

    // Remove trailing newline to match original behavior
    if result.ends_with('\n') && !content.ends_with('\n') {
        result.pop();
    }

    result
}

/// Transform a single line if it's an RPC method signature.
fn transform_rpc_method_line(line: &str) -> String {
    // Check if this looks like an RPC method (has request parameter and returns typing.Any)
    if !line.contains("request:") || !line.contains("-> typing.Any") {
        return line.to_string();
    }

    // Extract method name: look for "def method_name("
    let Some(def_start) = line.find("def ") else {
        return line.to_string();
    };

    let after_def = &line[def_start + 4..];
    let Some(paren_pos) = after_def.find('(') else {
        return line.to_string();
    };

    let method_name = &after_def[..paren_pos];

    // Skip non-RPC methods (subscribe/unsubscribe, connect, disconnect, etc.)
    if method_name.starts_with("subscribe")
        || method_name.starts_with("unsubscribe")
        || method_name == "connect"
        || method_name == "disconnect"
        || method_name == "start"
        || method_name == "stop"
        || method_name == "on"
        || method_name == "remove_listener"
    {
        return line.to_string();
    }

    // Convert method name to PascalCase for TypedDict names
    let pascal_name = snake_to_pascal(method_name);
    let request_type = format!("{}Request", pascal_name);
    let response_type = format!("{}Response", pascal_name);

    let mut transformed = line.to_string();

    // Replace optional dict parameter: `request: typing.Optional[dict] = None`
    // with `request: XxxRequest | None = None`
    if transformed.contains("typing.Optional[dict] = None") {
        transformed = transformed.replace(
            "typing.Optional[dict] = None",
            &format!("{} | None = None", request_type),
        );
    }
    // Replace required dict parameter: `request: dict)`
    // with `request: XxxRequest)`
    else if transformed.contains("request: dict)") {
        transformed = transformed.replace("request: dict)", &format!("request: {})", request_type));
    }

    // Replace return type: `-> typing.Any` with `-> XxxResponse`
    transformed = transformed.replace("-> typing.Any", &format!("-> {}", response_type));

    transformed
}
