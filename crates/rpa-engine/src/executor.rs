//! VM instruction executor.

use rpa_core::condition::Condition;
use rpa_core::context::Context;
use rpa_core::element::{Element, Rect};
use rpa_core::error::{RpaError, Result};
use rpa_core::instruction::{Instruction, MouseButton, ScrollDirection};
use rpa_core::target::Target;
use rpa_core::traits::Actor;
use rpa_core::value::Value;

use crate::cancellation::CancellationToken;
use crate::events::ExecutionEvent;
use crate::finder::MultiStrategyFinder;
use crate::retry;

use std::sync::Arc;
use std::time::Instant;

/// The instruction executor that dispatches VM operations.
pub struct Executor<'a> {
    /// The element finder.
    finder: &'a MultiStrategyFinder,
    /// The actor for UI operations.
    actor: &'a Arc<dyn Actor>,
    /// Execution context (mutable).
    ctx: &'a mut Context,
    /// Cancellation token.
    cancellation: CancellationToken,
    /// Start time of execution.
    start_time: Instant,
    /// Number of instructions executed.
    steps_executed: u32,
    /// Event sender for broadcasting execution events.
    event_tx: Option<tokio::sync::broadcast::Sender<ExecutionEvent>>,
}

/// Special control flow signals for the executor.
enum ControlFlow {
    /// Continue to the next instruction.
    Continue,
    /// Break out of the current loop.
    Break,
    /// Return a value from the current workflow.
    Return(Value),
}

impl<'a> Executor<'a> {
    /// Create a new executor.
    pub fn new(
        finder: &'a MultiStrategyFinder,
        actor: &'a Arc<dyn Actor>,
        ctx: &'a mut Context,
        cancellation: CancellationToken,
        event_tx: Option<tokio::sync::broadcast::Sender<ExecutionEvent>>,
    ) -> Self {
        Self {
            finder,
            actor,
            ctx,
            cancellation,
            start_time: Instant::now(),
            steps_executed: 0,
            event_tx,
        }
    }

    /// Execute a sequence of instructions.
    pub fn execute_block<'b>(
        &'b mut self,
        instructions: &'b [Instruction],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Value>>> + Send + 'b>> {
        Box::pin(async move {
            for (index, instr) in instructions.iter().enumerate() {
                if self.cancellation.is_cancelled() {
                    return Err(RpaError::Cancelled);
                }

                self.emit_event(ExecutionEvent::InstructionStart {
                    index,
                    description: format!("{:?}", instr),
                });

                match self.execute(instr).await? {
                    ControlFlow::Continue => {
                        self.steps_executed += 1;
                        self.emit_event(ExecutionEvent::InstructionComplete {
                            index,
                            duration_ms: 0,
                        });
                    }
                    ControlFlow::Break => break,
                    ControlFlow::Return(value) => return Ok(Some(value)),
                }
            }

            Ok(None)
        })
    }

    /// Execute a single instruction, returning a control flow signal.
    async fn execute(&mut self, instr: &Instruction) -> Result<ControlFlow> {
        match instr {
            Instruction::Click { target, button } => {
                self.exec_click(target, button).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::DoubleClick { target } => {
                let el = self.find_element(target).await?;
                self.actor.double_click(&el).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::Input {
                target,
                text,
                clear_first,
            } => {
                let el = self.find_element(target).await?;
                self.actor.input_text(&el, text, *clear_first).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::KeyPress { key, modifiers } => {
                self.actor.key_press(key, modifiers.clone()).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::Extract {
                target,
                spec: _,
                into_var,
            } => {
                let el = self.find_element(target).await?;
                let value = el.text.clone().map(Value::String).unwrap_or(Value::Null);
                self.ctx.set_var(into_var.clone(), value.clone());
                self.emit_event(ExecutionEvent::VariableSet {
                    name: into_var.clone(),
                    value,
                });
                Ok(ControlFlow::Continue)
            }
            Instruction::Wait { duration_ms } => {
                tokio::time::sleep(std::time::Duration::from_millis(*duration_ms)).await;
                Ok(ControlFlow::Continue)
            }
            Instruction::WaitFor {
                target,
                timeout_ms,
                interval_ms,
            } => {
                self.exec_wait_for(target, *timeout_ms, *interval_ms).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::Call { workflow, args: _ } => {
                self.emit_event(ExecutionEvent::WorkflowCall {
                    name: workflow.clone(),
                });
                Ok(ControlFlow::Continue)
            }
            Instruction::Loop {
                max,
                condition,
                body,
            } => self.exec_loop(max, condition, body).await,
            Instruction::If {
                condition,
                then_body,
                else_body,
            } => self.exec_if(condition, then_body, else_body).await,
            Instruction::Break => Ok(ControlFlow::Break),
            Instruction::Return(value) => Ok(ControlFlow::Return(value.clone())),
            Instruction::Log { message, level } => {
                match level {
                    rpa_core::instruction::LogLevel::Trace => tracing::trace!("{}", message),
                    rpa_core::instruction::LogLevel::Debug => tracing::debug!("{}", message),
                    rpa_core::instruction::LogLevel::Info => tracing::info!("{}", message),
                    rpa_core::instruction::LogLevel::Warn => tracing::warn!("{}", message),
                    rpa_core::instruction::LogLevel::Error => tracing::error!("{}", message),
                }
                Ok(ControlFlow::Continue)
            }
            Instruction::Scroll {
                target,
                direction,
                amount,
            } => {
                let el = self.find_element(target).await?;
                self.actor.scroll(&el, direction.clone(), *amount).await?;
                Ok(ControlFlow::Continue)
            }
            // ──────────────────────────────
            // Non-UIA / Mouse Operations
            // ──────────────────────────────
            Instruction::MouseMove { x, y } => {
                self.actor.mouse_move(*x, *y).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::MouseDown { button, x, y } => {
                self.actor.mouse_down(button.clone(), *x, *y).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::MouseUp { button, x, y } => {
                self.actor.mouse_up(button.clone(), *x, *y).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::Drag { from, to, button } => {
                self.exec_drag(from, to, button.clone()).await?;
                Ok(ControlFlow::Continue)
            }
            // ──────────────────────────────
            // Window Operations
            // ──────────────────────────────
            Instruction::SetForeground { target } => {
                let el = self.find_element(target).await?;
                self.actor.set_foreground(&el).await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::MoveWindow { .. } => {
                // MoveWindow requires WindowPerceptor which is not yet wired to Executor.
                // Return an error indicating this is not yet implemented.
                Err(RpaError::InvalidInstruction(
                    "MoveWindow requires WindowPerceptor".into(),
                ))
            }
            // ──────────────────────────────
            // Screenshot & OCR
            // ──────────────────────────────
            Instruction::Screenshot { target, region, save_path } => {
                self.exec_screenshot(target.as_ref(), region.as_ref(), save_path.as_deref())
                    .await?;
                Ok(ControlFlow::Continue)
            }
            Instruction::OcrRegion { target, region, into_var } => {
                let text = self.exec_ocr_region(target, region).await?;
                self.ctx.set_var(into_var, Value::String(text));
                Ok(ControlFlow::Continue)
            }
        }
    }

    /// Find an element using the multi-strategy finder with retry.
    async fn find_element(&self, target: &Target) -> Result<Element> {
        let config = self.ctx.retry_config.clone();
        retry::retry(&config, || async {
            self.finder.find(target, self.ctx).await
        })
        .await
    }

    /// Execute a click instruction.
    async fn exec_click(&self, target: &Target, button: &MouseButton) -> Result<()> {
        let el = self.find_element(target).await?;
        self.actor.click(&el, button.clone()).await?;
        Ok(())
    }

    /// Wait for an element to appear.
    async fn exec_wait_for(
        &self,
        target: &Target,
        timeout_ms: u64,
        interval_ms: u64,
    ) -> Result<Element> {
        let start = Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);
        let interval = std::time::Duration::from_millis(interval_ms);

        loop {
            match self.finder.find(target, self.ctx).await {
                Ok(el) => return Ok(el),
                Err(_) => {
                    if start.elapsed() >= timeout {
                        return Err(RpaError::Timeout(timeout_ms, format!("{:?}", target)));
                    }
                    tokio::time::sleep(interval).await;
                }
            }
        }
    }

    /// Execute a loop instruction.
    async fn exec_loop(
        &mut self,
        max: &Option<u32>,
        condition: &Option<Condition>,
        body: &[Instruction],
    ) -> Result<ControlFlow> {
        let max_iterations = max.unwrap_or(u32::MAX);
        let mut iteration: u32 = 0;

        while iteration < max_iterations {
            if self.cancellation.is_cancelled() {
                return Err(RpaError::Cancelled);
            }

            if let Some(cond) = condition {
                if !self.evaluate_condition(cond) {
                    break;
                }
            }

            match self.execute_block(body).await? {
                Some(value) => return Ok(ControlFlow::Return(value)),
                None => {}
            }

            iteration += 1;
        }

        Ok(ControlFlow::Continue)
    }

    /// Execute an if instruction.
    async fn exec_if(
        &mut self,
        condition: &Condition,
        then_body: &[Instruction],
        else_body: &Option<Vec<Instruction>>,
    ) -> Result<ControlFlow> {
        let result = if self.evaluate_condition(condition) {
            self.execute_block(then_body).await?
        } else if let Some(else_body) = else_body {
            self.execute_block(else_body).await?
        } else {
            None
        };

        match result {
            Some(value) => Ok(ControlFlow::Return(value)),
            None => Ok(ControlFlow::Continue),
        }
    }

    /// Execute a drag instruction (from one target to another).
    async fn exec_drag(&self, from: &Target, to: &Target, button: MouseButton) -> Result<()> {
        let from_el = self.find_element(from).await?;
        let to_el = self.find_element(to).await?;
        let (from_x, from_y) = from_el.center();
        let (to_x, to_y) = to_el.center();

        self.actor.mouse_move(from_x, from_y).await?;
        self.actor.mouse_down(button.clone(), from_x, from_y).await?;
        // Small delay between down and up
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        self.actor.mouse_move(to_x, to_y).await?;
        self.actor.mouse_up(button.clone(), to_x, to_y).await?;
        Ok(())
    }

    /// Execute a screenshot instruction.
    async fn exec_screenshot(
        &self,
        _target: Option<&Target>,
        region: Option<&Rect>,
        save_path: Option<&str>,
    ) -> Result<()> {
        let image_data = self.actor.screenshot(region.cloned()).await?;
        if let Some(path) = save_path {
            std::fs::write(path, &image_data)
                .map_err(|e| RpaError::ScreenshotFailed(format!("Failed to write screenshot: {}", e)))?;
        }
        Ok(())
    }

    /// Execute an OCR region instruction.
    async fn exec_ocr_region(&self, _target: &Target, _region: &Rect) -> Result<String> {
        // OcrRegion requires an OcrEngine which is not wired to Executor directly.
        // The typical flow would be: find window → screenshot window → OCR region.
        // For now, return an error indicating this needs more wiring.
        Err(RpaError::InvalidInstruction(
            "OcrRegion requires an OcrEngine to be wired to the VM".into(),
        ))
    }

    /// Evaluate a condition against the current context.
    fn evaluate_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::VarEquals { var, value } => self
                .ctx
                .get_var(var)
                .map(|v| v == value)
                .unwrap_or(false),
            Condition::VarNotEmpty { var } => self
                .ctx
                .get_var(var)
                .map(|v| !v.is_null())
                .unwrap_or(false),
            Condition::And(conditions) => conditions.iter().all(|c| self.evaluate_condition(c)),
            Condition::Or(conditions) => conditions.iter().any(|c| self.evaluate_condition(c)),
            Condition::Not(c) => !self.evaluate_condition(c),
            _ => {
                tracing::warn!("Condition type not yet fully supported: {:?}", condition);
                true
            }
        }
    }

    /// Emit an execution event.
    fn emit_event(&self, event: ExecutionEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Get the total execution time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Get the number of steps executed.
    pub fn steps_executed(&self) -> u32 {
        self.steps_executed
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

    struct TestActor {
        actions: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl Actor for TestActor {
        async fn click(&self, el: &Element, button: MouseButton) -> Result<()> {
            self.actions.lock().unwrap().push(format!("click:{}:{:?}", el.id, button));
            Ok(())
        }
        async fn double_click(&self, el: &Element) -> Result<()> {
            self.actions.lock().unwrap().push(format!("dblclick:{}", el.id));
            Ok(())
        }
        async fn input_text(&self, el: &Element, text: &str, _clear: bool) -> Result<()> {
            self.actions.lock().unwrap().push(format!("input:{}:{}", el.id, text));
            Ok(())
        }
        async fn key_press(&self, key: &str, _modifiers: Vec<rpa_core::instruction::ModifierKey>) -> Result<()> {
            self.actions.lock().unwrap().push(format!("key:{}", key));
            Ok(())
        }
        async fn scroll(&self, _el: &Element, _dir: ScrollDirection, _amt: u32) -> Result<()> {
            Ok(())
        }
        async fn mouse_move(&self, x: i32, y: i32) -> Result<()> {
            self.actions.lock().unwrap().push(format!("mousemove:{}:{}", x, y));
            Ok(())
        }
        async fn mouse_down(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
            self.actions.lock().unwrap().push(format!("mousedown:{:?}:{}:{}", button, x, y));
            Ok(())
        }
        async fn mouse_up(&self, button: MouseButton, x: i32, y: i32) -> Result<()> {
            self.actions.lock().unwrap().push(format!("mouseup:{:?}:{}:{}", button, x, y));
            Ok(())
        }
        async fn set_foreground(&self, el: &Element) -> Result<()> {
            self.actions.lock().unwrap().push(format!("setforeground:{}", el.id));
            Ok(())
        }
        async fn screenshot(&self, _region: Option<Rect>) -> Result<Vec<u8>> {
            Ok(vec![])
        }
    }

    struct TestPerceptor;

    #[async_trait::async_trait]
    impl Perceptor for TestPerceptor {
        async fn find(&self, _target: &Target, _ctx: &Context) -> Result<Element> {
            Ok(Element {
                id: "test_el".into(),
                bounds: Rect::new(10, 20, 100, 50),
                text: Some("test".into()),
                element_type: Some("Button".into()),
                platform_handle: None,
                process_id: None,
                process_name: None,
                window_title: None,
            })
        }
        async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>> {
            self.find(target, ctx).await.map(|el| vec![el])
        }
    }

    fn make_finder() -> MultiStrategyFinder {
        use crate::finder::{PerceptorEntry, StrategyType};
        MultiStrategyFinder::new(vec![PerceptorEntry {
            strategy_type: StrategyType::UIA,
            perceptor: Arc::new(TestPerceptor),
        }])
    }

    #[tokio::test]
    async fn executor_click() {
        let finder = make_finder();
        let actions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let actor: Arc<dyn Actor> = Arc::new(TestActor {
            actions: actions.clone(),
        });
        let mut ctx = Context::new();
        let cancellation = CancellationToken::new();

        let mut executor = Executor::new(&finder, &actor, &mut ctx, cancellation, None);

        let instr = Instruction::Click {
            target: Target::by_name("按钮"),
            button: MouseButton::Left,
        };

        executor.execute(&instr).await.unwrap();
        assert_eq!(actions.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn executor_wait() {
        let finder = make_finder();
        let actions: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let actor: Arc<dyn Actor> = Arc::new(TestActor {
            actions: actions.clone(),
        });
        let mut ctx = Context::new();
        let cancellation = CancellationToken::new();

        let mut executor = Executor::new(&finder, &actor, &mut ctx, cancellation, None);

        let instr = Instruction::Wait { duration_ms: 10 };
        executor.execute(&instr).await.unwrap();
    }
}