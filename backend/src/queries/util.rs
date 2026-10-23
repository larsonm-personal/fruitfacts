// replace underscores with spaces (the front end does space->underscore)
pub fn path_to_name(input: &str) -> String {
    str::replace(input, "_", " ")
}
