use crate::{
    network::proto,
    result::WSResult,
    sys::{LogicalModule, LogicalModuleNewArgs, MetricPublisherView},
    util::JoinHandleWrapper,
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};
use sysinfo::{CpuExt, CpuRefreshKind, RefreshKind, System, SystemExt};
use ws_derive::LogicalModule;

#[derive(LogicalModule)]
pub struct MetricPublisher {
    view: MetricPublisherView,
    user_read_block_time: Mutex<VecDeque<u64>>,
    user_hit_query_time: Mutex<VecDeque<u64>>,
    user_read_cnt: AtomicU64,
    user_local_hit_cnt: AtomicU64,
    user_s3_read_cnt: AtomicU64,
    begin_time: AtomicU64,
}

#[async_trait]
impl LogicalModule for MetricPublisher {
    fn inner_new(args: LogicalModuleNewArgs) -> Self
    where
        Self: Sized,
    {
        Self {
            view: MetricPublisherView::new(args.logical_modules_ref.clone()),
            user_read_block_time: Mutex::new(VecDeque::new()),
            user_hit_query_time: Mutex::new(VecDeque::new()),
            user_read_cnt: AtomicU64::new(0),
            user_local_hit_cnt: AtomicU64::new(0),
            user_s3_read_cnt: AtomicU64::new(0),
            begin_time: AtomicU64::new(0),
        }
    }
    async fn start(&self) -> WSResult<Vec<JoinHandleWrapper>> {
        let view = self.view.clone();
        // continuous send metrics to master
        Ok(vec![JoinHandleWrapper::from(tokio::spawn(async move {
            // start_http_handler(view).await;
            report_metric_task(view).await;
        }))])
    }
}

pub enum HitPosition {
    Local,
    Remote,
    S3,
}

impl MetricPublisher {
    pub fn record_read_one_block(
        &self,
        hit_pos: HitPosition,
        _filename: &str,
        _block_id: u32,
        time_ms: usize,
        hit_query_time: u64,
    ) {
        if self.user_read_cnt.load(Ordering::Relaxed) == 0 {
            self.begin_time.store(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                Ordering::Relaxed,
            );
        }
        let _ = self.user_read_cnt.fetch_add(1, Ordering::Relaxed);
        match hit_pos {
            HitPosition::Local => {
                let _ = self.user_local_hit_cnt.fetch_add(1, Ordering::Relaxed);
            }
            HitPosition::Remote => {}
            HitPosition::S3 => {
                let _ = self.user_s3_read_cnt.fetch_add(1, Ordering::Relaxed);
            }
        }
        {
            let mut queue = self.user_read_block_time.lock();
            queue.push_back(time_ms as u64);
            if queue.len() > 300 {
                let _ = queue.pop_front();
            }
        }
        {
            let mut queue = self.user_hit_query_time.lock();
            queue.push_back(hit_query_time);
            if queue.len() > 300 {
                let _ = queue.pop_front();
            }
        }
    }
}

async fn report_metric_task(view: MetricPublisherView) {
    // let mut m = machine_info::Machine::new();
    // let info = m.system_info();
    // let total_mem = info.memory as u32;
    // let total_cpu = (info.processor.frequency * (info.total_processors as u64)) as u32;
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(),
    );
    // First we update all information of our `System` struct.
    sys.refresh_all();
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        sys.refresh_all();
        // let status = m.system_status().unwrap();

        let cpu_all = sys.cpus()[0].frequency() * sys.cpus().len() as u64;
        let cpu_used =
            sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
        let start_read_time = view.metric_publisher().begin_time.load(Ordering::Relaxed);
        let current = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let read_duration = current - start_read_time;

        let total_read = view
            .metric_publisher()
            .user_read_cnt
            .load(Ordering::Relaxed);
        let local_read = view
            .metric_publisher()
            .user_local_hit_cnt
            .load(Ordering::Relaxed);
        let s3_read = view
            .metric_publisher()
            .user_s3_read_cnt
            .load(Ordering::Relaxed);

        let local_read_rate = local_read as f32 / total_read as f32;
        let s3_read_rate = s3_read as f32 / total_read as f32;
        let remote_read_rate = 1.0 - local_read_rate - s3_read_rate;

        let metric = proto::metric::RscMetric {
            cpu_used,
            mem_used: sys.used_memory() as f32,
            cpu_all: cpu_all as f32,
            mem_all: sys.total_memory() as f32,
            avg_block_time: {
                let lock = view.metric_publisher().user_read_block_time.lock();
                lock.iter().sum::<u64>() as f32 / lock.len() as f32
            },
            rps: view
                .metric_publisher()
                .user_read_cnt
                .load(Ordering::Relaxed) as f32
                / read_duration as f32,
            local_read_rate,
            remote_read_rate,
            s3_read_rate,
            hit_query_time: {
                let lock = view.metric_publisher().user_hit_query_time.lock();
                lock.iter().sum::<u64>() as f32 / lock.len() as f32
            },
        };

        // {
        //     tracing::info!(
        //         "total block read {}, local hit {}|{} , s3 read {}|{}",
        //         total,
        //         local_hit,
        //         local_hit as f32 / total as f32,
        //         s3_read,
        //         s3_read as f32 / total as f32
        //     );
        // }

        // println!("send metrics to master");
        if view.p2p().nodes_config.this.1.is_router() {
            view.metric_observor()
                .insert_node_rsc_metric(view.p2p().nodes_config.this.0, metric);
        } else {
            let _res = view.p2p().send_resp(1, 0, metric).await;
        }
    }
}
