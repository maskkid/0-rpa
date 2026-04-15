//! Multi-strategy element finder with fallback chain.

use rpa_core::context::Context;
use rpa_core::element::Element;
use rpa_core::error::{RpaError, Result};
use rpa_core::target::Target;
use rpa_core::traits::Perceptor;

use std::sync::Arc;

/// The type of perception strategy being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyType {
    UIA,
    Text,
    Image,
    Position,
    Window,
    Ocr,
}

/// A perceptor paired with its strategy type.
pub struct PerceptorEntry {
    pub strategy_type: StrategyType,
    pub perceptor: Arc<dyn Perceptor>,
}

/// Multi-strategy element finder that tries perception methods in order.
///
/// Default fallback order: UIA → Text → Image → Position.
pub struct MultiStrategyFinder {
    strategies: Vec<PerceptorEntry>,
}

impl MultiStrategyFinder {
    /// Create a new finder with the given strategy entries.
    pub fn new(strategies: Vec<PerceptorEntry>) -> Self {
        Self { strategies }
    }

    /// Find an element using the fallback chain.
    pub async fn find(&self, target: &Target, ctx: &Context) -> Result<Element> {
        // Direct position resolution
        if let Target::Position { x, y } = target {
            return Ok(Element {
                id: format!("pos_{}_{}", x, y),
                bounds: rpa_core::element::Rect::new(*x, *y, 1, 1),
                text: None,
                element_type: Some("position".into()),
                platform_handle: None,
                process_id: None,
                process_name: None,
                window_title: None,
            });
        }

        let applicable_order = self.applicable_strategies(target);
        let mut last_error = String::new();

        for strategy_type in &applicable_order {
            if let Some(entry) = self.strategies.iter().find(|e| e.strategy_type == *strategy_type) {
                match entry.perceptor.find(target, ctx).await {
                    Ok(el) => return Ok(el),
                    Err(e) => {
                        last_error = format!("{:?}: {}", strategy_type, e);
                        tracing::debug!(
                            strategy = format!("{:?}", strategy_type),
                            error = %e,
                            "Strategy failed, trying next"
                        );
                        continue;
                    }
                }
            }
        }

        Err(RpaError::ElementNotFound(format!(
            "All strategies failed for target {:?}. Last error: {}",
            target, last_error
        )))
    }

    /// Find all elements matching the target.
    pub async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>> {
        if let Target::Position { .. } = target {
            return self.find(target, ctx).await.map(|el| vec![el]);
        }

        let applicable_order = self.applicable_strategies(target);

        for strategy_type in &applicable_order {
            if let Some(entry) = self.strategies.iter().find(|e| e.strategy_type == *strategy_type) {
                match entry.perceptor.find_all(target, ctx).await {
                    Ok(els) => return Ok(els),
                    Err(e) => {
                        tracing::debug!(
                            strategy = format!("{:?}", strategy_type),
                            error = %e,
                            "Strategy failed for find_all, trying next"
                        );
                        continue;
                    }
                }
            }
        }

        Err(RpaError::ElementNotFound(format!(
            "All strategies failed for find_all on target {:?}",
            target
        )))
    }

    fn applicable_strategies(&self, target: &Target) -> Vec<StrategyType> {
        match target {
            Target::UIA { .. } => {
                vec![StrategyType::UIA, StrategyType::Text, StrategyType::Image]
            }
            Target::Text { .. } => {
                vec![StrategyType::Text, StrategyType::UIA, StrategyType::Image]
            }
            Target::Image { .. } => {
                vec![StrategyType::Image, StrategyType::UIA, StrategyType::Text]
            }
            Target::Position { .. } => vec![],
            Target::Window { .. } => vec![StrategyType::Window],
            Target::Region { .. } => vec![StrategyType::Window],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpa_core::context::Context;
    use rpa_core::element::{Element, Rect};
    use rpa_core::target::Target;

    fn make_element(id: &str) -> Element {
        Element {
            id: id.into(),
            bounds: Rect::new(0, 0, 100, 50),
            text: Some(id.into()),
            element_type: Some("Button".into()),
            platform_handle: None,
            process_id: None,
            process_name: None,
            window_title: None,
        }
    }

    struct MockPerceptor {
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl Perceptor for MockPerceptor {
        async fn find(&self, _target: &Target, _ctx: &Context) -> Result<Element> {
            if self.should_fail {
                Err(RpaError::ElementNotFound("mock fail".into()))
            } else {
                Ok(make_element("mock"))
            }
        }

        async fn find_all(&self, target: &Target, ctx: &Context) -> Result<Vec<Element>> {
            self.find(target, ctx).await.map(|el| vec![el])
        }
    }

    #[tokio::test]
    async fn find_position_directly() {
        let finder = MultiStrategyFinder::new(vec![]);
        let ctx = Context::new();
        let target = Target::at(100, 200);
        let el = finder.find(&target, &ctx).await.unwrap();
        assert_eq!(el.id, "pos_100_200");
    }

    #[tokio::test]
    async fn find_fallback_chain() {
        let finder = MultiStrategyFinder::new(vec![
            PerceptorEntry {
                strategy_type: StrategyType::UIA,
                perceptor: Arc::new(MockPerceptor { should_fail: true }),
            },
            PerceptorEntry {
                strategy_type: StrategyType::Text,
                perceptor: Arc::new(MockPerceptor { should_fail: false }),
            },
        ]);

        let ctx = Context::new();
        let target = Target::by_name("按钮");
        let el = finder.find(&target, &ctx).await.unwrap();
        assert_eq!(el.id, "mock");
    }

    #[tokio::test]
    async fn find_all_fail() {
        let finder = MultiStrategyFinder::new(vec![PerceptorEntry {
            strategy_type: StrategyType::UIA,
            perceptor: Arc::new(MockPerceptor { should_fail: true }),
        }]);

        let ctx = Context::new();
        let target = Target::by_name("按钮");
        let result = finder.find(&target, &ctx).await;
        assert!(result.is_err());
    }
}