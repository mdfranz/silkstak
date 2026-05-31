use crate::agent::tools;
use crate::extras::subagents::prompt;
use crate::provider::{AnyAgent, AnyModel, OpenAiAgent, OpenAiModel};
use rig::agent::{Agent, AgentBuilder};
use rig::completion::CompletionModel;

fn build_explore_agent_inner<M: CompletionModel + 'static>(
    model: M,
    max_turns: usize,
    max_text_file_size: u64,
) -> Agent<M> {
    let tools: Vec<Box<dyn rig::tool::ToolDyn>> = vec![
        Box::new(tools::ReadTool::new(None, None, Some(max_text_file_size))),
        Box::new(tools::GrepTool::new(None, None)),
        Box::new(tools::FindFilesTool::new(None, None)),
        Box::new(tools::ListDirTool::new(None, None)),
        Box::new(tools::WriteTodoList::new(None, None)),
        #[cfg(feature = "memory")]
        Box::new(crate::extras::memory::MemoryRead::new(None, None)),
        #[cfg(feature = "memory")]
        Box::new(crate::extras::memory::MemorySearch::new(None, None)),
    ];

    AgentBuilder::new(model)
        .preamble(prompt::EXPLORE_PROMPT)
        .default_max_turns(max_turns)
        .tools(tools)
        .build()
}

pub(crate) async fn build_explore_agent(model: AnyModel, max_turns: usize) -> AnyAgent {
    // Use a reasonable default file size for subagent reads
    let max_text_file_size = 10 * 1024 * 1024;
    match model {
        AnyModel::OpenRouter(m) => {
            AnyAgent::OpenRouter(build_explore_agent_inner(m, max_turns, max_text_file_size))
        }
        AnyModel::OpenAI(m) => AnyAgent::OpenAI(match m {
            OpenAiModel::Responses(m) => {
                OpenAiAgent::Responses(build_explore_agent_inner(m, max_turns, max_text_file_size))
            }
            OpenAiModel::Completions(m) => OpenAiAgent::Completions(build_explore_agent_inner(
                m,
                max_turns,
                max_text_file_size,
            )),
        }),
        AnyModel::Anthropic(m) => {
            AnyAgent::Anthropic(build_explore_agent_inner(m, max_turns, max_text_file_size))
        }
        AnyModel::Gemini(m) => {
            AnyAgent::Gemini(build_explore_agent_inner(m, max_turns, max_text_file_size))
        }
        AnyModel::Ollama(m) => {
            AnyAgent::Ollama(build_explore_agent_inner(m, max_turns, max_text_file_size))
        }
    }
}
