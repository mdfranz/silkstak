use crate::agent::tools;
use crate::extras::subagents::prompt;
use crate::provider::{AnyAgent, AnyModel, OpenAiAgent, OpenAiModel};
use rig::agent::{Agent, AgentBuilder};
use rig::completion::CompletionModel;

pub struct BuilderArgs {
    pub max_turns: usize,
    pub max_text_file_size: u64,
    pub max_read_lines: u64,
    pub max_grep_results: u64,
    pub max_find_results: u64,
    pub max_list_dir_entries: Option<u64>,
}

fn build_explore_agent_inner<M: CompletionModel + 'static>(
    model: M,
    args: BuilderArgs,
) -> Agent<M> {
    let preamble = prompt::explore_prompt();

    let tools: Vec<Box<dyn rig::tool::ToolDyn>> = vec![
        Box::new(tools::ReadTool::new(
            None,
            None,
            Some(args.max_text_file_size),
            args.max_read_lines,
        )),
        Box::new(tools::GrepTool::new(None, None, args.max_grep_results)),
        Box::new(tools::FindFilesTool::new(None, None, args.max_find_results)),
        Box::new(tools::ListDirTool::new(
            None,
            None,
            args.max_list_dir_entries,
        )),
        #[cfg(feature = "memory")]
        Box::new(crate::extras::memory::MemoryRead::new(None, None)),
        #[cfg(feature = "memory")]
        Box::new(crate::extras::memory::MemorySearch::new(None, None)),
    ];

    AgentBuilder::new(model)
        .preamble(&preamble)
        .default_max_turns(args.max_turns)
        .tools(tools)
        .build()
}

pub(crate) async fn build_explore_agent(
    model: AnyModel,
    max_turns: usize,
    cfg: &crate::config::Config,
) -> AnyAgent {
    let args = BuilderArgs {
        max_turns,
        max_text_file_size: cfg.max_text_file_size.unwrap_or(10 * 1024 * 1024),
        max_read_lines: cfg.resolve_subagent_max_read_lines(),
        max_grep_results: cfg.resolve_subagent_max_grep_results(),
        max_find_results: cfg.resolve_subagent_max_find_results(),
        max_list_dir_entries: cfg.resolve_subagent_max_list_dir_entries(),
    };

    match model {
        AnyModel::OpenAI(m) => AnyAgent::OpenAI(match m {
            OpenAiModel::Responses(m) => OpenAiAgent::Responses(build_explore_agent_inner(m, args)),
            OpenAiModel::Completions(m) => {
                OpenAiAgent::Completions(build_explore_agent_inner(m, args))
            }
        }),
        AnyModel::Anthropic(m) => AnyAgent::Anthropic(build_explore_agent_inner(m, args)),
        AnyModel::Gemini(m) => AnyAgent::Gemini(build_explore_agent_inner(m, args)),
        AnyModel::Ollama(m) => AnyAgent::Ollama(build_explore_agent_inner(m, args)),
    }
}
