# Coding Standards

## Comments

Label each step of a function body with a short comment, so the comments alone read as an outline of the function's logic:

```rust
// Report context load warnings without failing the command
for warning in context.warnings()
{
    let _ = writeln!(stderr, "warning: {warning}");
}

// Run command
let start = Instant::now();
let result = command.run(&context);
let result = render::emit(result);

// Check for updates
if let Some(update) = checker(
    context.global_config.updates.check_frequency,
    &mut context.global_state
)
{
    let _ = writeln!(stderr, "{update}");
}
```

This deliberately departs from the common advice to comment only what the code cannot say for itself. Reading a function's logic in English takes less effort than mentally parsing the code. Each label also states the step's intent, so review can check the code against it, and marks where one step ends and the next begins.

The usual objection is that comments drift from the code. Keep them current instead: whoever changes a step updates its label in the same change, and review checks that each label still matches its step. A short label is the default, not a hard rule. When a step needs more explanation, such as why it is done this way or what it guards against, write a fuller comment.

Tests follow the same practice with `// Arrange`, `// Act` and `// Assert`.
