/// Asks the user for confirmation. Returns `true` if accepted.
pub fn confirm() -> bool {
    dialoguer::Confirm::new()
        .with_prompt("Do you wish to proceed?")
        .default(true)
        .interact()
        .unwrap_or(false)
}
