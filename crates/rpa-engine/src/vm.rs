//! The Automation VM - the central execution engine.

use rpa_core::context::RetryConfig;
use rpa_core::error::{RpaError, Result};
use rpa_core::instruction::Instruction;
use rpa_core::task::TaskResult;
use rpa_core::traits::{Actor, JsRuntime, Perceptor, WorkflowProvider};
use rpa_core::value::Value;

use crate::cancellation::CancellationToken;
use crate::events::ExecutionEvent;
use crate::executor::Executor;
use crate::finder::{MultiStrategyFinder, PerceptorEntry, StrategyType};

use std::collections::HashMap;
use std::sync::Arc;

/// Strategy for finding elements.
#[derive(Debug, Clone)]
pub enum FindStrategy {
    /// Try strategies in the default order: UIA → Text → Image.
    Sequential,
    /// Try strategies in a custom order.
    Custom(Vec<StrategyType>),
}

/// Configuration for the VM.
#[derive(Debug, Clone)]
pub struct VmConfig {
    pub retry: RetryConfig,
    pub default_timeout_ms: u64,
    pub find_strategy: FindStrategy,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            retry: RetryConfig::default(),
            default_timeout_ms: 30_000,
            find_strategy: FindStrategy::Sequential,
        }
    }
}

/// The Automation VM - orchestrates all RPA operations.
pub struct Vm {
    perceptors: Vec<Arc<dyn Perceptor>>,
    perceptor_strategies: Vec<StrategyType>,
    actor: Arc<dyn Actor>,
    js_runtime: Option<Arc<dyn JsRuntime>>,
    workflow_provider: Option<Arc<dyn WorkflowProvider>>,
    config: VmConfig,
}

impl Vm {
    /// Create a new VM with default configuration.
    pub fn new(config: VmConfig) -> Self {
        Self {
            perceptors: Vec::new(),
            perceptor_strategies: Vec::new(),
            actor: Arc::new(NoopActor),
            js_runtime: None,
            workflow_provider: None,
            config,
        }
    }

    /// Add a perceptor for a specific strategy.
    pub fn with_perceptor(mut self, strategy_type: StrategyType, perceptor: impl Perceptor + 'static) -> Self {
        self.perceptors.push(Arc::new(perceptor));
        self.perceptor_strategies.push(strategy_type);
        self
    }

    /// Set the actor for UI operations.
    pub fn with_actor(mut self, actor: impl Actor + 'static) -> Self {
        self.actor = Arc::new(actor);
        self
    }

    /// Set the JS runtime.
    pub fn with_js_runtime(mut self, js: impl JsRuntime + 'static) -> Self {
        self.js_runtime = Some(Arc::new(js));
        self
    }

    /// Set the workflow provider.
    pub fn with_workflow_provider(mut self, provider: impl WorkflowProvider + 'static) -> Self {
        self.workflow_provider = Some(Arc::new(provider));
        self
    }

    /// Get a reference to the VM config.
    pub fn config(&self) -> &VmConfig {
        &self.config
    }

    /// Build a MultiStrategyFinder from the registered perceptors.
    fn build_finder(&self) -> MultiStrategyFinder {
        let entries: Vec<PerceptorEntry> = self
            .perceptors
            .iter()
            .zip(self.perceptor_strategies.iter())
            .map(|(p, s)| PerceptorEntry {
                strategy_type: *s,
                perceptor: Arc::clone(p),
            })
            .collect();
        MultiStrategyFinder::new(entries)
    }

    /// Run a sequence of instructions.
    pub async fn run(&self, instructions: &[Instruction]) -> Result<TaskResult> {
        let start = std::time::Instant::now();
        let cancellation = CancellationToken::new();
        let (event_tx, _) = tokio::sync::broadcast::channel::<ExecutionEvent>(1024);

        let mut ctx = rpa_core::context::Context::new();
        ctx.retry_config = self.config.retry.clone();

        let finder = self.build_finder();

        let mut executor = Executor::new(
            &finder,
            &self.actor,
            &mut ctx,
            cancellation,
            Some(event_tx),
        );

        let result = executor.execute_block(instructions).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        let steps_executed = executor.steps_executed();

        match result {
            Ok(Some(value)) => Ok(TaskResult {
                output: Some(value),
                duration_ms,
                steps_executed,
            }),
            Ok(None) => Ok(TaskResult {
                output: None,
                duration_ms,
                steps_executed,
            }),
            Err(e) => Err(e),
        }
    }

    /// Run a workflow by name with arguments.
    pub async fn run_workflow(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<TaskResult> {
        let provider = self
            .workflow_provider
            .as_ref()
            .ok_or_else(|| RpaError::WorkflowNotFound("No workflow provider registered".into()))?;

        let instructions = provider.get_workflow(name)?;

        let start = std::time::Instant::now();
        let cancellation = CancellationToken::new();
        let (event_tx, _) = tokio::sync::broadcast::channel::<ExecutionEvent>(1024);

        let mut ctx = rpa_core::context::Context::new();
        for (key, value) in args {
            ctx.set_var(key, value);
        }
        ctx.retry_config = self.config.retry.clone();
        ctx.push_call(name.to_string());

        let finder = self.build_finder();

        let mut executor = Executor::new(
            &finder,
            &self.actor,
            &mut ctx,
            cancellation,
            Some(event_tx),
        );

        let result = executor.execute_block(&instructions).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        let steps_executed = executor.steps_executed();

        match result {
            Ok(Some(value)) => Ok(TaskResult {
                output: Some(value),
                duration_ms,
                steps_executed,
            }),
            Ok(None) => Ok(TaskResult {
                output: None,
                duration_ms,
                steps_executed,
            }),
            Err(e) => Err(e),
        }
    }

    /// Cancel a running VM execution.
    pub async fn cancel(&self, cancellation: &CancellationToken) -> Result<()> {
        cancellation.cancel();
        Ok(())
    }
}

/// A no-op actor used as a default placeholder.
struct NoopActor;

#[async_trait::async_trait]
impl Actor for NoopActor {
    async fn click(
        &self,
        _element: &rpa_core::element::Element,
        _button: rpa_core::instruction::MouseButton,
    ) -> Result<()> {
        Err(RpaError::Action("No actor registered".into()))
    }

    async fn double_click(&self, _element: &rpa_core::element::Element) -> Result<()> {
        Err(RpaError::Action("No actor registered".into()))
    }

    async fn input_text(
        &self,
        _element: &rpa_core::element::Element,
        _text: &str,
        _clear_first: bool,
    ) -> Result<()> {
        Err(RpaError::Action("No actor registered".into()))
    }

    async fn key_press(&self, _key: &str, _modifiers: Vec<rpa_core::instruction::ModifierKey>) -> Result<()> {
        Err(RpaError::Action("No actor registered".into()))
    }

    async fn scroll(
        &self,
        _element: &rpa_core::element::Element,
        _direction: rpa_core::instruction::ScrollDirection,
        _amount: u32,
    ) -> Result<()> {
        Err(RpaError::Action("No actor registered".into()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rpa_core::context::Context;
    use rpa_core::element::{Element, Rect};
    use rpa_core::instruction::MouseButton;
    use rpa_core::target::Target;
    use rpa_core::traits::Perceptor;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct TestActor {
        clicks: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl Actor for TestActor {
        async fn click(&self, el: &Element, _button: MouseButton) -> Result<()> {
            self.clicks.lock().unwrap().push(el.id.clone());
            Ok(())
        }
        async fn double_click(&self, _el: &Element) -> Result<()> {
            Ok(())
        }
        async fn input_text(&self, el: &Element, text: &str, _clear: bool) -> Result<()> {
            self.clicks.lock().unwrap().push(format!("input:{}:{}", el.id, text));
            Ok(())
        }
        async fn key_press(&self, _key: &str, _modifiers: Vec<rpa_core::instruction::ModifierKey>) -> Result<()> {
            Ok(())
        }
        async fn scroll(&self, _el: &Element, _dir: rpa_core::instruction::ScrollDirection, _amt: u32) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Clone)]
    struct TestPerceptor;

    #[async_trait::async_trait]
    impl Perceptor for TestPerceptor {
        async fn find(&self, target: &Target, _ctx: &Context) -> Result<Element> {
            Ok(Element {
                id: format!("mock_{:?}", target),
                bounds: Rect::new(10, 20, 100, 50),
                text: Some("mock".into()),
                element_type: Some("Button".into()),
                platform_handle: None,
            })
        }
        async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>> {
            self.find(target, ctx).await.map(|el| vec![el])
        }
    }

    #[tokio::test]
    async fn vm_run_click_instruction() {
        let clicks = Arc::new(Mutex::new(Vec::new()));

        let vm = Vm::new(VmConfig::default())
            .with_perceptor(StrategyType::UIA, TestPerceptor)
            .with_actor(TestActor { clicks: clicks.clone() });

        let instructions = vec![
            Instruction::Click {
                target: Target::by_name("按钮"),
                button: MouseButton::Left,
            },
        ];

        let result = vm.run(&instructions).await.unwrap();
        assert!(result.duration_ms > 0 || result.steps_executed >= 1);
        assert!(!clicks.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn vm_run_wait_instruction() {
        let vm = Vm::new(VmConfig::default())
            .with_perceptor(StrategyType::UIA, TestPerceptor)
            .with_actor(TestActor { clicks: Arc::new(Mutex::new(Vec::new())) });

        let instructions = vec![
            Instruction::Wait { duration_ms: 10 },
        ];

        let result = vm.run(&instructions).await.unwrap();
        assert!(result.duration_ms >= 10);
    }

    #[tokio::test]
    async fn vm_run_loop_instruction() {
        let vm = Vm::new(VmConfig::default())
            .with_perceptor(StrategyType::UIA, TestPerceptor)
            .with_actor(TestActor { clicks: Arc::new(Mutex::new(Vec::new())) });

        let instructions = vec![
            Instruction::Loop {
                max: Some(3),
                condition: None,
                body: vec![Instruction::Wait { duration_ms: 1 }],
            },
        ];

        let result = vm.run(&instructions).await.unwrap();
        assert!(result.steps_executed >= 3);
    }
}