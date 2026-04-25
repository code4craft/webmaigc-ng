use std::{
    num::NonZeroU32,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use async_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use tokio::{
    sync::{mpsc, watch, RwLock},
    time::sleep,
};

use crate::{BoxFuture, Request, SpiderError, SpiderStage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineConfig {
    pub worker_count: usize,
    pub global_channel_capacity: usize,
    pub domain_channel_capacity: usize,
    pub default_domain_qps: NonZeroU32,
    pub max_pages_per_site: Option<usize>,
}

impl EngineConfig {
    pub const fn new(
        worker_count: usize,
        global_channel_capacity: usize,
        domain_channel_capacity: usize,
        default_domain_qps: NonZeroU32,
    ) -> Self {
        Self {
            worker_count,
            global_channel_capacity,
            domain_channel_capacity,
            default_domain_qps,
            max_pages_per_site: None,
        }
    }

    pub const fn with_max_pages_per_site(mut self, max_pages_per_site: usize) -> Self {
        self.max_pages_per_site = Some(max_pages_per_site);
        self
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            worker_count: 128,
            global_channel_capacity: 256,
            domain_channel_capacity: 1024,
            default_domain_qps: NonZeroU32::MIN,
            max_pages_per_site: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RobotsPolicy {
    pub raw: String,
    pub crawl_delay: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainRateLimit {
    pub qps: NonZeroU32,
    pub crawl_delay: Option<Duration>,
}

impl DomainRateLimit {
    pub const fn new(qps: NonZeroU32) -> Self {
        Self {
            qps,
            crawl_delay: None,
        }
    }

    pub fn with_crawl_delay(mut self, crawl_delay: Duration) -> Self {
        self.crawl_delay = Some(crawl_delay);
        self
    }

    pub fn min_interval(&self) -> Duration {
        let quota_interval = Duration::from_secs_f64(1.0 / self.qps.get() as f64);
        match self.crawl_delay {
            Some(crawl_delay) if crawl_delay > quota_interval => crawl_delay,
            _ => quota_interval,
        }
    }
}

#[derive(Debug)]
pub struct DomainSharedState {
    robots: RwLock<Option<RobotsPolicy>>,
    queue_depth: AtomicUsize,
}

impl Default for DomainSharedState {
    fn default() -> Self {
        Self {
            robots: RwLock::new(None),
            queue_depth: AtomicUsize::new(0),
        }
    }
}

impl DomainSharedState {
    pub async fn robots(&self) -> Option<RobotsPolicy> {
        self.robots.read().await.clone()
    }

    pub async fn set_robots(&self, robots: RobotsPolicy) {
        *self.robots.write().await = Some(robots);
    }

    pub fn queue_depth(&self) -> usize {
        self.queue_depth.load(Ordering::Relaxed)
    }

    pub fn increment_queue_depth(&self) {
        self.queue_depth.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_queue_depth(&self) {
        let mut current = self.queue_depth.load(Ordering::Relaxed);
        while current > 0 {
            match self.queue_depth.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureLevel {
    Healthy,
    Constrained,
    Saturated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullDecision {
    Continue,
    SlowDown,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuePressureSnapshot {
    pub queued: usize,
    pub capacity: usize,
}

impl QueuePressureSnapshot {
    pub fn ratio(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            self.queued as f64 / self.capacity as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainBackpressureSnapshot {
    pub domain: String,
    pub queue: QueuePressureSnapshot,
    pub level: BackpressureLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineBackpressureSnapshot {
    pub global: QueuePressureSnapshot,
    pub domains: Vec<DomainBackpressureSnapshot>,
    pub level: BackpressureLevel,
    pub pull_decision: PullDecision,
}

#[derive(Clone)]
pub struct DomainDispatcherHandle {
    pub domain: String,
    pub sender: mpsc::Sender<Request>,
    policy_tx: watch::Sender<DomainRateLimit>,
    state: Arc<DomainSharedState>,
    queue_capacity: usize,
}

impl DomainDispatcherHandle {
    pub fn rate_limit(&self) -> DomainRateLimit {
        *self.policy_tx.borrow()
    }

    pub fn update_rate_limit(&self, qps: NonZeroU32) -> Result<(), SpiderError> {
        let next = DomainRateLimit {
            qps,
            crawl_delay: self.policy_tx.borrow().crawl_delay,
        };

        self.policy_tx
            .send(next)
            .map_err(|err| SpiderError::new(SpiderStage::Schedule, err.to_string()))
    }

    pub fn update_crawl_delay(&self, crawl_delay: Duration) -> Result<(), SpiderError> {
        let next = DomainRateLimit {
            qps: self.policy_tx.borrow().qps,
            crawl_delay: Some(crawl_delay),
        };

        self.policy_tx
            .send(next)
            .map_err(|err| SpiderError::new(SpiderStage::Schedule, err.to_string()))
    }

    pub async fn cache_robots(&self, robots: RobotsPolicy) -> Result<(), SpiderError> {
        if let Some(crawl_delay) = robots.crawl_delay {
            self.update_crawl_delay(crawl_delay)?;
        }

        self.state.set_robots(robots).await;
        Ok(())
    }

    pub async fn robots(&self) -> Option<RobotsPolicy> {
        self.state.robots().await
    }

    pub fn backpressure_snapshot(&self) -> DomainBackpressureSnapshot {
        let queue = QueuePressureSnapshot {
            queued: self.state.queue_depth(),
            capacity: self.queue_capacity,
        };

        DomainBackpressureSnapshot {
            domain: self.domain.clone(),
            level: classify_backpressure(queue.ratio()),
            queue,
        }
    }
}

#[derive(Default)]
pub struct DomainDispatcherRegistry {
    handles: DashMap<String, DomainDispatcherHandle>,
}

impl DomainDispatcherRegistry {
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn get(&self, domain: &str) -> Option<DomainDispatcherHandle> {
        self.handles.get(domain).map(|entry| entry.value().clone())
    }

    pub fn get_or_insert_with<F>(&self, domain: String, init: F) -> DomainDispatcherHandle
    where
        F: FnOnce(String) -> DomainDispatcherHandle,
    {
        if let Some(existing) = self.handles.get(&domain) {
            return existing.value().clone();
        }

        let handle = init(domain.clone());
        self.handles.insert(domain, handle.clone());
        handle
    }
}

#[derive(Clone)]
pub struct SpiderEngine {
    config: EngineConfig,
    registry: Arc<DomainDispatcherRegistry>,
    global_tx: Sender<Request>,
    global_rx: Receiver<Request>,
    global_depth: Arc<AtomicUsize>,
}

impl SpiderEngine {
    pub fn new(config: EngineConfig) -> Self {
        let (global_tx, global_rx) = bounded(config.global_channel_capacity);
        let global_depth = Arc::new(AtomicUsize::new(0));

        Self {
            config,
            registry: Arc::new(DomainDispatcherRegistry::default()),
            global_tx,
            global_rx,
            global_depth,
        }
    }

    pub fn config(&self) -> EngineConfig {
        self.config
    }

    pub fn worker_receiver(&self) -> EngineWorkerReceiver {
        EngineWorkerReceiver {
            receiver: self.global_rx.clone(),
            global_depth: self.global_depth.clone(),
        }
    }

    pub fn worker_count(&self) -> usize {
        self.config.worker_count
    }

    pub fn active_domains(&self) -> usize {
        self.registry.len()
    }

    pub fn backpressure_snapshot(&self) -> EngineBackpressureSnapshot {
        let global = QueuePressureSnapshot {
            queued: self.global_depth.load(Ordering::Relaxed),
            capacity: self.config.global_channel_capacity,
        };

        let mut domains = self
            .registry
            .handles
            .iter()
            .map(|entry| entry.value().backpressure_snapshot())
            .collect::<Vec<_>>();
        domains.sort_by(|left, right| left.domain.cmp(&right.domain));

        let mut level = classify_backpressure(global.ratio());
        for domain in &domains {
            level = max_backpressure(level, domain.level);
        }

        let pull_decision = match level {
            BackpressureLevel::Healthy => PullDecision::Continue,
            BackpressureLevel::Constrained => PullDecision::SlowDown,
            BackpressureLevel::Saturated => PullDecision::Stop,
        };

        EngineBackpressureSnapshot {
            global,
            domains,
            level,
            pull_decision,
        }
    }

    pub fn should_pull_more(&self) -> PullDecision {
        self.backpressure_snapshot().pull_decision
    }

    /// Close the engine's global worker channel so that domain dispatchers stop forwarding
    /// and worker receivers observe end-of-stream once the channel is drained.
    pub fn shutdown(&self) {
        self.global_tx.close();
    }

    pub fn domain_handle(&self, domain: &str) -> Option<DomainDispatcherHandle> {
        self.registry.get(domain)
    }

    pub fn update_domain_rate_limit(
        &self,
        domain: &str,
        qps: NonZeroU32,
    ) -> Result<(), SpiderError> {
        self.registry
            .get(domain)
            .ok_or_else(|| SpiderError::new(SpiderStage::Schedule, "domain dispatcher not found"))?
            .update_rate_limit(qps)
    }

    pub fn update_domain_crawl_delay(
        &self,
        domain: &str,
        crawl_delay: Duration,
    ) -> Result<(), SpiderError> {
        self.registry
            .get(domain)
            .ok_or_else(|| SpiderError::new(SpiderStage::Schedule, "domain dispatcher not found"))?
            .update_crawl_delay(crawl_delay)
    }

    pub fn cache_domain_robots(
        &self,
        domain: &str,
        robots: RobotsPolicy,
    ) -> BoxFuture<'_, Result<(), SpiderError>> {
        let handle = self.registry.get(domain);

        Box::pin(async move {
            handle
                .ok_or_else(|| {
                    SpiderError::new(SpiderStage::Schedule, "domain dispatcher not found")
                })?
                .cache_robots(robots)
                .await
        })
    }

    pub fn dispatch(&self, request: Request) -> BoxFuture<'_, Result<(), SpiderError>> {
        let registry = self.registry.clone();
        let global_tx = self.global_tx.clone();
        let global_depth = self.global_depth.clone();
        let config = self.config;

        Box::pin(async move {
            let domain = request.domain_key()?;
            let handle = registry.get_or_insert_with(domain.clone(), |domain| {
                spawn_domain_dispatcher(
                    domain,
                    config.domain_channel_capacity,
                    config.default_domain_qps,
                    global_tx.clone(),
                    global_depth.clone(),
                )
            });

            handle.state.increment_queue_depth();
            handle.sender.send(request).await.map_err(|err| {
                handle.state.decrement_queue_depth();
                SpiderError::new(SpiderStage::Schedule, err.to_string())
            })
        })
    }
}

#[derive(Clone)]
pub struct EngineWorkerReceiver {
    receiver: Receiver<Request>,
    global_depth: Arc<AtomicUsize>,
}

impl EngineWorkerReceiver {
    pub async fn recv(&self) -> Result<Request, async_channel::RecvError> {
        let request = self.receiver.recv().await?;
        decrement_atomic(self.global_depth.as_ref());
        Ok(request)
    }
}

fn spawn_domain_dispatcher(
    domain: String,
    capacity: usize,
    default_qps: NonZeroU32,
    global_tx: Sender<Request>,
    global_depth: Arc<AtomicUsize>,
) -> DomainDispatcherHandle {
    let (sender, mut receiver) = mpsc::channel::<Request>(capacity);
    let (policy_tx, mut policy_rx) = watch::channel(DomainRateLimit::new(default_qps));
    let state = Arc::new(DomainSharedState::default());
    let state_for_task = state.clone();

    tokio::spawn(async move {
        let mut current_policy = *policy_rx.borrow();
        let mut last_dispatched_at: Option<Instant> = None;

        while let Some(request) = receiver.recv().await {
            state_for_task.decrement_queue_depth();
            if policy_rx.has_changed().unwrap_or(false) {
                let _ = policy_rx.changed().await;
                current_policy = *policy_rx.borrow();
            }

            if let Some(last_at) = last_dispatched_at {
                let min_interval = current_policy.min_interval();
                let elapsed = last_at.elapsed();
                if elapsed < min_interval {
                    sleep(min_interval - elapsed).await;
                }
            }

            global_depth.fetch_add(1, Ordering::Relaxed);
            if global_tx.send(request).await.is_err() {
                decrement_atomic(global_depth.as_ref());
                break;
            }

            last_dispatched_at = Some(Instant::now());
        }
    });

    DomainDispatcherHandle {
        domain,
        sender,
        policy_tx,
        state,
        queue_capacity: capacity,
    }
}

fn classify_backpressure(ratio: f64) -> BackpressureLevel {
    if ratio >= 1.0 {
        BackpressureLevel::Saturated
    } else if ratio >= 0.8 {
        BackpressureLevel::Constrained
    } else {
        BackpressureLevel::Healthy
    }
}

fn max_backpressure(left: BackpressureLevel, right: BackpressureLevel) -> BackpressureLevel {
    use BackpressureLevel::*;

    match (left, right) {
        (Saturated, _) | (_, Saturated) => Saturated,
        (Constrained, _) | (_, Constrained) => Constrained,
        _ => Healthy,
    }
}

fn decrement_atomic(counter: &AtomicUsize) {
    let mut current = counter.load(Ordering::Relaxed);
    while current > 0 {
        match counter.compare_exchange_weak(
            current,
            current - 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return,
            Err(actual) => current = actual,
        }
    }
}
