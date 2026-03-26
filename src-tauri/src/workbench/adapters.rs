use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderOutputMode {
    PlainTextAssistant,
    StreamJson,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderExecutionPlan {
    pub program: String,
    pub args: Vec<String>,
    pub label: String,
    pub output_mode: ProviderOutputMode,
}

pub fn build_provider_plans(
    provider_id: &str,
    configured_command: Option<String>,
    prompt: &str,
) -> Vec<ProviderExecutionPlan> {
    match provider_id {
        "codex" => build_codex_plans(configured_command, prompt),
        "claude" => build_claude_plans(configured_command, prompt),
        "gemini" => build_gemini_plans(configured_command, prompt),
        "kimi" => build_kimi_plans(configured_command, prompt),
        _ => vec![],
    }
}

fn build_codex_plans(
    configured_command: Option<String>,
    prompt: &str,
) -> Vec<ProviderExecutionPlan> {
    if let Some(program) = configured_command {
        return vec![codex_plan(program, prompt)];
    }

    let mut plans = vec![codex_plan("codex".into(), prompt)];
    #[cfg(target_os = "windows")]
    {
        plans.push(ProviderExecutionPlan {
            program: "wsl.exe".into(),
            args: vec![
                "-e".into(),
                "codex".into(),
                "exec".into(),
                prompt.to_string(),
            ],
            label: "wsl.exe -e codex exec".into(),
            output_mode: ProviderOutputMode::PlainTextAssistant,
        });
    }
    plans
}

fn build_claude_plans(
    configured_command: Option<String>,
    prompt: &str,
) -> Vec<ProviderExecutionPlan> {
    let program = configured_command.unwrap_or_else(|| "claude".into());
    vec![ProviderExecutionPlan {
        label: format!("{program} -p"),
        program,
        args: vec![
            "-p".into(),
            prompt.to_string(),
            "--output-format".into(),
            "stream-json".into(),
        ],
        output_mode: ProviderOutputMode::StreamJson,
    }]
}

fn build_gemini_plans(
    configured_command: Option<String>,
    prompt: &str,
) -> Vec<ProviderExecutionPlan> {
    let program = configured_command.unwrap_or_else(|| "gemini".into());
    vec![ProviderExecutionPlan {
        label: format!("{program} -p"),
        program,
        args: vec![
            "-p".into(),
            prompt.to_string(),
            "--output-format".into(),
            "stream-json".into(),
        ],
        output_mode: ProviderOutputMode::StreamJson,
    }]
}

fn codex_plan(program: String, prompt: &str) -> ProviderExecutionPlan {
    if command_is_wsl(&program) {
        return ProviderExecutionPlan {
            label: format!("{program} -e codex exec"),
            program,
            args: vec![
                "-e".into(),
                "codex".into(),
                "exec".into(),
                prompt.to_string(),
            ],
            output_mode: ProviderOutputMode::PlainTextAssistant,
        };
    }

    ProviderExecutionPlan {
        label: format!("{program} exec"),
        program,
        args: vec!["exec".into(), prompt.to_string()],
        output_mode: ProviderOutputMode::PlainTextAssistant,
    }
}

fn build_kimi_plans(
    configured_command: Option<String>,
    prompt: &str,
) -> Vec<ProviderExecutionPlan> {
    let program = configured_command.unwrap_or_else(|| "kimi".into());
    vec![ProviderExecutionPlan {
        label: format!("{program} --print"),
        program,
        args: vec![
            "--print".into(),
            "-p".into(),
            prompt.to_string(),
            "--output-format".into(),
            "stream-json".into(),
        ],
        output_mode: ProviderOutputMode::StreamJson,
    }]
}

fn command_is_wsl(program: &str) -> bool {
    let stem = Path::new(program)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(program);
    stem.eq_ignore_ascii_case("wsl")
}

#[cfg(test)]
mod tests {
    use super::{build_provider_plans, ProviderOutputMode};

    #[test]
    fn codex_override_is_used_as_program() {
        let plans = build_provider_plans("codex", Some("C:\\tools\\codex.exe".into()), "ship it");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].program, "C:\\tools\\codex.exe");
        assert_eq!(plans[0].args, vec!["exec", "ship it"]);
        assert_eq!(plans[0].output_mode, ProviderOutputMode::PlainTextAssistant);
    }

    #[test]
    fn claude_uses_stream_json_print_mode() {
        let plans = build_provider_plans("claude", None, "ship it");
        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].args,
            vec!["-p", "ship it", "--output-format", "stream-json"]
        );
        assert_eq!(plans[0].output_mode, ProviderOutputMode::StreamJson);
    }

    #[test]
    fn gemini_uses_stream_json_print_mode() {
        let plans = build_provider_plans("gemini", None, "ship it");
        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].args,
            vec!["-p", "ship it", "--output-format", "stream-json"]
        );
    }

    #[test]
    fn kimi_uses_print_mode_with_stream_json() {
        let plans = build_provider_plans("kimi", None, "ship it");
        assert_eq!(plans.len(), 1);
        assert_eq!(
            plans[0].args,
            vec!["--print", "-p", "ship it", "--output-format", "stream-json"]
        );
        assert_eq!(plans[0].output_mode, ProviderOutputMode::StreamJson);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_default_includes_wsl_fallback_for_codex() {
        let plans = build_provider_plans("codex", None, "ship it");
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].program, "codex");
        assert_eq!(plans[1].program, "wsl.exe");
    }
}
