default: fmt test clippy

[no-cd]
check *ARGS:
  cargo check {{ARGS}}

[no-cd]
fmt:
  cargo +nightly fmt -- \
    --config group_imports=StdExternalCrate \
    --config imports_granularity=Module \
    --config wrap_comments=true \
    --config comment_width=100 \
    --config format_code_in_doc_comments=true

[no-cd]
test *ARGS:
  cargo test {{ARGS}}

[no-cd]
clippy:
  cargo clippy --all-targets

[no-cd]
doc *ARGS:
  cargo +nightly doc {{ARGS}}
